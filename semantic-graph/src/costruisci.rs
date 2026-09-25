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

    // Indice `id -> &Fatto` pre-costruito una sola volta (O(N)).
    //
    // Senza questo indice, per ogni coppia di fatti il ciclo di costruzione
    // degli archi faceva `fatti.iter().find(|f| f.id == altro_id).unwrap()`:
    // una scansione lineare O(N) dentro un ciclo già O(N²), portando il costo
    // complessivo a O(N³). Con l'indice, il recupero del fatto per id è O(1).
    //
    // Nota sul `unwrap`: è sicuro perché `altro_id` proviene da `candidati`,
    // che è costruito iterando esattamente gli elementi di `fatti` — quindi
    // ogni id presente nei candidati esiste di sicuro nell'indice. Il
    // `HashMap::get` ritorna `None` solo per un id assente, che qui non può
    // verificarsi per costruzione.
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
                .then_with(|| a.2.cmp(&b.2))
        });
        candidati.truncate(k);

        // Soglia effettiva per questo nodo: il floor assoluto, oppure il
        // percentile dei punteggi dei top-k se la soglia dinamica è attiva.
        // Con `percentile_cutoff = 0.0` il percentile è il minimo dei top-k,
        // quindi `soglia_effettiva = max(soglia, minimo) = soglia` — identico
        // al comportamento statico (retrocompatibilità).
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
                // Recupero O(1) tramite l'indice pre-costruito (vedi sopra).
                // In precedenza: `fatti.iter().find(...).unwrap()` — O(N) per
                // ogni arco, portando l'intera costruzione a O(N³).
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
            soglia: 0.7, // Adeguata allo spazio degli assi normalizzati [0, 1]
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
        // Fatti con valori fuori scala o negativi
        let f1 = fatto(1, 2.0, -0.5, 1.2);
        let f2 = fatto(2, 1.5, 0.8, 0.5);
        let pesi = [0.6, 0.25, 0.15];

        let p = f1.punteggio(&f2, pesi);
        assert!((0.0..=1.0).contains(&p));
    }

    #[test]
    fn tie_break_pareto_prioritizza_dominatore() {
        // Pesi con peso zero su Colbert: A e B producono lo stesso score scalare.
        // A ha id 10 ma domina B su Colbert. B ha id 2 (più basso).
        // Il tie-break deve scegliere A nonostante B abbia un NodeId inferiore.
        let f0 = fatto(1, 1.0, 1.0, 1.0);
        let f_dominante = fatto(10, 0.5, 0.5, 0.9); // ID 10, Colbert alto
        let f_dominato = fatto(2, 0.5, 0.5, 0.1);   // ID 2, Colbert basso

        let config = GraphConfig {
            k: 1,
            soglia: 0.0,
            percentile_cutoff: 0.0,
            pesi: [0.5, 0.5, 0.0], // Colbert ignorato nello score scalare
        };

        let g = costruisci(&[f0, f_dominante, f_dominato], config);

        // Il vicino di f0 deve essere NodeId(10) (dominante), non NodeId(2):
        // a parità di score scalare, il tie-break Pareto sceglie chi domina
        // sugli assi non pesati. Il grafo è simmetrico, quindi l'arco (1,2)
        // può esistere legittimamente dal lato di f_dominato (che preferisce
        // f0 come vicino) — qui verifichiamo solo la scelta del lato di f0.
        let vicini_f0 = g.vicini(NodeId(1));
        assert!(
            vicini_f0.contains(&NodeId(10)),
            "il vicino di f0 deve essere il dominante (10), trovati: {vicini_f0:?}"
        );
    }

    #[test]
    fn percentile_cutoff_zero_retrocompatibile() {
        // Con percentile_cutoff = 0.0 la soglia dinamica è disattivata:
        // la soglia effettiva è il floor statico. Stesso comportamento
        // del codice prima dell'introduzione della soglia dinamica.
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
        // Con percentile_cutoff = 0.5 (mediana dei top-k), la soglia effettiva
        // per ogni nodo è la mediana dei suoi punteggi: circa metà dei top-k
        // per nodo viene tagliata. Il grafo risultante deve essere più
        // selettivo (meno archi) del grafo con la sola soglia statica a 0.0.
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
            percentile_cutoff: 0.5, // mediana dei top-k per nodo
            ..GraphConfig::default()
        };
        let g_statica = costruisci(&fatti, config_statica);
        let g_dinamica = costruisci(&fatti, config_dinamica);

        // Con la soglia statica a 0.0, ogni nodo si collega ai suoi top-3
        // (tutti gli altri): grafo completo su 4 nodi = 6 archi.
        assert_eq!(g_statica.n_archi(), 6);

        // Con la mediana, ogni nodo perde circa metà dei suoi candidati:
        // il grafo deve avere meno archi di quello statico.
        assert!(
            g_dinamica.n_archi() < g_statica.n_archi(),
            "la soglia dinamica deve produrre meno archi della statica: {} vs {}",
            g_dinamica.n_archi(),
            g_statica.n_archi()
        );
    }

    #[test]
    fn calcola_percentile_interpolazione() {
        // Serie [10, 20, 30, 40, 50]: mediana (p=0.5) = 30.
        let s = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(calcola_percentile(&s, 0.0), 10.0);
        assert_eq!(calcola_percentile(&s, 1.0), 50.0);
        assert_eq!(calcola_percentile(&s, 0.5), 30.0);
        // Interpolazione: p=0.25 → idx = 1.0 → 20.0; p=0.75 → idx = 3.0 → 40.0.
        assert_eq!(calcola_percentile(&s, 0.25), 20.0);
        assert_eq!(calcola_percentile(&s, 0.75), 40.0);
        // Serie vuota → 0.0.
        assert_eq!(calcola_percentile(&[], 0.5), 0.0);
        // Clamping di p fuori range.
        assert_eq!(calcola_percentile(&s, -1.0), 10.0);
        assert_eq!(calcola_percentile(&s, 2.0), 50.0);
    }
}
