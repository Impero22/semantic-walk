//! # costruisci — la costruzione del grafo di prossimità
//!
//! Strategia **ibrida**: k-nearest come base (struttura), soglia come filtro
//! di qualità. Ogni fatto si collega ai suoi k vicini più simili, ma solo se
//! il punteggio supera la soglia minima.

use crate::{Edge, Graph, GraphConfig, NodeId};
use semantic_combiner::{combine, NormalizedAxes};

/// La descrizione di un fatto della memoria: l'id e le sue sonde.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fatto {
    pub id: NodeId,
    pub dense: f64,
    pub sparse: f64,
    pub colbert: f64,
}

impl Fatto {
    /// Il punteggio combinato con un altro fatto, dati i pesi.
    pub fn punteggio(&self, altro: &Fatto, pesi: [f64; 3]) -> f64 {
        let axes = NormalizedAxes {
            dense: self.dense * altro.dense,
            sparse: self.sparse * altro.sparse,
            colbert: self.colbert * altro.colbert,
        };
        combine(&axes, pesi)
    }
}

/// Costruisce il grafo di prossimità da una lista di fatti.
pub fn costruisci(fatti: &[Fatto], config: GraphConfig) -> Graph {
    let mut grafo = Graph::new(config);

    if fatti.is_empty() {
        return grafo;
    }

    let mut fatti: Vec<Fatto> = fatti.to_vec();
    fatti.sort_by_key(|f| f.id);
    fatti.dedup_by_key(|f| f.id);

    let mut nodi: Vec<NodeId> = fatti.iter().map(|f| f.id).collect();
    nodi.sort_unstable();
    nodi.dedup();
    grafo.nodi = nodi;

    let k = config.k.max(1);
    let soglia = config.soglia.clamp(0.0, 1.0);
    let pesi = config.pesi;

    let mut archi_vec: Vec<(u64, u64, f64, f64, f64, f64)> = Vec::new();

    for (i, fatto) in fatti.iter().enumerate() {
        let mut candidati: Vec<(f64, NodeId)> = fatti
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, altro)| (fatto.punteggio(altro, pesi), altro.id))
            .collect();

        candidati.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
        });
        candidati.truncate(k);

        for (score, altro_id) in candidati {
            if score >= soglia {
                let (from, to) = if fatto.id <= altro_id {
                    (fatto.id.0, altro_id.0)
                } else {
                    (altro_id.0, fatto.id.0)
                };
                if from == to {
                    continue;
                }
                let altro = fatti.iter().find(|f| f.id == altro_id).unwrap();
                archi_vec.push((
                    from,
                    to,
                    score,
                    fatto.dense * altro.dense,
                    fatto.sparse * altro.sparse,
                    fatto.colbert * altro.colbert,
                ));
            }
        }
    }

    archi_vec.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    archi_vec.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

    grafo.archi = archi_vec
        .into_iter()
        .map(|(from, to, score, dense, sparse, colbert)| Edge {
            from: NodeId(from),
            to: NodeId(to),
            score,
            dense,
            sparse,
            colbert,
        })
        .collect();

    grafo
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metriche::grado;

    fn fatto(id: u64, dense: f64, sparse: f64, colbert: f64) -> Fatto {
        Fatto {
            id: NodeId(id),
            dense,
            sparse,
            colbert,
        }
    }

    #[test]
    fn costruzione_vuota() {
        let g = costruisci(&[], GraphConfig::default());
        assert_eq!(g.n_nodi(), 0);
        assert_eq!(g.n_archi(), 0);
    }

    #[test]
    fn nodi_unici_e_ordinati() {
        let fatti = vec![
            fatto(3, 0.9, 0.8, 0.7),
            fatto(1, 0.8, 0.7, 0.6),
            fatto(3, 0.9, 0.8, 0.7),
        ];
        let g = costruisci(&fatti, GraphConfig::default());
        assert_eq!(g.n_nodi(), 2);
        assert_eq!(g.nodi, vec![NodeId(1), NodeId(3)]);
        assert!(
            g.archi.iter().all(|e| e.from != e.to),
            "nessun self-loop (from == to)"
        );
        assert_eq!(grado(&g, NodeId(3)), 1);
    }

    #[test]
    fn simmetria_degli_archi() {
        let fatti = vec![fatto(1, 1.0, 1.0, 1.0), fatto(2, 1.0, 1.0, 1.0)];
        let g = costruisci(&fatti, GraphConfig::default());
        assert!(g.ha_arco(NodeId(1), NodeId(2)));
        assert!(g.ha_arco(NodeId(2), NodeId(1)));
        assert_eq!(g.n_archi(), 1);
    }

    #[test]
    fn soglia_filtra_archi_deboli() {
        let fatti = vec![fatto(1, 0.1, 0.1, 0.1), fatto(2, 0.9, 0.9, 0.9)];
        let config = GraphConfig {
            soglia: 0.5,
            ..GraphConfig::default()
        };
        let g = costruisci(&fatti, config);
        assert_eq!(g.n_archi(), 0);
    }

    #[test]
    fn k_limita_i_vicini() {
        let fatti = vec![
            fatto(1, 1.0, 1.0, 1.0),
            fatto(2, 0.9, 0.9, 0.9),
            fatto(3, 0.1, 0.1, 0.1),
            fatto(4, 0.2, 0.2, 0.2),
        ];
        let config = GraphConfig {
            k: 1,
            ..GraphConfig::default()
        };
        let g = costruisci(&fatti, config);
        assert!(g.ha_arco(NodeId(1), NodeId(2)));
        assert!(g.ha_arco(NodeId(3), NodeId(1)));
        assert!(g.ha_arco(NodeId(4), NodeId(1)));
        assert!(!g.ha_arco(NodeId(3), NodeId(4)));
        assert_eq!(g.n_archi(), 3);
    }
}
