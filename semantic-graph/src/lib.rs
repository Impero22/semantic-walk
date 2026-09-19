//! # semantic-graph — il grafo di prossimità
//!
//! L'ultima stanza della casa. Il combinatore risponde a "quanto è simile A
//! a B?", il gate a "vale la pena confrontare A con B?". Il grafo va oltre:
//! organizza l'insieme dei fatti in uno spazio, rivelando la **forma** della
//! memoria — i cluster, gli orfani, i ponti.
//!
//! Non è un elenco ordinato per punteggio. È una **mappa**.
//!
//! ## Filosofia
//!
//! - **Il giudizio è già nei dati**: il grafo estrae la geometria, non la
//!   inventa. La comprensione è già nella geometria.
//! - **Coerenza Pareto**: il grafo non viola mai l'invariante del
//!   combinatore — se A domina B su tutti gli assi, A è più vicino a ogni
//!   riferimento comune.
//! - **Costruzione economica, risultato coerente**: la costruzione può
//!   essere asimmetrica (k-nearest), ma il grafo risultante è simmetrico.
//!
//! ## Nota sulla memoria
//!
//! A differenza del gate (che decide in tempo reale sotto budget), il grafo
//! si costruisce una volta e si consulta. Qui l'allocazione è accettabile:
//! è il prezzo della mappa. Lo zero-alloc resta per le metriche, che
//! calcolano su stack al bisogno.
//!
//! ## Moduli
//!
//! - [`costruisci`] — la costruzione del grafo (ibrida k-nearest + soglia).
//! - [`metriche`] — grado, clustering, orfani (funzioni pure).

pub mod costruisci;
pub mod metriche;

pub use costruisci::{costruisci, Fatto};
pub use metriche::{clustering, e_orfano, grado, gradi, orfani};

/// L'identificativo di un nodo del grafo — l'id di un fatto della memoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

/// Un arco del grafo: un legame pesato tra due fatti.
///
/// Lo score è il punteggio combinato (la forza del legame). La tripletta
/// `(dense, sparse, colbert)` è la *qualità* del legame — da quale asse
/// arriva la vicinanza.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub score: f64,
    pub dense: f64,
    pub sparse: f64,
    pub colbert: f64,
}

impl Edge {
    /// Il punteggio simmetrico dell'arco (la similarità è simmetrica).
    pub fn score(&self) -> f64 {
        self.score
    }
}

/// Configurazione del grafo.
///
/// - `k` — i vicini per nodo nella costruzione k-nearest.
/// - `soglia` — floor assoluto in `[0, 1]`: un arco esiste solo se il
///   punteggio combinato supera *almeno* questo valore.
/// - `percentile_cutoff` — soglia dinamica in `[0, 1]`: per ogni nodo, la
///   soglia effettiva è `max(soglia, percentile dei punteggi dei top-k)`.
///   Con `0.0` la soglia dinamica è disattivata (comportamento statico
///   puro, retrocompatibile).
/// - `pesi` — i pesi del combinatore `[dense, sparse, colbert]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphConfig {
    pub k: usize,
    pub soglia: f64,
    pub percentile_cutoff: f64,
    pub pesi: [f64; 3],
}

impl Default for GraphConfig {
    fn default() -> Self {
        GraphConfig {
            k: 5,
            soglia: 0.0,
            // soglia statica pura (retrocompatibile).
            //
            // Valore raccomandato per la produzione con dataset a canale
            // singolo (sola similarità Jaccard sulle celle): **0.50**.
            // Calibrato dallo sweep su 153 traiettorie reali (fe35366):
            // a 0.50 il grado medio converge a k=5, la varianza si dimezza
            // (21.92→14.34), la connettività resta un'unica componente e il
            // clustering si preserva (0.493→0.435). Sopra 0.5 la topologia
            // degrada (clustering a 0.327 a 0.75, 22 componenti a 0.9).
            // Da ricalibrare quando CrispEmbed fornirà sparse e colbert.
            percentile_cutoff: 0.0,
            pesi: [0.6, 0.25, 0.15], // fusione canonica dal progetto
        }
    }
}

/// Il grafo di prossimità.
///
/// - `config` — la configurazione con cui è stato costruito.
/// - `nodi` — i nodi, ordinati e senza duplicati.
/// - `archi` — gli archi, ordinati e senza duplicati (simmetrici).
#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    pub config: GraphConfig,
    pub nodi: Vec<NodeId>,
    pub archi: Vec<Edge>,
}

impl Graph {
    /// Un grafo vuoto con la configurazione data.
    pub fn new(config: GraphConfig) -> Self {
        Graph {
            config,
            nodi: Vec::new(),
            archi: Vec::new(),
        }
    }

    /// Il numero di nodi.
    pub fn n_nodi(&self) -> usize {
        self.nodi.len()
    }

    /// Il numero di archi.
    pub fn n_archi(&self) -> usize {
        self.archi.len()
    }

    /// `true` se l'arco (simmetrico) tra `a` e `b` esiste.
    ///
    /// La ricerca è binaria: gli archi sono ordinati per coppia ordinata
    /// (from, to) con `from < to`, così l'arco A—B è memorizzato una sola
    /// volta con l'estremo minore in `from`.
    pub fn ha_arco(&self, a: NodeId, b: NodeId) -> bool {
        let (from, to) = if a <= b { (a, b) } else { (b, a) };
        self.archi
            .binary_search_by(|e| (e.from, e.to).cmp(&(from, to)))
            .is_ok()
    }

    /// I vicini di un nodo: tutti i nodi collegati ad esso da un arco.
    ///
    /// La ricerca è lineare sugli archi; il risultato non è ordinato.
    pub fn vicini(&self, n: NodeId) -> Vec<NodeId> {
        self.archi
            .iter()
            .filter(|e| e.from == n || e.to == n)
            .map(|e| if e.from == n { e.to } else { e.from })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grafo_vuoto() {
        let g = Graph::new(GraphConfig::default());
        assert_eq!(g.n_nodi(), 0);
        assert_eq!(g.n_archi(), 0);
    }

    #[test]
    fn arco_simmetrico() {
        let mut g = Graph::new(GraphConfig::default());
        let a = NodeId(1);
        let b = NodeId(2);
        g.nodi = vec![a, b];
        g.archi = vec![Edge {
            from: a,
            to: b,
            score: 0.7,
            dense: 0.8,
            sparse: 0.5,
            colbert: 0.6,
        }];
        // L'arco esiste in entrambe le direzioni.
        assert!(g.ha_arco(a, b));
        assert!(g.ha_arco(b, a));
        // Ma non esiste verso un terzo nodo.
        assert!(!g.ha_arco(a, NodeId(3)));
    }

    #[test]
    fn coerenza_pareto() {
        // A domina B su tutti gli assi: se B si collega a R, anche A deve
        // essere almeno altrettanto vicino (l'arco A—R deve esistere).
        use crate::costruisci::Fatto;

        let a = Fatto {
            id: NodeId(1),
            dense: 0.9,
            sparse: 0.9,
            colbert: 0.9,
        };
        let b = Fatto {
            id: NodeId(2),
            dense: 0.5,
            sparse: 0.5,
            colbert: 0.5,
        };
        let r = Fatto {
            id: NodeId(3),
            dense: 0.8,
            sparse: 0.8,
            colbert: 0.8,
        };

        let g = costruisci(&[a, b, r], GraphConfig::default());
        // B e R sono simili (0.5*0.8 = 0.4 su ogni asse pesato) — dovrebbero
        // collegarsi. E poiché A domina B, anche A—R deve esistere.
        assert!(g.ha_arco(NodeId(2), NodeId(3)), "B—R deve esistere");
        assert!(g.ha_arco(NodeId(1), NodeId(3)), "A—R deve esistere (A domina B)");
    }
}

pub mod zonizzazione;
