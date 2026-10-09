//! # probe_adapter — le sonde semantica come strategie esplicite
//!
//! La **Via B** del contratto ProbeAdapter (09/10), fedele alla proposta di
//! Camillo (08/10): entrambe le sonde integrate come **strategie esplicite**
//! in un unico tipo.
//!
//! ```text
//! ProbeAdapter::Curvature   (FrameAdapter — geometria locale del frame)
//! ProbeAdapter::Relational  (ProximityAdapter — struttura relazionale)
//! ```
//!
//! A differenza della Via A (trait `GraphAdapter` con `probe_kind()`, in
//! `solver.rs`), qui la sonda è un **selettore di strategia** a runtime: lo
//! stesso codice sceglie quale delle due dimensioni misurare. La
//! complementarità tra le due sonde (lezione del collasso FrameAdapter,
//! 08/10) resta dichiarata dal tipo, ma la selezione è esplicita e
//! componibile — si può costruire un `ProbeAdapter` per ogni strategia e
//! confrontarne i cammini a runtime, senza cambiare il trait consumato dal
//! solver.
//!
//! Questo modulo è la controparte esplorativa della Via A: due strade
//! percorse fino in fondo, con i loro risultati, da confrontare quando
//! Camillo rientra.

use std::collections::HashMap;

use crate::graph_adapter::{FrameAdapter, ProximityAdapter};
use crate::solver::{GraphAdapter, ProbeKind};
use crate::KinematicState;
use semantic_combiner::FactId;
use semantic_graph::Graph;

/// Le sonde semantiche come strategie esplicite.
///
/// Incapsula le due convenzioni complementari — la geometria locale del
/// frame (`Curvature`) e la struttura relazionale del grafo (`Relational`) —
/// dietro un unico selettore di strategia.
#[derive(Debug, Clone)]
pub enum ProbeAdapter {
    /// Sonda di curvatura: `FrameAdapter` (convenzione canonica
    /// `derive_state` del 06/10).
    Curvature(FrameAdapter),
    /// Sonda relazionale: `ProximityAdapter` (media delle sonde degli archi).
    Relational(ProximityAdapter),
}

impl ProbeAdapter {
    /// Costruisce una sonda di curvatura dal grafo e dalle traiettorie dense.
    pub fn curvature(grafo: Graph, traiettorie: HashMap<FactId, Vec<Vec<f64>>>) -> Self {
        ProbeAdapter::Curvature(FrameAdapter::new(grafo, traiettorie))
    }

    /// Costruisce una sonda relazionale dal grafo.
    pub fn relational(grafo: Graph) -> Self {
        ProbeAdapter::Relational(ProximityAdapter::new(grafo))
    }

    /// La natura della sonda selezionata.
    pub fn kind(&self) -> ProbeKind {
        match self {
            ProbeAdapter::Curvature(_) => ProbeKind::Curvature,
            ProbeAdapter::Relational(_) => ProbeKind::Relational,
        }
    }
}

impl GraphAdapter for ProbeAdapter {
    fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)> {
        match self {
            ProbeAdapter::Curvature(a) => a.neighbors(node),
            ProbeAdapter::Relational(a) => a.neighbors(node),
        }
    }

    fn probe_kind(&self) -> ProbeKind {
        self.kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_graph::{Edge, GraphConfig, NodeId};

    fn arco(from: u64, to: u64, score: f64, dense: f64, sparse: f64, colbert: f64) -> Edge {
        Edge {
            from: NodeId(from),
            to: NodeId(to),
            score,
            dense,
            sparse,
            colbert,
        }
    }

    #[test]
    fn kind_dichiara_la_sonda() {
        let mut g = Graph::new(GraphConfig::default());
        g.nodi = vec![NodeId(1), NodeId(2)];
        g.archi = vec![arco(1, 2, 0.5, 0.5, 0.5, 0.5)];

        let cur = ProbeAdapter::curvature(g.clone(), HashMap::new());
        assert_eq!(cur.kind(), ProbeKind::Curvature);
        assert_eq!(cur.probe_kind(), ProbeKind::Curvature);

        let rel = ProbeAdapter::relational(g);
        assert_eq!(rel.kind(), ProbeKind::Relational);
        assert_eq!(rel.probe_kind(), ProbeKind::Relational);
    }

    #[test]
    fn entrambe_le_strategie_espongono_vicini() {
        let mut g = Graph::new(GraphConfig::default());
        g.nodi = vec![NodeId(1), NodeId(2), NodeId(3)];
        g.archi = vec![
            arco(1, 2, 0.8, 0.7, 0.6, 0.5),
            arco(1, 3, 0.4, 0.3, 0.2, 0.1),
        ];

        // Relational: stati dalla media delle sonde degli archi.
        let rel = ProbeAdapter::relational(g.clone());
        let mut vicini = rel.neighbors(1);
        vicini.sort_by_key(|(id, _)| *id);
        assert_eq!(vicini.len(), 2);
        assert_eq!(vicini[0].0, 2);
        assert!((vicini[0].1.velocity - 0.8).abs() < 1e-6);

        // Curvature: senza traiettorie, tutti gli stati sono default (zero),
        // ma l'adiacenza è preservata.
        let cur = ProbeAdapter::curvature(g, HashMap::new());
        let mut vicini = cur.neighbors(1);
        vicini.sort_by_key(|(id, _)| *id);
        assert_eq!(vicini.len(), 2);
        assert_eq!(vicini[0].0, 2);
        assert_eq!(vicini[0].1.velocity, 0.0);
    }
}
