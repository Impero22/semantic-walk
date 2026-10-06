//! Misura il vero minimo inerziale sugli archi del Dataset B con derive_state
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_graph::{Edge, Graph, GraphConfig, NodeId};
use semantic_walk::state::{cosine, derive_state};
use semantic_walk::KinematicState;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_b")
}
fn cache_path() -> PathBuf {
    data_dir().join("dataset_b_cache_fixed.npz")
}
const PROX_THRESHOLD: f32 = 0.7;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct IndexEntry { category: String, text_a: String, text_b: String }
type DatasetIndex = HashMap<String, IndexEntry>;

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
        rows.push(flat[i * dim..(i + 1) * dim].iter().map(|&x| x as f64).collect());
    }
    Ok(rows)
}

fn costruisci_grafo(trajectories: &[Vec<Vec<f64>>]) -> Graph {
    let mut nodi: Vec<NodeId> = Vec::new();
    let mut archi: Vec<Edge> = Vec::new();
    let mut base = 0usize;
    for traj in trajectories {
        for _ in traj { nodi.push(NodeId(base as u64)); base += 1; }
    }
    let mut visti: HashSet<(u64, u64)> = HashSet::new();
    let push_arco = |archi: &mut Vec<Edge>, visti: &mut HashSet<(u64,u64)>, from: usize, to: usize, sim: f32| {
        if visti.insert((from as u64, to as u64)) {
            archi.push(Edge { from: NodeId(from as u64), to: NodeId(to as u64), score: sim as f64, dense: sim as f64, sparse: 0.0, colbert: 0.0 });
        }
    };
    let mut base2 = 0usize;
    for traj in trajectories {
        let n = traj.len();
        for i in 0..n {
            let from = base2 + i;
            for j in (i + 1)..(i + 3).min(n) {
                let to = base2 + j;
                let sim = cosine(&traj[i], &traj[j]);
                push_arco(&mut archi, &mut visti, from, to, sim);
            }
        }
        base2 += n;
    }
    let mut base3 = 0usize;
    for traj in trajectories {
        let n = traj.len();
        for i in 0..n {
            let from = base3 + i;
            for j in (i + 1)..n {
                let to = base3 + j;
                let sim = cosine(&traj[i], &traj[j]);
                if sim >= PROX_THRESHOLD {
                    push_arco(&mut archi, &mut visti, from, to, sim);
                }
            }
        }
        base3 += n;
    }
    Graph { config: GraphConfig::default(), nodi, archi }
}

fn main() {
    let index_file = File::open(data_dir().join("dataset_b_index.json")).unwrap();
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader).unwrap();

    let npz_file = File::open(&cache_path()).unwrap();
    let mut zip = ZipArchive::new(BufReader::new(npz_file)).unwrap();

    let mut trajectories: Vec<Vec<Vec<f64>>> = Vec::new();
    for (pair_id, _entry) in &index {
        let seq_a = read_dense(&mut zip, &format!("{}_a_dense", pair_id)).unwrap();
        let seq_b = read_dense(&mut zip, &format!("{}_b_dense", pair_id)).unwrap();
        if !seq_a.is_empty() { trajectories.push(seq_a); }
        if !seq_b.is_empty() { trajectories.push(seq_b); }
    }

    let grafo = costruisci_grafo(&trajectories);

    let mut stati: HashMap<NodeId, KinematicState> = HashMap::new();
    let mut base = 0usize;
    for traj in &trajectories {
        for (i, frame) in traj.iter().enumerate() {
            stati.insert(NodeId((base + i) as u64), derive_state(traj, i, frame));
        }
        base += traj.len();
    }

    let mut min_action = f32::MAX;
    let mut min_pair = None;
    let mut count_below_00049 = 0;
    let mut count_below_0002 = 0;
    let mut count_zero = 0;
    let mut total_distinct = 0;
    for e in &grafo.archi {
        let s_from = stati.get(&e.from).copied().unwrap_or_default();
        let s_to = stati.get(&e.to).copied().unwrap_or_default();
        if s_from == s_to { continue; }
        total_distinct += 1;
        let a = s_from.inertial_action(&s_to, 1.0, 1.0, 100.0, 1.0);
        if a < min_action { min_action = a; min_pair = Some((e.from.0, e.to.0, a)); }
        if a < 0.00049 { count_below_00049 += 1; }
        if a < 0.002 { count_below_0002 += 1; }
        if a == 0.0 { count_zero += 1; }
    }
    println!("Nodi: {}, Archi direzionati: {}, Archi stati distinti: {}", grafo.nodi.len(), grafo.archi.len(), total_distinct);
    println!("Minimo inerziale (derive_state, u!=v): {:.10}", min_action);
    if let Some((f, t, a)) = min_pair { println!("  arco {} -> {}: azione {:.10}", f, t, a); }
    println!("Archi con azione < 0.00049 (soglia debug_assert test): {}", count_below_00049);
    println!("Archi con azione < 0.002 (default solver): {}", count_below_0002);
    println!("Archi con azione == 0.0: {}", count_zero);
}
