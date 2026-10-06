//! # graph_adapter — il ponte dal grafo di prossimità al beam
//!
//! L'adattatore che chiude l'anello tra `semantic-graph` (il grafo di
//! prossimità costruito dai fatti della memoria) e il `BeamSolver` di
//! `semantic-walk` (la ricerca a orizzonte con ampiezza dinamica).
//!
//! ## ⚠️ La convenzione dello stato intrinseco (aggiornata 06/10)
//!
//! **Decisione di contratto con Camillo (06/10): `derive_state` (geometria
//! locale del frame) è la convenzione CANONICA** per costruire il
//! `KinematicState` che alimenta il beam. La convenzione è ora eseguibile
//! dal codice di produzione: vive in [`crate::state::derive_state`], non più
//! solo nei test e nei benchmark.
//!
//! La **media delle sonde degli archi** (la convenzione di questo modulo,
//! concordata il 03/10) è **declassata a strategia secondaria/opzionale**.
//! Motivo: la media appiattisce la varianza tra i nodi (tutti gli stati
//! finiscono quasi identici), degradando la capacità del beam di distinguere
//! le traiettorie — il problema già emerso con la calibrazione di kappa. La
//! geometria locale del frame (`derive_state`) cattura la dinamica reale del
//! vettore (norma, accelerazione come delta di norma, curvatura come
//! deviazione angolare) senza spalmarla sugli archi adiacenti.
//!
//! Questo modulo mantiene `ProximityAdapter` (media sonde) come **strategia
//! secondaria** per compatibilità con i benchmark esistenti
//! (`bench_proximity_adapter_end2end`, `diag_*`) e per i casi in cui lo
//! stato per-frame non è disponibile (es. nodi aggregati senza geometria
//! locale). Il contratto canonico per il beam è `derive_state`.
//!
//! ## La convenzione (storica, 03/10 — ora secondaria)
//!
//! Nel `ProximityGraph` le sonde (dense/sparse/colbert) vivono sugli **archi**,
//! non sui nodi. Il `KinematicState` del solver è invece **assoluto al nodo**:
//! deve essere una proprietà intrinseca, stabile e indipendente dal cammino di
//! provenienza — altrimenti lo stato di un nodo cambierebbe a seconda che ci si
//! arrivi da A o da C, rompendo la semantica di `extend`.
//!
//! La media delle sonde dei suoi archi:
//!
//! * `velocity` — la media degli `score` → la forza complessiva del legame del
//!   nodo col grafo.
//! * `acceleration` — la media dei `dense` → la componente densa media.
//! * `curvature` — la media di `(sparse + colbert)/2` → la curvatura media.
//!
//! Così camminare A→B→C produce un'azione inerziale
//! `α·Δv² + β·Δa² + γ·|Δκ|` basata su come cambia il *profilo medio* del nodo
//! lungo il cammino — la "rigidità della camminata" che il solver vuole
//! misurare — coerente per qualsiasi cammino che attraversi il nodo.
//!
//! ## Perché lo stato è precalcolato
//!
//! Lo stato intrinseco di ogni nodo è calcolato **una sola volta** in
//! `GraphAdapter::new` (per direttiva di Camillo: evitare la ricalcolatura
//! delle medie a ogni chiamata di `neighbors()`). La mappa `HashMap<NodeId,
//! KinematicState>` è la cache di questa inizializzazione.

use std::collections::HashMap;

use crate::solver::GraphAdapter;
use crate::KinematicState;
use semantic_combiner::FactId;
use semantic_graph::{Edge, Graph, NodeId};

/// L'adattatore dal grafo di prossimità al beam.
///
/// Precalcola lo stato intrinseco di ogni nodo (media delle sonde dei suoi
/// archi) e lo restituisce insieme ai vicini.
#[derive(Debug, Clone)]
pub struct ProximityAdapter {
    grafo: Graph,
    /// La cache degli stati intrinseci, indicizzata per `NodeId`.
    stati: HashMap<NodeId, KinematicState>,
}

impl ProximityAdapter {
    /// Costruisce l'adattatore dal grafo di prossimità.
    ///
    /// Precalcola lo stato intrinseco di ogni nodo presente nel grafo come
    /// media delle sonde dei suoi archi. I nodi senza archi (isolati) non
    /// vengono inseriti nella cache: `neighbors` restituirà comunque i vicini
    /// reali, e per lo stato userà il `Default` se mancante.
    pub fn new(grafo: Graph) -> Self {
        let stati = grafo
            .nodi
            .iter()
            .filter_map(|&n| Self::stato_intrinseco(&grafo, n).map(|s| (n, s)))
            .collect();
        ProximityAdapter { grafo, stati }
    }

    /// Lo stato intrinseco di un nodo: la media delle sonde dei suoi archi.
    ///
    /// Restituisce `None` se il nodo non ha archi (stato non definibile).
    fn stato_intrinseco(grafo: &Graph, n: NodeId) -> Option<KinematicState> {
        let archi: Vec<Edge> = grafo
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

    /// Lo stato intrinseco di un nodo (la media delle sonde dei suoi archi).
    ///
    /// Restituisce `KinematicState::default()` se il nodo non ha archi
    /// (stato non definibile) o non è nella cache.
    pub fn state_of(&self, node: FactId) -> KinematicState {
        self.stati.get(&NodeId(node)).copied().unwrap_or_default()
    }
}

impl GraphAdapter for ProximityAdapter {
    fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)> {
        let n = NodeId(node);
        self.grafo
            .vicini(n)
            .into_iter()
            .map(|vicino| {
                let stato = self
                    .stati
                    .get(&vicino)
                    .copied()
                    .unwrap_or_default();
                (vicino.0, stato)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_graph::{Edge, Graph, GraphConfig, NodeId};

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
    fn stato_intrinseco_media_delle_sonde() {
        // Nodo 1 collegato a 2 e 3 con sonde note.
        let mut g = Graph::new(GraphConfig::default());
        g.nodi = vec![NodeId(1), NodeId(2), NodeId(3)];
        g.archi = vec![
            arco(1, 2, 0.8, 0.7, 0.6, 0.5),
            arco(1, 3, 0.4, 0.3, 0.2, 0.1),
        ];
        let adapter = ProximityAdapter::new(g);
        let stato = adapter.stati.get(&NodeId(1)).unwrap();
        // velocity = media score = (0.8+0.4)/2 = 0.6
        assert!((stato.velocity - 0.6).abs() < 1e-6);
        // acceleration = media dense = (0.7+0.3)/2 = 0.5
        assert!((stato.acceleration - 0.5).abs() < 1e-6);
        // curvature = media (sparse+colbert)/2 = ((0.6+0.5)/2 + (0.2+0.1)/2)/2 = (0.55+0.15)/2 = 0.35
        assert!((stato.curvature - 0.35).abs() < 1e-6);
    }

    #[test]
    fn nodo_isolato_senza_stato() {
        let mut g = Graph::new(GraphConfig::default());
        g.nodi = vec![NodeId(1), NodeId(2)];
        g.archi = vec![arco(1, 2, 0.5, 0.5, 0.5, 0.5)];
        let adapter = ProximityAdapter::new(g);
        // Il nodo 1 e 2 hanno archi, il nodo 3 non esiste.
        assert!(adapter.stati.contains_key(&NodeId(1)));
        assert!(adapter.stati.contains_key(&NodeId(2)));
        // neighbors di un nodo senza vicini restituisce vuoto.
        assert!(adapter.neighbors(99).is_empty());
    }

    #[test]
    fn neighbors_restituisce_vicini_con_stato() {
        let mut g = Graph::new(GraphConfig::default());
        g.nodi = vec![NodeId(1), NodeId(2), NodeId(3)];
        g.archi = vec![
            arco(1, 2, 0.8, 0.7, 0.6, 0.5),
            arco(1, 3, 0.4, 0.3, 0.2, 0.1),
        ];
        let adapter = ProximityAdapter::new(g);
        let mut vicini = adapter.neighbors(1);
        vicini.sort_by_key(|(id, _)| *id);
        assert_eq!(vicini.len(), 2);
        assert_eq!(vicini[0].0, 2);
        assert_eq!(vicini[1].0, 3);
        // Lo stato del vicino 2: media delle sonde dei suoi archi (solo 1—2).
        assert!((vicini[0].1.velocity - 0.8).abs() < 1e-6);
        assert!((vicini[0].1.acceleration - 0.7).abs() < 1e-6);
        assert!((vicini[0].1.curvature - 0.55).abs() < 1e-6);
    }
}