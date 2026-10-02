//! # costruisci — la costruzione del grafo di prossimità
//!
//! Strategia **ibrida**: k-nearest come base (struttura), soglia come filtro
//! di qualità. Ogni fatto si collega ai suoi k vicini più simili, ma solo se
//! il punteggio supera la soglia minima.

use std::collections::HashMap;

use crate::{Edge, Graph, GraphConfig, NodeId};
use semantic_combiner::{
    combine, pareto_compare, NormalizedAxes, ParetoOrder, LAMBDA_CALIBRATO,
};

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
        let axes = NormalizedAxes::normalize(
            self.dense * altro.dense,
            self.sparse * altro.sparse,
            self.colbert * altro.colbert,
            LAMBDA_CALIBRATO,
        );
        combine(&axes, pesi)
    }
}

/// Distanza euclidea pesata dal punto ideale (1.0, 1.0, 1.0) nello spazio
/// degli assi normalizzati.
///
/// Usata come criterio di spareggio deterministico nel tie-break dei
/// candidati kNN: a parità di score scalare e di non-dominanza sul fronte di
/// Pareto, vince il candidato la cui triade di sonde è globalmente più vicina
/// alla saturazione ottimale, pesata secondo i pesi del combiner. In questo
/// modo una deviazione su un canale pesante (es. sparse) penalizza più di una
/// su un canale leggero (es. dense), in coerenza con `PESI_CALIBRATI`.
fn distanza_ottimo_pesata(axes: &NormalizedAxes, pesi: [f64; 3]) -> f64 {
    let d = 1.0 - axes.dense;
    let s = 1.0 - axes.sparse;
    let c = 1.0 - axes.colbert;
    pesi[0] * d * d + pesi[1] * s * s + pesi[2] * c * c
}

/// Calcola il valore al percentile `p` (in `[0, 1]`) di una serie di punteggi.
///
/// Usa l'interpolazione lineare semplice (metodo di tipo "nearest-rank
/// interpola"): ordina i punteggi in modo crescente e interpola tra i due
/// valori che racchiudono l'indice `p * (n - 1)`. Con `p = 0.0` restituisce
/// il minimo, con `p = 1.0` il massimo.
///
/// Una serie vuota restituisce `0.0` (nessun candidato → nessun taglio).
fn calcola_percentile(scores: &[f64], percentile: f64) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let p = percentile.clamp(0.0, 1.0);
    let mut ordinati = scores.to_vec();
    ordinati.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let idx = p * (ordinati.len() - 1) as f64;
    let lower = idx.floor() as usize;
    let upper = idx.ceil() as usize;
    if lower == upper {
        ordinati[lower]
    } else {
        let weight = idx - lower as f64;
        ordinati[lower] * (1.0 - weight) + ordinati[upper] * weight
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
    let percentile_cutoff = config.percentile_cutoff.clamp(0.0, 1.0);
    let pesi = config.pesi;

    let mut archi_vec: Vec<(u64, u64, f64, f64, f64, f64)> = Vec::new();

    let indice_fatti: HashMap<NodeId, &Fatto> =
        fatti.iter().map(|f| (f.id, f)).collect();

    for (i, fatto) in fatti.iter().enumerate() {
        let mut candidati: Vec<(f64, NormalizedAxes, NodeId)> = fatti
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, altro)| {
                let axes = NormalizedAxes::normalize(
                    fatto.dense * altro.dense,
                    fatto.sparse * altro.sparse,
                    fatto.colbert * altro.colbert,
                    LAMBDA_CALIBRATO,
                );
                let score = combine(&axes, pesi);
                (score, axes, altro.id)
            })
            .collect();

        candidati.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| match pareto_compare(&a.1, &b.1) {
                    ParetoOrder::ADominatesB => std::cmp::Ordering::Less,
                    ParetoOrder::BDominatesA => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                })
                .then_with(|| {
                    let dist_a = distanza_ottimo_pesata(&a.1, pesi);
                    let dist_b = distanza_ottimo_pesata(&b.1, pesi);
                    dist_a
                        .partial_cmp(&dist_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.2.cmp(&b.2))
        });
        candidati.truncate(k);

        let scores_k: Vec<f64> = candidati.iter().map(|c| c.0).collect();
        let soglia_effettiva = soglia.max(calcola_percentile(&scores_k, percentile_cutoff));

        for (score, _axes, altro_id) in candidati {
            if score >= soglia_effettiva {
                let (from, to) = if fatto.id <= altro_id {
                    (fatto.id.0, altro_id.0)
                } else {
                    (altro_id.0, fatto.id.0)
                };
                if from == to {
                    continue;
                }
                let altro = indice_fatti[&altro_id];
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
            soglia: 0.7,
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

    #[test]
    fn punteggio_applica_clamping_normalizzazione() {
        let f1 = fatto(1, 2.0, -0.5, 1.2);
        let f2 = fatto(2, 1.5, 0.8, 0.5);
        let pesi = [0.6, 0.25, 0.15];

        let p = f1.punteggio(&f2, pesi);
        assert!((0.0..=1.0).contains(&p));
    }

    #[test]
    fn distanza_ottimo_pesata_premia_saturazione_asse_dominante() {
        let pesi = [0.2, 0.8, 0.0];
        let axes_a = NormalizedAxes {
            dense: 0.9,
            sparse: 0.1,
            colbert: 0.5,
        };
        let axes_b = NormalizedAxes {
            dense: 0.5,
            sparse: 0.2,
            colbert: 0.5,
        };

        let dist_a = distanza_ottimo_pesata(&axes_a, pesi);
        let dist_b = distanza_ottimo_pesata(&axes_b, pesi);

        assert!((dist_a - 0.650).abs() < 1e-6);
        assert!((dist_b - 0.562).abs() < 1e-6);
        assert!(
            dist_b < dist_a,
            "B deve essere più vicino all'ottimo pesato rispetto ad A"
        );
    }

    #[test]
    fn tie_break_pareto_prioritizza_dominatore() {
        let f0 = fatto(1, 1.0, 1.0, 1.0);
        let f_dominante = fatto(10, 0.5, 0.5, 0.9);
        let f_dominato = fatto(2, 0.5, 0.5, 0.1);

        let config = GraphConfig {
            k: 1,
            soglia: 0.0,
            percentile_cutoff: 0.0,
            pesi: [0.5, 0.5, 0.0],
        };

        let g = costruisci(&[f0, f_dominante, f_dominato], config);

        let vicini_f0 = g.vicini(NodeId(1));
        assert!(
            vicini_f0.contains(&NodeId(10)),
            "il vicino di f0 deve essere il dominante (10), trovati: {vicini_f0:?}"
        );
    }

    #[test]
    fn tie_break_pesato_vince_chi_e_piu_vicino_all_ottimo() {
        let f0 = fatto(1, 1.0, 1.0, 1.0);
        let a = fatto(10, 0.9, 0.2, 0.5);
        let b = fatto(2, 0.5, 0.3, 0.5);

        let config = GraphConfig {
            k: 1,
            soglia: 0.0,
            percentile_cutoff: 0.0,
            pesi: [0.2, 0.8, 0.0],
        };

        let g = costruisci(&[f0, a, b], config);

        let vicini_f0 = g.vicini(NodeId(1));
        assert!(
            vicini_f0.contains(&NodeId(2)),
            "il vicino di f0 deve essere B (2), il più vicino all'ottimo pesato, trovati: {vicini_f0:?}"
        );
    }

    #[test]
    fn percentile_cutoff_zero_retrocompatibile() {
        let fatti = vec![
            fatto(1, 0.9, 0.9, 0.9),
            fatto(2, 0.8, 0.8, 0.8),
            fatto(3, 0.7, 0.7, 0.7),
        ];
        let config_dinamica = GraphConfig {
            soglia: 0.5,
            percentile_cutoff: 0.0,
            ..GraphConfig::default()
        };
        let config_statica = GraphConfig {
            soglia: 0.5,
            ..GraphConfig::default()
        };
        let g_dinamica = costruisci(&fatti, config_dinamica);
        let g_statica = costruisci(&fatti, config_statica);
        assert_eq!(g_dinamica.archi, g_statica.archi);
    }

    #[test]
    fn soglia_dinamica_piu_selettiva_del_floor() {
        let fatti = vec![
            fatto(1, 1.0, 1.0, 1.0),
            fatto(2, 0.9, 0.9, 0.9),
            fatto(3, 0.5, 0.5, 0.5),
            fatto(4, 0.1, 0.1, 0.1),
        ];
        let config_statica = GraphConfig {
            k: 3,
            soglia: 0.0,
            ..GraphConfig::default()
        };
        let config_dinamica = GraphConfig {
            k: 3,
            soglia: 0.0,
            percentile_cutoff: 0.5,
            ..GraphConfig::default()
        };
        let g_statica = costruisci(&fatti, config_statica);
        let g_dinamica = costruisci(&fatti, config_dinamica);

        assert_eq!(g_statica.n_archi(), 6);

        assert!(
            g_dinamica.n_archi() < g_statica.n_archi(),
            "la soglia dinamica deve produrre meno archi della statica: {} vs {}",
            g_dinamica.n_archi(),
            g_statica.n_archi()
        );
    }

    #[test]
    fn calcola_percentile_interpolazione() {
        let s = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(calcola_percentile(&s, 0.0), 10.0);
        assert_eq!(calcola_percentile(&s, 1.0), 50.0);
        assert_eq!(calcola_percentile(&s, 0.5), 30.0);
        assert_eq!(calcola_percentile(&s, 0.25), 20.0);
        assert_eq!(calcola_percentile(&s, 0.75), 40.0);
        assert_eq!(calcola_percentile(&[], 0.5), 0.0);
        assert_eq!(calcola_percentile(&s, -1.0), 10.0);
        assert_eq!(calcola_percentile(&s, 2.0), 50.0);
    }

    #[test]
    fn test_qualita_selezione_saturazione_asse_dominante() {
        for asse_dominante in 0..3 {
            let mut pesi = [0.1, 0.1, 0.1];
            pesi[asse_dominante] = 0.8;

            let f0 = fatto(1, 1.0, 1.0, 1.0);

            let ca = fatto(10, 0.9, 0.2, 0.2);
            let cb = fatto(20, 0.2, 0.9, 0.2);
            let cc = fatto(30, 0.2, 0.2, 0.9);

            let score_a = f0.punteggio(&ca, pesi);
            let score_b = f0.punteggio(&cb, pesi);
            let score_c = f0.punteggio(&cc, pesi);

            match asse_dominante {
                0 => {
                    assert!(score_a > score_b, "Con asse 0 dominante, A deve superare B");
                    assert!(score_a > score_c, "Con asse 0 dominante, A deve superare C");
                }
                1 => {
                    assert!(score_b > score_a, "Con asse 1 dominante, B deve superare A");
                    assert!(score_b > score_c, "Con asse 1 dominante, B deve superare C");
                }
                2 => {
                    assert!(score_c > score_a, "Con asse 2 dominante, C deve superare A");
                    assert!(score_c > score_b, "Con asse 2 dominante, C deve superare B");
                }
                _ => unreachable!(),
            }
        }
    }
}
