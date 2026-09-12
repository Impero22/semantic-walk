use semantic_graph::costruisci::{costruisci, Fatto};
use semantic_graph::metriche::grado;
use semantic_graph::{GraphConfig, NodeId};

#[test]
fn selfloop_con_id_duplicati() {
    let fatti = vec![
        Fatto { id: NodeId(3), dense: 0.9, sparse: 0.8, colbert: 0.7 },
        Fatto { id: NodeId(1), dense: 0.8, sparse: 0.7, colbert: 0.6 },
        Fatto { id: NodeId(3), dense: 0.9, sparse: 0.8, colbert: 0.7 }, // duplicato
    ];
    let g = costruisci(&fatti, GraphConfig::default());
    let self_loops: Vec<_> = g.archi.iter().filter(|e| e.from == e.to).collect();
    eprintln!("nodi: {:?}", g.nodi);
    eprintln!("n_archi: {}", g.n_archi());
    for e in &g.archi {
        eprintln!("arco: {} -- {} (score {:.4})", e.from.0, e.to.0, e.score);
    }
    eprintln!("self-loop: {}", self_loops.len());
    eprintln!("grado nodo 3: {}", grado(&g, NodeId(3)));
    assert!(self_loops.is_empty(), "self-loop presente: {}", self_loops.len());
}
