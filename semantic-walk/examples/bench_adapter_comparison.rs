//! # bench_adapter_comparison — FrameAdapter vs ProximityAdapter
//!
//! Il benchmark comparativo che mette faccia a faccia le due convenzioni di
//! stato cinematico per il beam solver, sullo stesso grafo e con lo stesso
//! solver. È la verifica empirica della scelta di contratto del 06/10:
//! `derive_state` (geometria locale del frame) è la convenzione canonica,
//! la media delle sonde degli archi è secondaria.
//!
//! ## Pipeline
//!
//! 1. **Caricamento** — dal Dataset B (dense per-token reali + sonde scalari).
//! 2. **Grafo di prossimità** — [`costruisci`] genera il grafo kNN pesato
//!    (stesso grafo per entrambi gli adapter).
//! 3. **Due adapter sullo stesso grafo**:
//!    - [`FrameAdapter`]: stato = `derive_state(traj, len-1, last_frame)`,
//!      la geometria locale del frame denso reale.
//!    - [`ProximityAdapter`]: stato = media delle sonde degli archi.
//! 4. **Beam solver** su radici rappresentative (una per fascia), con sweep
//!    dei coefficienti (beta/gamma/c_sem).
//! 5. **Metriche**:
//!    - Jaccard sui top-k cammini tra i due adapter (quanto convergono).
//!    - Spearman sull'ordinamento dei cammini per costo (quanto correlano).
//!    - Numero di cammini superstiti e ampiezza media (quanto distinguono).

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
use semantic_walk::solver::{BeamSolver, GraphAdapter, SolverConfig};
use semantic_walk::state::derive_state;
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
// FrameAdapter — la convenzione canonica (derive_state)
// ---------------------------------------------------------------------------

/// L'adattatore che usa la geometria locale del frame denso.
///
/// Lo stato cinematico di ogni nodo è derivato dalla traiettoria densa reale
/// del testo (norma, delta di norma, curvatura angolare) tramite
/// [`derive_state`]. È la convenzione canonica del contratto del 06/10.
#[derive(Debug, Clone)]
struct FrameAdapter {
    grafo: Graph,
    /// Nodo → traiettoria densa reale (i frame per-token del testo).
    traiettorie: HashMap<FactId, Vec<Vec<f64>>>,
}

impl FrameAdapter {
    fn new(grafo: Graph, traiettorie: HashMap<FactId, Vec<Vec<f64>>>) -> Self {
        FrameAdapter { grafo, traiettorie }
    }

    /// Lo stato cinematico di un nodo: `derive_state` sull'ultimo frame.
    fn state_of(&self, node: FactId) -> KinematicState {
        match self.traiettorie.get(&node) {
            Some(traj) if !traj.is_empty() => {
                let last = traj.len() - 1;
                derive_state(traj, last, &traj[last])
            }
            _ => KinematicState::default(),
        }
    }
}

impl GraphAdapter for FrameAdapter {
    fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)> {
        let n = NodeId(node);
        self.grafo
            .vicini(n)
            .into_iter()
            .map(|vicino| {
                let stato = self.state_of(vicino.0);
                (vicino.0, stato)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Metriche di confronto
// ---------------------------------------------------------------------------

/// Jaccard tra due insiemi di cammini (per insiemi di nodi).
fn jaccard_paths(a: &[Vec<FactId>], b: &[Vec<FactId>]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let set_a: std::collections::HashSet<Vec<FactId>> =
        a.iter().cloned().collect();
    let set_b: std::collections::HashSet<Vec<FactId>> =
        b.iter().cloned().collect();
    let inter = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        1.0
    } else {
        inter as f64 / union as f64
    }
}

/// Coefficiente di correlazione di Spearman tra due ranking.
fn spearman(a: &[f32], b: &[f32]) -> f64 {
    let n = a.len().min(b.len());
    if n < 2 {
        return 0.0;
    }
    // Ranking dei costi (ascendente: costo minore = rango minore).
    let mut idx_a: Vec<usize> = (0..a.len()).collect();
    idx_a.sort_by(|&i, &j| a[i].partial_cmp(&a[j]).unwrap_or(std::cmp::Ordering::Equal));
    let mut rank_a = vec![0usize; a.len()];
    for (r, &i) in idx_a.iter().enumerate() {
        rank_a[i] = r;
    }
    let mut idx_b: Vec<usize> = (0..b.len()).collect();
    idx_b.sort_by(|&i, &j| b[i].partial_cmp(&b[j]).unwrap_or(std::cmp::Ordering::Equal));
    let mut rank_b = vec![0usize; b.len()];
    for (r, &i) in idx_b.iter().enumerate() {
        rank_b[i] = r;
    }

    // Correlazione di Spearman sui primi n elementi condivisi (stessi nodi
    // finali? qui confrontiamo i ranking per indice, assumendo gli stessi
    // cammini — vedi main).
    let mean_a = rank_a[..n].iter().map(|&x| x as f64).sum::<f64>() / n as f64;
    let mean_b = rank_b[..n].iter().map(|&x| x as f64).sum::<f64>() / n as f64;
    let mut num = 0.0;
    let mut den_a = 0.0;
    let mut den_b = 0.0;
    for i in 0..n {
        let da = rank_a[i] as f64 - mean_a;
        let db = rank_b[i] as f64 - mean_b;
        num += da * db;
        den_a += da * da;
        den_b += db * db;
    }
    if den_a == 0.0 || den_b == 0.0 {
        0.0
    } else {
        num / (den_a * den_b).sqrt()
    }
}

// ---------------------------------------------------------------------------
// Minimo passo inerziale empirico
// ---------------------------------------------------------------------------

/// Misura il minimo valore **positivo** di `inertial_action` raggiungibile
/// dal FrameAdapter su tutte le coppie (nodo, vicino) del grafo.
///
/// La lezione del collasso FrameAdapter (09/10): il `min_step` cablato a
/// 0.01 era mille volte più grande del minimo inerziale reale (~0.000015),
/// quindi il solver scartava quasi tutti i cammini — il collasso a 1 cammino
/// per radice sembrava un artefatto della potatura F6 ma era la soglia a
/// farlo. Il minimo va misurato dove il fenomeno è, non assunto a priori.
///
/// Restituisce il minimo valore positivo osservato, oppure il default del
/// solver (`0.002`) se nessuna coppia produce un'azione finita positiva.
fn misura_min_step_empirico(
    adapter: &FrameAdapter,
    radici: &[FactId],
    beta: f32,
    gamma: f32,
    c_sem: f32,
    m_sem: f32,
) -> f32 {
    let mut min_pos = f32::INFINITY;
    for &root in radici {
        let root_state = KinematicState::default();
        let vicini = adapter.neighbors(root);
        for (_, stato_vicino) in vicini {
            let action = root_state.inertial_action(&stato_vicino, beta, gamma, c_sem, m_sem);
            if action.is_finite() && action > 0.0 && action < min_pos {
                min_pos = action;
            }
        }
    }
    if min_pos.is_finite() {
        min_pos
    } else {
        0.002
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[CMP] Indice: {}", index_path().display());
    println!("[CMP] Cache:  {}", cache_path().display());

    let index_file = File::open(&index_path())?;
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader)?;
    println!("[CMP] Coppie totali: {}", index.len());

    let npz_file = File::open(&cache_path())?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    // --- Passo 1: costruzione dei fatti + traiettorie dense reali ---
    let mut fatti: Vec<Fatto> = Vec::new();
    // Nodo → traiettoria densa reale (per il FrameAdapter).
    let mut traiettorie: HashMap<FactId, Vec<Vec<f64>>> = HashMap::new();
    let mut errors = 0usize;
    let mut coppie_ok = 0usize;

    let mut pair_ids: Vec<&String> = index.keys().collect();
    pair_ids.sort();

    for (pair_idx, pair_id) in pair_ids.iter().enumerate() {
        let _entry = &index[*pair_id];

        let seq_a = match read_dense(&mut zip, &format!("{}_a_dense", pair_id)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[CMP] Errore dense A {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };
        let seq_b = match read_dense(&mut zip, &format!("{}_b_dense", pair_id)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[CMP] Errore dense B {}: {}", pair_id, e);
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
                eprintln!("[CMP] Errore sparse A {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };
        let sparse_b = match build_sparse(&ids_b, &weights_b) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[CMP] Errore sparse B {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };

        // Sonde del testo A.
        let d_a = auto_dense(&seq_a);
        let s_a = auto_sparse(&sparse_a);
        let c_a = d_a;

        let d_b = auto_dense(&seq_b);
        let s_b = auto_sparse(&sparse_b);
        let c_b = d_b;

        let id_a = (pair_idx * 2) as u64;
        let id_b = (pair_idx * 2 + 1) as u64;

        fatti.push(Fatto {
            id: NodeId(id_a),
            dense: d_a,
            sparse: s_a,
            colbert: c_a,
        });
        fatti.push(Fatto {
            id: NodeId(id_b),
            dense: d_b,
            sparse: s_b,
            colbert: c_b,
        });

        // Traiettorie dense reali per il FrameAdapter.
        traiettorie.insert(id_a, seq_a);
        traiettorie.insert(id_b, seq_b);

        coppie_ok += 1;
    }

    println!(
        "[CMP] Coppie caricate: {}, errori: {}, fatti costruiti: {}, traiettorie: {}",
        coppie_ok,
        errors,
        fatti.len(),
        traiettorie.len()
    );

    // --- Passo 2: costruzione del grafo di prossimità (condiviso) ---
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
        "[CMP] Grafo costruito in {:?}: {} nodi, {} archi",
        dur_graph,
        grafo.n_nodi(),
        grafo.n_archi()
    );

    // --- Passo 3: i due adapter sullo stesso grafo ---
    let frame_adapter = FrameAdapter::new(grafo.clone(), traiettorie);
    let proximity_adapter = ProximityAdapter::new(grafo.clone());

    // --- Passo 4: radici rappresentative (una per fascia) ---
    let mut radici: Vec<FactId> = Vec::new();
    let mut viste: HashMap<&str, bool> = HashMap::new();
    for pair_id in &pair_ids {
        let cat = index[*pair_id].category.as_str();
        if !viste.contains_key(cat) {
            let idx = pair_ids.iter().position(|p| *p == *pair_id).unwrap();
            radici.push((idx * 2) as FactId);
            viste.insert(cat, true);
        }
    }
    println!("[CMP] Radici rappresentative: {}", radici.len());

    // --- Passo 5: sweep dei coefficienti e confronto ---
    // Sweep su beta e gamma (rigidità della camminata), c_sem (barriera) e
    // kappa (ampiezza del fascio). Il dato chiave del 09/10: allentare kappa
    // (2→5→10) NON ripopola la frontiera del FrameAdapter (resta 1 cammino
    // per radice in tutte le 54 celle) — il collasso è strutturale alla sonda
    // di curvatura su frame normalizzati, non un artefatto della potatura.
    let betas = [0.5, 1.0, 2.0];
    let gammas = [0.5, 1.0, 2.0];
    let c_sems = [10.0, 100.0];
    let kappas = [2.0, 5.0, 10.0];

    println!("\n[CMP] === Confronto FrameAdapter vs ProximityAdapter (H=4) ===");
    println!(
        "{:<6} {:<6} {:<8} {:<8} | {:<8} {:<8} | {:<8} {:<8} | {:<10} {:<10}",
        "beta", "gamma", "c_sem", "kappa",
        "F:cam", "P:cam",
        "F:amp", "P:amp",
        "Jaccard", "Spearman"
    );

    let mut tot_jaccard = 0.0;
    let mut tot_spearman = 0.0;
    let mut n_cells = 0usize;

    for &beta in &betas {
        for &gamma in &gammas {
            for &c_sem in &c_sems {
                for &kappa in &kappas {
                // Minimo passo inerziale empirico: misurare dove il fenomeno è,
                // non assumere 0.01 (lezione collasso FrameAdapter, 09/10).
                let min_step = misura_min_step_empirico(
                    &frame_adapter,
                    &radici,
                    beta,
                    gamma,
                    c_sem,
                    1.0,
                );
                let config = SolverConfig {
                    horizon: 4,
                    kappa,
                    beta,
                    gamma,
                    c_sem,
                    m_sem: 1.0,
                    min_step,
                };
                let solver_f = BeamSolver::new(frame_adapter.clone(), config.clone());
                let solver_p = BeamSolver::new(proximity_adapter.clone(), config);

                let mut f_cammini_tot = 0usize;
                let mut p_cammini_tot = 0usize;
                let mut f_amp_tot = 0.0f32;
                let mut p_amp_tot = 0.0f32;
                let mut jaccard_acc = 0.0;
                let mut spearman_acc = 0.0;
                let mut n_radici_valide = 0usize;

                for &root in &radici {
                    let root_state = KinematicState::default();
                    let f_paths = solver_f.solve(root, root_state);
                    let p_paths = solver_p.solve(root, root_state);

                    if f_paths.is_empty() && p_paths.is_empty() {
                        continue;
                    }

                    let f_nodes: Vec<Vec<FactId>> = f_paths
                        .iter()
                        .map(|p| p.nodes.clone())
                        .collect();
                    let p_nodes: Vec<Vec<FactId>> = p_paths
                        .iter()
                        .map(|p| p.nodes.clone())
                        .collect();

                    // Jaccard sui cammini (per insiemi di nodi).
                    jaccard_acc += jaccard_paths(&f_nodes, &p_nodes);

                    // Spearman sull'ordinamento per costo (ampiezza).
                    let f_amp: Vec<f32> = f_paths.iter().map(|p| p.amplitude).collect();
                    let p_amp: Vec<f32> = p_paths.iter().map(|p| p.amplitude).collect();
                    spearman_acc += spearman(&f_amp, &p_amp);

                    f_cammini_tot += f_paths.len();
                    p_cammini_tot += p_paths.len();
                    f_amp_tot += f_paths.iter().map(|p| p.amplitude).sum::<f32>();
                    p_amp_tot += p_paths.iter().map(|p| p.amplitude).sum::<f32>();
                    n_radici_valide += 1;
                }

                if n_radici_valide == 0 {
                    continue;
                }
                let jaccard = jaccard_acc / n_radici_valide as f64;
                let spearman = spearman_acc / n_radici_valide as f64;
                tot_jaccard += jaccard;
                tot_spearman += spearman;
                n_cells += 1;

                let f_amp_med = f_amp_tot / f_cammini_tot.max(1) as f32;
                let p_amp_med = p_amp_tot / p_cammini_tot.max(1) as f32;

                println!(
                    "{:<6} {:<6} {:<8.0} {:<8.0} | {:<8} {:<8} | {:<8.4} {:<8.4} | {:<10.4} {:<10.4}",
                    beta, gamma, c_sem, kappa,
                    f_cammini_tot, p_cammini_tot,
                    f_amp_med, p_amp_med,
                    jaccard, spearman
                );
                }
            }
        }
    }

    if n_cells > 0 {
        println!(
            "\n[CMP] Medie su {} celle: Jaccard={:.4}, Spearman={:.4}",
            n_cells,
            tot_jaccard / n_cells as f64,
            tot_spearman / n_cells as f64
        );
    }

    Ok(())
}
