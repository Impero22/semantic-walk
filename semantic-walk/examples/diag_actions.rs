//! # diag_actions — diagnostica della distribuzione dei costi inerziali
//!
//! Misura la distribuzione dell'azione inerziale `S_Inertial` sugli archi
//! reali del grafo di prossimità costruito dal Dataset B, per capire perché
//! il beam solver non pota (357.521 cammini per radice).
//!
//! Ipotesi: la scala di ampiezza è compressa — i costi cumulati sono piccoli
//! rispetto a `kappa`, quindi tutti gli upper bound stanno in un intervallo
//! stretto e la soglia relativa `incumbent − PRUNE_EPSILON` non scarta nulla.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_graph::{costruisci, Fatto, Graph, GraphConfig, NodeId};
use semantic_walk::ordered_sparse::OrderedSparseSequence;
use semantic_walk::KinematicState;

fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_b")
}

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

fn auto_sparse(sparse: &OrderedSparseSequence) -> f64 {
    let overlap = sparse.global_overlap(sparse);
    (overlap as f64) / SIGNATURE_BITS
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_file = File::open(data_dir().join("dataset_b_index.json"))?;
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader)?;

    let npz_file = File::open(data_dir().join("dataset_b_cache_fixed.npz"))?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    let mut fatti: Vec<Fatto> = Vec::new();
    let mut _errors = 0usize;

    let mut pair_ids: Vec<&String> = index.keys().collect();
    pair_ids.sort();

    for (pair_idx, pair_id) in pair_ids.iter().enumerate() {
        let seq_a = match read_dense(&mut zip, &format!("{}_a_dense", pair_id)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Errore dense A {}: {}", pair_id, e);
                _errors += 1;
                continue;
            }
        };
        let seq_b = match read_dense(&mut zip, &format!("{}_b_dense", pair_id)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Errore dense B {}: {}", pair_id, e);
                _errors += 1;
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
                eprintln!("Errore sparse A {}: {}", pair_id, e);
                _errors += 1;
                continue;
            }
        };
        let sparse_b = match build_sparse(&ids_b, &weights_b) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Errore sparse B {}: {}", pair_id, e);
                _errors += 1;
                continue;
            }
        };

        let d_a = auto_dense(&seq_a);
        let s_a = auto_sparse(&sparse_a);
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
    }

    let config = GraphConfig {
        k: 5,
        soglia: 0.5,
        percentile_cutoff: 0.0,
        pesi: [0.6, 0.25, 0.15],
    };
    let grafo: Graph = costruisci(&fatti, config);
    println!("Grafo: {} nodi, {} archi", grafo.n_nodi(), grafo.n_archi());

    // Parametri del benchmark (SolverConfig del bench end-to-end).
    let beta = 1.0f32;
    let gamma = 1.0f32;
    let c_sem = 100.0f32;
    let m_sem = 1.0f32;
    let kappa = 0.05f32;

    // Ricostruisce lo stato intrinseco di un nodo come media delle sonde dei
    // suoi archi (stessa convenzione di ProximityAdapter::stato_intrinseco).
    fn stato_intrinseco(grafo: &Graph, n: NodeId) -> Option<KinematicState> {
        let archi: Vec<_> = grafo
            .archi
            .iter()
            .copied()
            .filter(|e| e.from == n || e.to == n)
            .collect();
        if archi.is_empty() {
            return None;
        }
        let count = archi.len() as f64;
        let velocity = archi.iter().map(|e| e.score).sum::<f64>() / count;
        let acceleration = archi.iter().map(|e| e.dense).sum::<f64>() / count;
        let curvature = archi
            .iter()
            .map(|e| (e.sparse + e.colbert) / 2.0)
            .sum::<f64>()
            / count;
        Some(KinematicState {
            velocity: velocity as f32,
            acceleration: acceleration as f32,
            curvature: curvature as f32,
        })
    }

    // Misura l'azione inerziale su tutti gli archi del grafo, usando gli
    // stati intrinseci (da→to).
    let mut azioni: Vec<f32> = Vec::new();
    let mut inf = 0usize;
    for e in &grafo.archi {
        let s_from = stato_intrinseco(&grafo, e.from).unwrap_or_default();
        let s_to = stato_intrinseco(&grafo, e.to).unwrap_or_default();
        let a = s_from.inertial_action(&s_to, beta, gamma, c_sem, m_sem);
        if a.is_infinite() {
            inf += 1;
        } else {
            azioni.push(a);
        }
    }

    azioni.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = azioni.len();
    let min = azioni.first().copied().unwrap_or(0.0);
    let max = azioni.last().copied().unwrap_or(0.0);
    let mean = azioni.iter().sum::<f32>() / n.max(1) as f32;
    let p50 = azioni[n / 2];
    let p90 = azioni[(n as f64 * 0.9) as usize];
    let p99 = azioni[(n as f64 * 0.99) as usize];

    println!("\n=== Distribuzione azione inerziale sugli archi ===");
    println!("archi totali: {} (inf: {})", azioni.len() + inf, inf);
    println!("min:  {:.6}", min);
    println!("p50:  {:.6}", p50);
    println!("p90:  {:.6}", p90);
    println!("p99:  {:.6}", p99);
    println!("max:  {:.6}", max);
    println!("mean: {:.6}", mean);

    // Cosa implica per il beam: con kappa=2, l'ampiezza di un cammino di H
    // passi è exp(−S_cum/2). Quanto varia l'ampiezza per i cammini tipici?
    println!("\n=== Implicazione per il beam (kappa=2, H=4) ===");
    for passi in 1..=4usize {
        let s_min = min * passi as f32;
        let s_p50 = p50 * passi as f32;
        let s_p90 = p90 * passi as f32;
        let amp_min = (-s_min / kappa).exp();
        let _amp_p50 = (-s_p50 / kappa).exp();
        let amp_p90 = (-s_p90 / kappa).exp();
        println!(
            "H={}: ampiezza cumulativa per costo {:.4}..{:.4} → {:.4}..{:.4}",
            passi, s_min, s_p90, amp_min, amp_p90
        );
    }

    // Quanto è largo l'intervallo di ampiezza rispetto a PRUNE_EPSILON=0.002?
    let s_min_h4 = min * 4.0;
    let s_p90_h4 = p90 * 4.0;
    let amp_range = ((-s_min_h4 / kappa).exp()) - ((-s_p90_h4 / kappa).exp());
    println!("\nAmpiezza range H=4 (min→p90): {:.4}", amp_range);
    println!("PRUNE_EPSILON = 0.002 → la potatura scarta solo se < incumbent − 0.002");
    println!(
        "Se il range di ampiezza è {} < 0.002, la potatura non scarta nulla.",
        if amp_range < 0.002 { "PIÙ PICCOLO" } else { "più grande" }
    );

    Ok(())
}