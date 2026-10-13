//! Dissezione di un pair_id arbitrario (default pair_196) — confronta il cammino
//! oracolo esaustivo vs beam, passo per passo, con lo stato reale del root.
//!
//! Uso: cargo run --example diag_pair -- <pair_id> [orizzonte]
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
fn index_path() -> std::path::PathBuf { data_dir().join("dataset_b_index.json") }
fn cache_path() -> std::path::PathBuf { data_dir().join("dataset_b_cache_fixed.npz") }
const SIGNATURE_BITS: f64 = 128.0;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct IndexEntry { category: String, text_a: String, text_b: String }
type DatasetIndex = HashMap<String, IndexEntry>;

fn read_npz_array<T: npyz::Deserialize>(zip: &mut ZipArchive<BufReader<File>>, name: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    Ok(reader.into_vec::<T>()?)
}
fn read_dense(zip: &mut ZipArchive<BufReader<File>>, name: &str) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    let shape = reader.shape().to_vec();
    let flat = reader.into_vec::<f32>()?;
    let dim = if shape.len() >= 2 { shape[1] as usize } else { 1 };
    let n = if flat.is_empty() { 0 } else { flat.len() / dim };
    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        rows.push(flat[i*dim..(i+1)*dim].iter().map(|&x| x as f64).collect::<Vec<f64>>());
    }
    Ok(rows)
}
fn build_sparse(ids: &[i64], weights: &[f32]) -> Result<OrderedSparseSequence, &'static str> {
    if ids.len() != weights.len() { return Err("len"); }
    let frames: Vec<Vec<(u32, f32)>> = ids.iter().zip(weights.iter()).map(|(&id, &w)| vec![(id as u32, w)]).collect();
    OrderedSparseSequence::from_frames(&frames)
}
fn auto_dense(seq: &[Vec<f64>]) -> f64 {
    if seq.is_empty() { return 0.0; }
    let n = seq.len();
    let mut total = 0.0; let mut count = 0;
    for i in 0..n {
        let mut best = -1.0f64;
        for j in 0..n {
            if i == j { continue; }
            let dot: f64 = seq[i].iter().zip(seq[j].iter()).map(|(a,b)| a*b).sum();
            let na: f64 = seq[i].iter().map(|x| x*x).sum::<f64>().sqrt();
            let nb: f64 = seq[j].iter().map(|x| x*x).sum::<f64>().sqrt();
            let sim = if na>0.0 && nb>0.0 { dot/(na*nb) } else { 0.0 };
            if sim > best { best = sim; }
        }
        if best >= 0.0 { total += best; count += 1; }
    }
    if count == 0 { 0.0 } else { total / count as f64 }
}
fn auto_sparse(sparse: &OrderedSparseSequence) -> f64 {
    let overlap = sparse.global_overlap(sparse);
    (overlap as f64) / SIGNATURE_BITS
}
fn relazionale_a_b(seq_a: &[Vec<f64>], seq_b: &[Vec<f64>]) -> f64 {
    let la = seq_a.len() as f64;
    let lb = seq_b.len() as f64;
    if la == 0.0 || lb == 0.0 { return 0.0; }
    (la - lb).abs() / (la + lb)
}

fn exhaustive_path(
    adapter: &ProximityAdapter, root: FactId, horizon: usize,
    beta: f32, gamma: f32, c_sem: f32, m_sem: f32,
) -> Option<(f32, Vec<FactId>)> {
    let mut best_cost = f32::INFINITY;
    let mut best_path: Vec<FactId> = Vec::new();
    let root_state = adapter.state_of(root);
    let mut stack: Vec<(FactId, KinematicState, f32, usize, HashSet<FactId>, Vec<FactId>)> = Vec::new();
    let mut initial_visited = HashSet::new();
    initial_visited.insert(root);
    stack.push((root, root_state, 0.0, 0, initial_visited, vec![root]));
    while let Some((node, state, cost, depth, visited, path)) = stack.pop() {
        if depth >= horizon {
            if cost < best_cost { best_cost = cost; best_path = path; }
            continue;
        }
        let neighbors = adapter.neighbors(node);
        if neighbors.is_empty() {
            if cost < best_cost { best_cost = cost; best_path = path; }
            continue;
        }
        for (next_node, next_state) in neighbors {
            let step_cost = state.inertial_action(&next_state, beta, gamma, c_sem, m_sem);
            let new_cost = cost + step_cost;
            if new_cost >= best_cost { continue; }
            if !visited.contains(&next_node) {
                let mut nv = visited.clone(); nv.insert(next_node);
                let mut np = path.clone(); np.push(next_node);
                stack.push((next_node, next_state.clone(), new_cost, depth+1, nv, np));
            }
        }
    }
    if best_cost.is_finite() { Some((best_cost, best_path)) } else { None }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let target = args.get(1).cloned().unwrap_or_else(|| "pair_196".to_string());
    let horizon: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);

    let index_file = File::open(&index_path())?;
    let index: DatasetIndex = serde_json::from_reader(BufReader::new(index_file))?;
    let npz_file = File::open(&cache_path())?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    let mut fatti: Vec<Fatto> = Vec::new();
    let mut pair_keys: Vec<&String> = index.keys().collect();
    pair_keys.sort();
    for (pair_idx, pair_id) in pair_keys.iter().enumerate() {
        let seq_a = read_dense(&mut zip, &format!("{}_a_dense", pair_id))?;
        let seq_b = read_dense(&mut zip, &format!("{}_b_dense", pair_id))?;
        let ids_a: Vec<i64> = read_npz_array(&mut zip, &format!("{}_a_ids", pair_id))?;
        let weights_a: Vec<f32> = read_npz_array(&mut zip, &format!("{}_a_weights", pair_id))?;
        let ids_b: Vec<i64> = read_npz_array(&mut zip, &format!("{}_b_ids", pair_id))?;
        let weights_b: Vec<f32> = read_npz_array(&mut zip, &format!("{}_b_weights", pair_id))?;
        let sparse_a = build_sparse(&ids_a, &weights_a)?;
        let sparse_b = build_sparse(&ids_b, &weights_b)?;
        fatti.push(Fatto { id: NodeId((pair_idx*2) as u64), colbert: auto_dense(&seq_a), sparse: auto_sparse(&sparse_a), relazionale: relazionale_a_b(&seq_a, &seq_b) });
        fatti.push(Fatto { id: NodeId((pair_idx*2+1) as u64), colbert: auto_dense(&seq_b), sparse: auto_sparse(&sparse_b), relazionale: relazionale_a_b(&seq_a, &seq_b) });
    }

    let config = GraphConfig { k: 5, soglia: 0.5, percentile_cutoff: 0.0, pesi: [0.6, 0.25, 0.15] };
    let grafo = costruisci(&fatti, config);
    let adapter = ProximityAdapter::new(grafo.clone());
    let (beta, gamma, c_sem, m_sem) = (1.0, 1.0, 100.0, 1.0);
    let min_step = measure_min_step(&grafo, &adapter, beta, gamma);
    let solver_config = SolverConfig { horizon, kappa: 2.0, min_step, ..Default::default() };
    let solver = BeamSolver::new(adapter.clone(), solver_config);

    let idx = pair_keys.iter().position(|k| *k == &target);
    let idx = match idx {
        Some(i) => i,
        None => { eprintln!("pair_id '{}' non trovato", target); std::process::exit(2); }
    };
    let root = (idx * 2) as FactId;
    let cat = &index[&target].category;
    println!("target={} idx={} root={} categoria={} H={} min_step={:.7}", target, idx, root, cat, horizon, min_step);

    // Oracolo con cammino
    if let Some((ocost, opath)) = exhaustive_path(&adapter, root, horizon, beta, gamma, c_sem, m_sem) {
        println!("\n=== ORACOLO: costo={:.4} cammino={:?} ===", ocost, opath);
        let mut state = adapter.state_of(root);
        let mut running = 0.0;
        for w in opath.windows(2) {
            let (a, b) = (w[0], w[1]);
            let st = adapter.neighbors(a).into_iter().find(|(n, _)| *n == b).map(|(_, s)| s).unwrap();
            let step = state.inertial_action(&st, beta, gamma, c_sem, m_sem);
            running += step;
            println!("  step {}->{}: cost={:.4} (cum={:.4})", a, b, step, running);
            state = st;
        }
    } else {
        println!("oracolo: nessun cammino");
    }

    // Beam: tutti i cammini
    let paths = solver.solve(root, adapter.state_of(root));
    println!("\n=== BEAM: {} cammini ===", paths.len());
    let mut sorted: Vec<_> = paths.iter().collect();
    sorted.sort_by(|a, b| a.total_cost.partial_cmp(&b.total_cost).unwrap_or(std::cmp::Ordering::Equal));
    for p in &sorted {
        println!("  cost={:.4} path={:?}", p.total_cost, p.nodes);
    }

    Ok(())
}

/// Misura il minimo incremento inerziale positivo sugli archi reali del grafo.
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
            if from == to { continue; }
            let a = from.inertial_action(&to, beta, gamma, 100.0, 1.0);
            if a > 0.0 && a < min { min = a; }
        }
    }
    if min.is_finite() { min } else { 1e-6 }
}
