//! # diag_prune — diagnostica della potatura per livello
//!
//! Esegue il beam su una radice reale e stampa, livello per livello:
//! quanti nodi entrano, quanti sopravvivono alla potatura F6, e il rapporto.

use std::time::Instant;

use semantic_graph::{costruisci, Fatto, GraphConfig, NodeId};
use semantic_walk::graph_adapter::ProximityAdapter;
use semantic_walk::solver::{BeamSolver, SolverConfig};
use semantic_walk::KinematicState;

fn main() {
    // Costruiamo fatti sintetici deterministici con struttura a cluster.
    let mut fatti: Vec<Fatto> = Vec::new();
    for i in 0..200usize {
        let d = ((i % 7) as f64) / 6.0;          // dense in [0,1]
        let s = ((i % 5) as f64) / 4.0;          // sparse in [0,1]
        let c = ((i % 3) as f64) / 2.0;          // colbert in [0,1]
        fatti.push(Fatto { id: NodeId((i*2) as u64), dense: d, sparse: s, colbert: c });
        fatti.push(Fatto { id: NodeId((i*2+1) as u64), dense: s, sparse: c, colbert: d });
    }

    let config = GraphConfig { k: 5, soglia: 0.5, percentile_cutoff: 0.0, pesi: [0.6, 0.25, 0.15] };
    let grafo = costruisci(&fatti, config);
    println!("Grafo: {} nodi, {} archi", grafo.n_nodi(), grafo.n_archi());

    let adapter = ProximityAdapter::new(grafo.clone());
    let solver = BeamSolver::new(adapter, SolverConfig {
        horizon: 4, kappa: 0.05, min_step: 0.01, ..Default::default()
    });

    let root = 0u64;
    let t = Instant::now();
    let paths = solver.solve(root, KinematicState::default());
    println!("Cammini totali: {} in {:?}", paths.len(), t.elapsed());

    // Distribuzione dei costi cumulati dei cammini.
    let mut costs: Vec<f32> = paths.iter().map(|p| p.total_cost).collect();
    costs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = costs.len();
    if n > 0 {
        println!("Costi cammini: min={:.6} p50={:.6} p90={:.6} p99={:.6} max={:.6}",
            costs[0], costs[n/2], costs[(n*9)/10], costs[(n*99)/100], costs[n-1]);
    }
}
