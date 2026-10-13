//! # verify_oracle_real_dataset — Oracolo esaustivo vs BeamSolver (Dataset B)
//!
//! Misura lo scostamento del beam dall'ottimo globale su TUTTE le radici
//! rappresentative (i 200 nodi A, uno per coppia), aggregato per fascia.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_combiner::FactId;
use semantic_graph::{costruisci, Fatto, GraphConfig, NodeId};
use semantic_walk::graph_adapter::ProximityAdapter;
use semantic_walk::ordered_sparse::OrderedSparseSequence;
use semantic_walk::solver::{BeamSolver, GraphAdapter, SolverConfig};
use semantic_walk::KinematicState;

fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_b")
}

fn index_path() -> std::path::PathBuf {
    data_dir().join("dataset_b_index.json")
}

fn cache_path() -> std::path::PathBuf {
    data_dir().join("dataset_b_cache_fixed.npz")
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
    if t <= 1 {
        return 0.0;
    }
    let mut sum = 0.0f64;
    for i in 0..t {
        let vec_a = &seq[i];
        let mut max_sim = f64::NEG_INFINITY;
        for j in 0..t {
            if i == j {
                continue;
            }
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

/// Similarità relazionale A-vs-B: max-pooling simmetrico mediato.
///
/// Per ogni token di A prende la similarità massima verso i token di B, e
/// per ogni token di B la similarità massima verso i token di A; poi media
/// i due pool. Riflette la simmetria della relazione geometrica tra i due
/// fatti della coppia (contratto 13/10, punto 3).
fn relazionale_a_b(seq_a: &[Vec<f64>], seq_b: &[Vec<f64>]) -> f64 {
    if seq_a.is_empty() || seq_b.is_empty() {
        return 0.0;
    }
    let mut sum_a_to_b = 0.0f64;
    for vec_a in seq_a.iter() {
        let mut max_sim = f64::NEG_INFINITY;
        for vec_b in seq_b.iter() {
            let dot: f64 = vec_a.iter().zip(vec_b.iter()).map(|(x, y)| x * y).sum();
            if dot > max_sim {
                max_sim = dot;
            }
        }
        sum_a_to_b += max_sim;
    }
    let mut sum_b_to_a = 0.0f64;
    for vec_b in seq_b.iter() {
        let mut max_sim = f64::NEG_INFINITY;
        for vec_a in seq_a.iter() {
            let dot: f64 = vec_b.iter().zip(vec_a.iter()).map(|(x, y)| x * y).sum();
            if dot > max_sim {
                max_sim = dot;
            }
        }
        sum_b_to_a += max_sim;
    }
    let pool_a = sum_a_to_b / seq_a.len() as f64;
    let pool_b = sum_b_to_a / seq_b.len() as f64;
    (pool_a + pool_b) / 2.0
}

/// Oracolo esaustivo: esplora tutti i cammini entro l'orizzonte con
/// cycle-detection e restituisce il costo minimo globale.
/// Misura il minimo incremento inerziale positivo sugli archi reali del grafo,
/// usando gli stati intrinseci dell'adapter. Convenzione del test di
/// riferimento (`beam_solver_su_dataset_b`): il bound deve stare dove il
/// fenomeno è, non sotto (lezione Notepad 81 / commit 2746332).
fn measure_min_step(
    grafo: &semantic_graph::Graph,
    adapter: &ProximityAdapter,
    beta: f32,
    gamma: f32,
) -> f32 {
    let mut min = f32::INFINITY;
    for n in 0..grafo.n_nodi() {
        let from = adapter.state_of(n as FactId);
        for (nb, _) in adapter.neighbors(n as FactId) {
            let to = adapter.state_of(nb);
            if from == to {
                continue;
            }
            let a = from.inertial_action(&to, beta, gamma, 100.0, 1.0);
            if a > 0.0 && a < min {
                min = a;
            }
        }
    }
    if min.is_finite() {
        min
    } else {
        1e-6
    }
}

fn exhaustive_min(
    adapter: &ProximityAdapter,
    root: FactId,
    root_state: KinematicState,
    horizon: usize,
    beta: f32,
    gamma: f32,
    c_sem: f32,
    m_sem: f32,
) -> Option<f32> {
    let mut best_cost = f32::INFINITY;

    let mut stack: Vec<(FactId, KinematicState, f32, usize, HashSet<FactId>)> = Vec::new();

    let mut initial_visited = HashSet::new();
    initial_visited.insert(root);
    stack.push((root, root_state, 0.0, 0, initial_visited));

    while let Some((node, state, cost, depth, visited)) = stack.pop() {
        if depth >= horizon {
            if cost < best_cost {
                best_cost = cost;
            }
            continue;
        }

        let neighbors = adapter.neighbors(node);
        if neighbors.is_empty() {
            if cost < best_cost {
                best_cost = cost;
            }
            continue;
        }

        for (next_node, next_state) in neighbors {
            let step_cost = state.inertial_action(&next_state, beta, gamma, c_sem, m_sem);
            let new_cost = cost + step_cost;

            if new_cost >= best_cost {
                continue;
            }

            if !visited.contains(&next_node) {
                let mut next_visited = visited.clone();
                next_visited.insert(next_node);
                stack.push((next_node, next_state.clone(), new_cost, depth + 1, next_visited));
            }
        }
    }

    if best_cost.is_finite() {
        Some(best_cost)
    } else {
        None
    }
}

/// Minimo del beam: il cammino superstite a costo totale minimo.
fn beam_min(
    solver: &BeamSolver<ProximityAdapter>,
    root: FactId,
    root_state: KinematicState,
) -> Option<f32> {
    let paths = solver.solve(root, root_state);
    paths
        .iter()
        .map(|p| p.total_cost)
        .fold(None, |acc, c| match acc {
            None => Some(c),
            Some(m) => Some(m.min(c)),
        })
}

/// Aggrega gli scostamenti: media, massimo, p90, p99.
fn summarize(deltas: &[f64]) -> (f64, f64, f64, f64) {
    if deltas.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut sorted = deltas.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
    let max = *sorted.last().unwrap();
    let p90 = sorted[(sorted.len() as f64 * 0.90).floor() as usize];
    let p99 = sorted[(sorted.len() as f64 * 0.99).floor() as usize];
    (mean, max, p90, p99)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_file = File::open(&index_path())?;
    let index: DatasetIndex = serde_json::from_reader(BufReader::new(index_file))?;

    let npz_file = File::open(&cache_path())?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    let mut fatti: Vec<Fatto> = Vec::new();
    let mut pair_ids: Vec<&String> = index.keys().collect();
    pair_ids.sort();

    for (pair_idx, pair_id) in pair_ids.iter().enumerate() {
        let seq_a = read_dense(&mut zip, &format!("{}_a_dense", pair_id))?;
        let seq_b = read_dense(&mut zip, &format!("{}_b_dense", pair_id))?;
        let ids_a: Vec<i64> = read_npz_array(&mut zip, &format!("{}_a_ids", pair_id))?;
        let weights_a: Vec<f32> = read_npz_array(&mut zip, &format!("{}_a_weights", pair_id))?;
        let ids_b: Vec<i64> = read_npz_array(&mut zip, &format!("{}_b_ids", pair_id))?;
        let weights_b: Vec<f32> = read_npz_array(&mut zip, &format!("{}_b_weights", pair_id))?;

        let sparse_a = build_sparse(&ids_a, &weights_a)?;
        let sparse_b = build_sparse(&ids_b, &weights_b)?;

        fatti.push(Fatto {
            id: NodeId((pair_idx * 2) as u64),
            colbert: auto_dense(&seq_a),
            sparse: auto_sparse(&sparse_a),
            relazionale: relazionale_a_b(&seq_a, &seq_b),
        });
        fatti.push(Fatto {
            id: NodeId((pair_idx * 2 + 1) as u64),
            colbert: auto_dense(&seq_b),
            sparse: auto_sparse(&sparse_b),
            relazionale: relazionale_a_b(&seq_a, &seq_b),
        });
    }

    let config = GraphConfig {
        k: 5,
        soglia: 0.5,
        percentile_cutoff: 0.0,
        pesi: [0.6, 0.25, 0.15],
    };
    let grafo = costruisci(&fatti, config);
    let adapter = ProximityAdapter::new(grafo.clone());

    let horizon = 4;
    let (beta, gamma, c_sem, m_sem) = (1.0, 1.0, 100.0, 1.0);
    let min_step = measure_min_step(&grafo, &adapter, beta, gamma);
    let solver_config = SolverConfig {
        horizon,
        kappa: 2.0,
        min_step,
        ..Default::default()
    };
    let solver = BeamSolver::new(adapter.clone(), solver_config);

    let radici: Vec<FactId> = (0..pair_ids.len()).map(|i| (i * 2) as FactId).collect();

    let mut delta_abs_all: Vec<f64> = Vec::new();
    let mut delta_rel_all: Vec<f64> = Vec::new();
    let mut per_fascia: HashMap<&'static str, Vec<f64>> = HashMap::new();
    // (pair_id, fascia, delta_rel, oracle, beam) per identificare i casi peggiori
    let mut per_root: Vec<(String, &'static str, f64, f32, f32)> = Vec::new();

    println!("\n[ORACLE] === Scostamento Beam vs Oracolo esaustivo (H={}, {} radici) ===", horizon, radici.len());

    let mut oracle_none = 0usize;
    let mut beam_none = 0usize;

    for &root in &radici {
        let pair_idx = (root / 2) as usize;
        let pair_id = pair_ids[pair_idx];
        let cat = index[pair_id].category.clone();
        let fascia: &'static str = if cat.starts_with("fascia1") {
            "fascia1_parafrasi"
        } else if cat.starts_with("fascia2") {
            "fascia2_trasformazioni"
        } else {
            "fascia3_divergenza"
        };

        let oracle = exhaustive_min(&adapter, root, adapter.state_of(root), horizon, beta, gamma, c_sem, m_sem);
        let beam = beam_min(&solver, root, adapter.state_of(root));

        match (oracle, beam) {
            (Some(o), Some(b)) => {
                let d_abs = (b - o).max(0.0) as f64;
                let d_rel = if o > 0.0 { (d_abs / o as f64) * 100.0 } else { 0.0 };
                delta_abs_all.push(d_abs);
                delta_rel_all.push(d_rel);
                per_fascia.entry(fascia).or_default().push(d_rel);
                per_root.push((pair_id.to_string(), fascia, d_rel, o, b));
            }
            (None, _) => {
                oracle_none += 1;
            }
            (Some(_), None) => {
                beam_none += 1;
            }
        }
    }

    println!("\n[ORACLE] === Riepilogo aggregato ===");
    println!("Radici valutate: {} (oracolo vuoto: {}, beam vuoto: {})", radici.len(), oracle_none, beam_none);

    let (m_abs, mx_abs, p90_abs, p99_abs) = summarize(&delta_abs_all);
    let (m_rel, mx_rel, p90_rel, p99_rel) = summarize(&delta_rel_all);

    println!("\nScostamento ASSOLUTO (Δ costo):");
    println!("  media  = {:.6}", m_abs);
    println!("  p90    = {:.6}", p90_abs);
    println!("  p99    = {:.6}", p99_abs);
    println!("  max    = {:.6}", mx_abs);

    println!("\nScostamento RELATIVO (Δ% sul costo oracolo):");
    println!("  media  = {:.4}%", m_rel);
    println!("  p90    = {:.4}%", p90_rel);
    println!("  p99    = {:.4}%", p99_rel);
    println!("  max    = {:.4}%", mx_rel);

    println!("\nPer fascia (Δ% relativo):");
    for (fascia, deltas) in &per_fascia {
        let (f_m, f_mx, f_p90, _) = summarize(deltas);
        println!(
            "  {:<24} n={:<4} media={:.4}%  p90={:.4}%  max={:.4}%",
            fascia,
            deltas.len(),
            f_m,
            f_p90,
            f_mx
        );
    }

    // TOP 15 casi peggiori per-radice
    per_root.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    println!("\n=== TOP 15 scostamenti per-radice ===");
    for (pair_id, fascia, d_rel, o, b) in per_root.iter().take(15) {
        println!(
            "  pair={:<10} fascia={:<24} Δrel={:>8.2}%  oracle={:.4} beam={:.4}",
            pair_id, fascia, d_rel, o, b
        );
    }

    Ok(())
}
