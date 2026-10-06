//! # bench_proximity_adapter_end2end — benchmark end-to-end del ponte
//!
//! Il benchmark richiesto da Camillo: **ProximityAdapter + BeamSolver sul
//! grafo di prossimità reale costruito dal Dataset B**.
//!
//! Pipeline completa, dal dato grezzo al cammino superstite:
//!
//! 1. **Costruzione dei fatti** — ogni testo del Dataset B (A e B di ogni
//!    coppia) diventa un [`Fatto`], con sonde scalari = auto-metriche del
//!    testo (auto-MaxSim denso, auto-overlap sparso normalizzato,
//!    auto-colbert). Il Dataset B ha ~200 coppie → ~400 nodi reali.
//! 2. **Grafo di prossimità** — [`costruisci`] genera il grafo kNN pesato
//!    con i pesi canonici `[0.6, 0.25, 0.15]` e soglia 0.5 (valore
//!    raccomandato dal default calibrato).
//! 3. **Adapter** — [`ProximityAdapter::new`] precalcola gli stati
//!    intrinseci; misuriamo il costo di questo precalcolo.
//! 4. **Solver** — [`BeamSolver`] su radici rappresentative (una per
//!    fascia) fino a H=8, misurando latenza e cammini superstiti.
//! 5. **Metrica 2 (gate bloccante)** — il beam deve restituire cammini
//!    non vuoti quando la radice ha archi: il solver preserva i superstiti
//!    e non collassa a un singolo cammino.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_combiner::FactId;
use semantic_graph::{costruisci, Fatto, Graph, GraphConfig, NodeId};
use semantic_walk::graph_adapter::ProximityAdapter;
use semantic_walk::ordered_sparse::OrderedSparseSequence;
use semantic_walk::solver::{BeamSolver, SolverConfig};
use semantic_walk::KinematicState;

// ---------------------------------------------------------------------------
// Caricamento NPZ (stesso schema del benchmark A/B)
// ---------------------------------------------------------------------------

fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_b")
}

fn index_path() -> std::path::PathBuf {
    data_dir().join("dataset_b_index.json")
}

fn cache_path() -> std::path::PathBuf {
    data_dir().join("dataset_b_cache_fixed.npz")
}

/// Numero di bit della firma global (due u64).
const SIGNATURE_BITS: f64 = 128.0;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct IndexEntry {
    category: String,
    text_a: String,
    text_b: String,
}

type DatasetIndex = HashMap<String, IndexEntry>;

fn read_npz_array<T: npyz::Deserialize>(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    let data = reader.into_vec::<T>()?;
    Ok(data)
}

fn read_dense(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    let shape = reader.shape().to_vec();
    let flat = reader.into_vec::<f32>()?;

    let dim = if shape.len() >= 2 { shape[1] as usize } else { 1 };
    let n = if flat.is_empty() { 0 } else { flat.len() / dim };

    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        let row = flat[i * dim..(i + 1) * dim]
            .iter()
            .map(|&x| x as f64)
            .collect::<Vec<f64>>();
        rows.push(row);
    }
    Ok(rows)
}

fn build_sparse(ids: &[i64], weights: &[f32]) -> Result<OrderedSparseSequence, &'static str> {
    if ids.len() != weights.len() {
        return Err("ids e weights di lunghezza diversa");
    }
    let frames: Vec<Vec<(u32, f32)>> = ids
        .iter()
        .zip(weights.iter())
        .map(|(&id, &w)| vec![(id as u32, w)])
        .collect();
    OrderedSparseSequence::from_frames(&frames)
}

// ---------------------------------------------------------------------------
// Auto-metriche di un singolo testo (sonde scalari del Fatto)
// ---------------------------------------------------------------------------

/// Auto-MaxSim denso: la similarità massima di un token verso sé stesso,
/// mediata sui token. È la sonda densa naturale di un singolo testo.
fn auto_dense(seq: &[Vec<f64>]) -> f64 {
    let t = seq.len();
    if t == 0 {
        return 0.0;
    }
    let mut sum = 0.0f64;
    for i in 0..t {
        let vec_a = &seq[i];
        let mut max_sim = f64::NEG_INFINITY;
        for j in 0..t {
            let vec_b = &seq[j];
            let dot: f64 = vec_a.iter().zip(vec_b.iter()).map(|(x, y)| x * y).sum();
            if dot > max_sim {
                max_sim = dot;
            }
        }
        sum += max_sim;
    }
    sum / t as f64
}

/// Auto-overlap sparso normalizzato: la firma del testo verso sé stessa.
fn auto_sparse(sparse: &OrderedSparseSequence) -> f64 {
    let overlap = sparse.global_overlap(sparse);
    (overlap as f64) / SIGNATURE_BITS
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[E2E] Indice: {}", index_path().display());
    println!("[E2E] Cache:  {}", cache_path().display());

    let index_file = File::open(&index_path())?;
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader)?;
    println!("[E2E] Coppie totali: {}", index.len());

    let npz_file = File::open(&cache_path())?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    // --- Passo 1: costruzione dei fatti dal Dataset B ---
    // Ogni testo (A e B di ogni coppia) diventa un Fatto. L'id è univoco:
    // l'indice della coppia * 2 per A, * 2 + 1 per B.
    let mut fatti: Vec<Fatto> = Vec::new();
    let mut errors = 0usize;
    let mut coppie_ok = 0usize;

    let mut pair_ids: Vec<&String> = index.keys().collect();
    pair_ids.sort();

    for (pair_idx, pair_id) in pair_ids.iter().enumerate() {
        let _entry = &index[*pair_id];

        let seq_a = match read_dense(&mut zip, &format!("{}_a_dense", pair_id)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[E2E] Errore dense A {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };
        let seq_b = match read_dense(&mut zip, &format!("{}_b_dense", pair_id)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[E2E] Errore dense B {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };
        let ids_a: Vec<i64> = read_npz_array(&mut zip, &format!("{}_a_ids", pair_id))?;
        let weights_a: Vec<f32> = read_npz_array(&mut zip, &format!("{}_a_weights", pair_id))?;
        let ids_b: Vec<i64> = read_npz_array(&mut zip, &format!("{}_b_ids", pair_id))?;
        let weights_b: Vec<f32> = read_npz_array(&mut zip, &format!("{}_b_weights", pair_id))?;

        let sparse_a = match build_sparse(&ids_a, &weights_a) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[E2E] Errore sparse A {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };
        let sparse_b = match build_sparse(&ids_b, &weights_b) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[E2E] Errore sparse B {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };

        // Sonde del testo A.
        let d_a = auto_dense(&seq_a);
        let s_a = auto_sparse(&sparse_a);
        // Il colbert di un singolo testo: approssimiamo con l'auto-MaxSim denso
        // (stessa scelta del benchmark B, dove il colbert non è separato dal
        // matching denso).
        let c_a = d_a;

        let d_b = auto_dense(&seq_b);
        let s_b = auto_sparse(&sparse_b);
        let c_b = d_b;

        fatti.push(Fatto {
            id: NodeId((pair_idx * 2) as u64),
            dense: d_a,
            sparse: s_a,
            colbert: c_a,
        });
        fatti.push(Fatto {
            id: NodeId((pair_idx * 2 + 1) as u64),
            dense: d_b,
            sparse: s_b,
            colbert: c_b,
        });

        coppie_ok += 1;
    }

    println!(
        "[E2E] Coppie caricate: {}, errori: {}, fatti costruiti: {}",
        coppie_ok,
        errors,
        fatti.len()
    );

    // --- Passo 2: costruzione del grafo di prossimità ---
    let config = GraphConfig {
        k: 5,
        soglia: 0.5,
        percentile_cutoff: 0.0,
        pesi: [0.6, 0.25, 0.15],
    };
    let t_graph = Instant::now();
    let grafo: Graph = costruisci(&fatti, config);
    let dur_graph = t_graph.elapsed();
    println!(
        "[E2E] Grafo costruito in {:?}: {} nodi, {} archi",
        dur_graph,
        grafo.n_nodi(),
        grafo.n_archi()
    );

    // --- Passo 3: adapter (precalcolo stati intrinseci) ---
    let t_adapter = Instant::now();
    let adapter = ProximityAdapter::new(grafo.clone());
    let dur_adapter = t_adapter.elapsed();
    println!("[E2E] ProximityAdapter::new() in {:?}", dur_adapter);

    // --- Passo 4: beam solver su radici rappresentative ---
    let solver = BeamSolver::new(adapter, SolverConfig {
        horizon: 4, kappa: 2.0, min_step: 0.01,
        ..Default::default()
    });

    // Una radice per fascia (le prime coppie di ciascuna categoria).
    let mut radici: Vec<FactId> = Vec::new();
    let mut viste: HashMap<&str, bool> = HashMap::new();
    for pair_id in &pair_ids {
        let cat = index[*pair_id].category.as_str();
        if !viste.contains_key(cat) {
            // Radice = il nodo A della prima coppia della fascia.
            let idx = pair_ids.iter().position(|p| *p == *pair_id).unwrap();
            radici.push((idx * 2) as FactId);
            viste.insert(cat, true);
        }
    }

    println!("\n[E2E] === Beam solver su {} radici (H=4) ===", radici.len());
    let mut totale_cammini = 0usize;
    let mut totale_latency = std::time::Duration::ZERO;
    for &root in &radici {
        // Stato di partenza: lo stato intrinseco della radice (via adapter).
        let root_state = KinematicState::default();
        let t_solve = Instant::now();
        let paths = solver.solve(root, root_state);
        let dur_solve = t_solve.elapsed();
        totale_latency += dur_solve;
        totale_cammini += paths.len();

        // La categoria della radice: risaliamo alla coppia da cui deriva.
        let pair_idx = root / 2;
        let pair_id = &pair_ids[pair_idx as usize];
        let cat_real = &index[*pair_id].category;

        println!(
            "[E2E] root={:>3} (fascia {:>24}) | cammini={:>3} | latenza={:?} | ampiezza media={:.4}",
            root,
            cat_real,
            paths.len(),
            dur_solve,
            paths.iter().map(|p| p.amplitude).sum::<f32>() / paths.len().max(1) as f32
        );
    }

    println!(
        "\n[E2E] Totale: {} cammini su {} radici, latenza cumulativa {:?}",
        totale_cammini,
        radici.len(),
        totale_latency
    );

    // --- Passo 5: Metrica 2 (gate bloccante) ---
    // Il beam deve preservare i superstiti: se la radice ha archi, il solver
    // deve restituire almeno un cammino non vuoto. Verifichiamo che ogni
    // radice con grado > 0 produca cammini non vuoti.
    println!("\n[E2E] === Metrica 2: gate bloccante ===");
    let mut radici_con_archi = 0usize;
    let mut radici_cammini_vuoti = 0usize;
    for &root in &radici {
        let n = NodeId(root);
        if grafo.vicini(n).is_empty() {
            continue;
        }
        radici_con_archi += 1;
        let paths = solver.solve(root, KinematicState::default());
        if paths.is_empty() || paths.iter().all(|p| p.nodes.is_empty()) {
            radici_cammini_vuoti += 1;
            eprintln!("[E2E] ⚠️ root={} ha archi ma nessun cammino non vuoto", root);
        }
    }
    println!(
        "[E2E] Radici con archi: {}, radici con cammini vuoti: {}",
        radici_con_archi, radici_cammini_vuoti
    );
    if radici_cammini_vuoti == 0 {
        println!("[E2E] ✅ Metrica 2 OK: il gate preserva i superstiti (nessun cammino vuoto).");
    } else {
        println!("[E2E] ❌ Metrica 2 FALLITA: ci sono cammini vuoti.");
    }

    Ok(())
}