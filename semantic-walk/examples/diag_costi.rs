//! # diag_costi — distribuzione reale dei costi e delle ampiezze
//!
//! Carica il Dataset B reale, costruisce il grafo, e misura la distribuzione
//! delle azioni inerziali sugli archi e dei costi cumulati dei cammini.
//! Serve a capire perché la potatura F6 non discrimina.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_walk::solver::GraphAdapter;
use semantic_graph::{costruisci, Fatto, GraphConfig, NodeId};
use semantic_walk::graph_adapter::ProximityAdapter;
use semantic_walk::ordered_sparse::OrderedSparseSequence;
use semantic_walk::solver::{BeamSolver, SolverConfig};

const SIGNATURE_BITS: f64 = 128.0;

fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_b")
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct IndexEntry {
    text_a: String,
    text_b: String,
    category: String,
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
        let row = flat[i * dim..(i + 1) * dim].iter().map(|&x| x as f64).collect::<Vec<f64>>();
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
    let index_path = data_dir().join("dataset_b_index.json");
    let f = File::open(&index_path)?;
    let r = BufReader::new(f);
    let _index: HashMap<String, IndexEntry> = serde_json::from_reader(r)?;

    let cache = data_dir().join("dataset_b_cache_fixed.npz");
    let mut zip = ZipArchive::new(BufReader::new(File::open(&cache)?))?;

    // Estrai i nomi per trovare le coppie.
    let mut pair_ids: Vec<String> = Vec::new();
    for i in 0..zip.len() {
        let name = zip.by_index(i)?.name().to_string();
        if name.ends_with("_a_dense.npy") {
            pair_ids.push(name.trim_end_matches("_a_dense.npy").to_string());
        }
    }
    pair_ids.sort();
    println!("Coppie trovate: {}", pair_ids.len());

    let mut fatti: Vec<Fatto> = Vec::new();
    for (pair_idx, pair_id) in pair_ids.iter().enumerate() {
        let seq_a = read_dense(&mut zip, &format!("{}_a_dense", pair_id))?;
        let seq_b = read_dense(&mut zip, &format!("{}_b_dense", pair_id))?;

        // sparse: ids e weights
        let ids_a: Vec<i64> = {
            let fname = npyz::npz::file_name_from_array_name(&format!("{}_a_ids", pair_id));
            let file = zip.by_name(&fname)?;
            NpyFile::new(file)?.into_vec::<i64>()?
        };
        let w_a: Vec<f32> = {
            let fname = npyz::npz::file_name_from_array_name(&format!("{}_a_weights", pair_id));
            let file = zip.by_name(&fname)?;
            NpyFile::new(file)?.into_vec::<f32>()?
        };
        let ids_b: Vec<i64> = {
            let fname = npyz::npz::file_name_from_array_name(&format!("{}_b_ids", pair_id));
            let file = zip.by_name(&fname)?;
            NpyFile::new(file)?.into_vec::<i64>()?
        };
        let w_b: Vec<f32> = {
            let fname = npyz::npz::file_name_from_array_name(&format!("{}_b_weights", pair_id));
            let file = zip.by_name(&fname)?;
            NpyFile::new(file)?.into_vec::<f32>()?
        };

        let sparse_a = build_sparse(&ids_a, &w_a)?;
        let sparse_b = build_sparse(&ids_b, &w_b)?;

        let d_a = auto_dense(&seq_a);
        let s_a = auto_sparse(&sparse_a);
        let c_a = d_a;
        let d_b = auto_dense(&seq_b);
        let s_b = auto_sparse(&sparse_b);
        let c_b = d_b;

        fatti.push(Fatto { id: NodeId((pair_idx * 2) as u64), dense: d_a, sparse: s_a, colbert: c_a });
        fatti.push(Fatto { id: NodeId((pair_idx * 2 + 1) as u64), dense: d_b, sparse: s_b, colbert: c_b });
    }
    println!("Fatti costruiti: {}", fatti.len());

    let config = GraphConfig { k: 5, soglia: 0.5, percentile_cutoff: 0.0, pesi: [0.6, 0.25, 0.15] };
    let grafo = costruisci(&fatti, config);
    println!("Grafo: {} nodi, {} archi", grafo.n_nodi(), grafo.n_archi());

    // Misura la distribuzione delle azioni inerziali sugli archi.
    // Attraverso l'adapter, per ogni nodo prendiamo i vicini e calcoliamo
    // l'azione inerziale con lo stato del nodo (default) e del vicino.
    let adapter = ProximityAdapter::new(grafo.clone());
    let config_solver = SolverConfig { horizon: 4, kappa: 0.01, min_step: 0.0005, ..Default::default() };

    // Distribuzione dei ΔS sugli archi (usando l'adapter: stato reale del nodo
    // sorgente vs stato del vicino). NOTA: misurare la distanza dal default
    // (0,0,0) è un artefatto — i nodi reali hanno stati ~(0.97,1.0,0.59), quindi
    // la distanza dal default (~0.85) non è la vera azione tra nodi. Qui usiamo
    // inertial_action con lo stato reale di entrambi gli estremi dell'arco.
    let mut azioni: Vec<f32> = Vec::new();
    for nid in 0..grafo.n_nodi() {
        let _n = NodeId(nid as u64);
        let stato_n = adapter.state_of(nid as u64);
        for (v, st) in adapter.neighbors(nid as u64) {
            let _ = v;
            let a = stato_n.inertial_action(
                &st,
                config_solver.beta,
                config_solver.gamma,
                config_solver.c_sem,
                config_solver.m_sem,
            );
            azioni.push(a);
        }
    }
    azioni.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = azioni.len();
    if n > 0 {
        println!("\nDistribuzione azioni sugli archi (n={}):", n);
        println!("  min={:.6} p50={:.6} p90={:.6} p99={:.6} max={:.6}",
            azioni[0], azioni[n/2], azioni[(n*9)/10], azioni[(n*99)/100], azioni[n-1]);
    }

    // Distribuzione dei costi cumulati dei cammini reali.
    // Il root parte dallo stato intrinseco reale del nodo (media delle sonde
    // dei suoi archi), NON dal default: partire dal default (0,0,0) quando i
    // nodi reali hanno stati ~(0.97, 1.0, 0.59) renderebbe il primo passo un
    // artefatto enorme (~2.06) che domina il costo cumulato e schiaccia le
    // ampiezze, rendendo il pruning F6 cieco.
    let root_state = adapter.state_of(0u64);
    let solver = BeamSolver::new(adapter, config_solver);
    let t = Instant::now();
    let paths = solver.solve(0u64, root_state);
    println!("\nBeam su root=0: {} cammini in {:?}", paths.len(), t.elapsed());

    let mut costs: Vec<f32> = paths.iter().map(|p| p.total_cost).collect();
    costs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = costs.len();
    if n > 0 {
        println!("Costi cammini (n={}):", n);
        println!("  min={:.6} p50={:.6} p90={:.6} p99={:.6} max={:.6}",
            costs[0], costs[n/2], costs[(n*9)/10], costs[(n*99)/100], costs[n-1]);
        let mut amps: Vec<f32> = paths.iter().map(|p| p.amplitude).collect();
        amps.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("Ampiezze cammini:");
        println!("  min={:.6} p50={:.6} p90={:.6} p99={:.6} max={:.6}",
            amps[0], amps[n/2], amps[(n*9)/10], amps[(n*99)/100], amps[n-1]);
    }
    Ok(())
}
