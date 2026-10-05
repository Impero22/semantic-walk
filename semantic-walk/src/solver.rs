//! # solver — il beam con ampiezza dinamica sul grafo semantico
//!
//! La stanza dove la ricerca nel grafo diventa un **beam che avanza
//! livello per livello**, restituendo l'intero insieme di cammini superstiti
//! fino all'orizzonte `H`.
//!
//! ## Perché il beam, e perché dinamico
//!
//! Le direttive di Camillo (03/10) sono esplicite: troncare a un singolo
//! cammino ottimo vanificherebbe la preservazione dei token di confine. Il
//! pre-filtro F6 esclude solo chi *non può vincere*; tutto ciò che sopravvive
//! fino all'orizzonte è un cammino che porta informazione.
//!
//! Il beam è quindi **dinamico**: la sua ampiezza non è una costante
//! predefinita, ma è governata dall'incumbent — l'upper bound di ampiezza
//! massimo tra i nodi correnti della frontiera — usato come **soglia
//! relativa**. Un nodo resta nel beam finché la sua ampiezza massima
//! raggiungibile non scende sotto la soglia dell'incumbent oltre il margine
//! d'incertezza `PRUNE_EPSILON` (la stessa fascia di "verdetto incerto" dei
//! token di confine).
//!
//! ## Il disaccoppiamento via trait
//!
//! Il solver lavora su un'astrazione di adiacenza: il trait [`GraphAdapter`]
//! che, dato un nodo, restituisce i vicini con il relativo [`KinematicState`].
//! Questo svincola il motore di ricerca dalla struttura fisica del grafo:
//!
//! * [`ProximityGraph`](crate::adapter::ProximityGraph) (in `adapter.rs`)
//!   è l'implementazione concreta per il Dataset B;
//! * i test unitari possono usare un adapter fittizio su grafi minimi,
//!   senza costruire la struttura fisica reale.

use crate::frontier::{Frontier, FrontierNode};
use crate::KinematicState;
use semantic_combiner::FactId;

/// L'astrazione di adiacenza su cui opera il solver.
///
/// Dato un nodo, restituisce i suoi vicini con il relativo stato cinematico.
/// Il solver non conosce la struttura fisica del grafo: conosce solo questa
/// porta di accesso.
pub trait GraphAdapter {
    /// I vicini di `node`, con lo stato cinematico di ciascuno.
    ///
    /// L'ordine non è significativo per la correttezza del beam (la potatura
    /// F6 è indipendente dall'ordine di espansione); può esserlo per la
    /// riproducibilità dei test, ed è quindi lasciato all'implementazione.
    fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)>;
}

/// I parametri del beam con ampiezza dinamica.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolverConfig {
    /// L'orizzonte di ricerca: numero massimo di passi.
    pub horizon: usize,
    /// Il damping dell'ampiezza (kappa_break).
    pub kappa: f32,
    /// Coefficienti dell'azione inerziale (rigidità della camminata).
    pub beta: f32,
    pub gamma: f32,
    /// La velocità semantica massima (barriera relativistica, Fase 1).
    pub c_sem: f32,
    /// La massa semantica (coefficiente del termine relativistico, Fase 1).
    pub m_sem: f32,
    /// Il bound inerziale per-livello (minimo empirico osservato).
    pub min_step: f32,
}

impl Default for SolverConfig {
    fn default() -> Self {
        SolverConfig {
            horizon: 8,
            kappa: 1.0,
            beta: 1.0,
            gamma: 1.0,
            c_sem: 100.0,
            m_sem: 1.0,
            min_step: 0.002,
        }
    }
}

/// Un cammino superstite: la sequenza di nodi visitati fino all'orizzonte.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    /// I nodi del cammino, in ordine di visita.
    pub nodes: Vec<FactId>,
    /// Il costo inerziale cumulato totale.
    pub total_cost: f32,
    /// L'ampiezza finale del cammino, `exp(−S_tot / κ)`.
    pub amplitude: f32,
}

/// Il beam con ampiezza dinamica.
///
/// Avanza livello per livello sul grafo tramite [`GraphAdapter`], applicando
/// a ogni livello la potatura F6 esatta (incumbent come soglia relativa) e
/// restituendo l'intero insieme di cammini superstiti fino all'orizzonte.
#[derive(Debug, Clone)]
pub struct BeamSolver<A> {
    adapter: A,
    config: SolverConfig,
}

impl<A: GraphAdapter> BeamSolver<A> {
    /// Costruisce il solver su un adapter con la configurazione data.
    pub fn new(adapter: A, config: SolverConfig) -> Self {
        Self { adapter, config }
    }

    /// Esegue la ricerca dal nodo radice, restituendo tutti i cammini
    /// superstiti fino all'orizzonte.
    ///
    /// Il beam è dinamico: l'ampiezza di ogni livello è governata
    /// dall'incumbent (upper bound di ampiezza massimo tra i nodi correnti),
    /// usato come soglia relativa. I cammini che sopravvivono alla potatura
    /// F6 fino all'orizzonte vengono restituiti per intero — nessun
    /// troncamento a un singolo cammino ottimo.
    pub fn solve(&self, root: FactId, root_state: KinematicState) -> Vec<Path> {
        let mut frontier = Frontier::new(self.config.kappa, self.config.horizon, self.config.min_step);
        frontier.push(FrontierNode::root(root, root_state, self.config.min_step));

        let mut paths: Vec<Path> = Vec::new();

        while !frontier.is_empty() {
            let mut next = Frontier::new(self.config.kappa, self.config.horizon, self.config.min_step);

            while let Some(node) = frontier.pop() {
                // Se abbiamo raggiunto l'orizzonte, il cammino è completo:
                // viene salvato per intero e non viene ulteriormente espanso.
                if node.depth >= self.config.horizon {
                    paths.push(Self::to_path(&node));
                    continue;
                }

                let neighbors = self.adapter.neighbors(node.node_id);

                // Dead-end prima dell'orizzonte: il cammino termina qui. È un
                // cammino superstite valido — porta informazione fino al suo
                // punto di arresto, e va restituito per intero.
                if neighbors.is_empty() {
                    paths.push(Self::to_path(&node));
                    continue;
                }

                for (next_id, next_state) in neighbors {
                    if let Some(child) = node.extend(
                        next_id,
                        next_state,
                        self.config.beta,
                        self.config.gamma,
                        self.config.c_sem,
                        self.config.m_sem,
                        self.config.min_step,
                        self.config.kappa,
                    ) {
                        next.push(child);
                    }
                }
            }

            // Potatura F6 esatta sul livello dei figli appena espansi:
            // l'incumbent è il migliore tra i nodi dello stesso livello di
            // profondità (pari), non il root del livello precedente. Il root
            // non compete coi figli — sono livelli diversi. Potare qui evita
            // il caso degenere in cui il root (ampiezza 1.0) pota tutti i
            // figli perché hanno costo cumulato maggiore.
            next.prune();

            frontier = next;
        }

        paths
    }

    /// Converte un nodo della frontiera in un cammino superstite.
    ///
    /// La radice è il punto di partenza noto (il contesto della ricerca), non
    /// un passo del cammino: la sequenza restituita contiene quindi i soli
    /// nodi visitati dopo la radice.
    fn to_path(node: &FrontierNode) -> Path {
        Path {
            nodes: node.nodes.iter().skip(1).copied().collect(),
            total_cost: node.cum_cost,
            amplitude: node.amplitude,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(v: f32, a: f32, k: f32) -> KinematicState {
        KinematicState { velocity: v, acceleration: a, curvature: k }
    }

    /// Un adapter fittizio su un grafo a due nodi: 0 → 1.
    struct LinearAdapter;

    impl GraphAdapter for LinearAdapter {
        fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)> {
            match node {
                0 => vec![(1, state(1.0, 0.0, 0.0))],
                _ => vec![],
            }
        }
    }

    #[test]
    fn beam_restituisce_il_cammino_lineare() {
        let solver = BeamSolver::new(LinearAdapter, SolverConfig {
            horizon: 2,
            ..Default::default()
        });
        let paths = solver.solve(0, state(0.0, 0.0, 0.0));
        // Un solo cammino: 0 → 1, due passi.
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].nodes, vec![1]);
        assert_eq!(paths[0].total_cost, 0.5000125); // m·c²·(γ−1) con Δv=1, c=100, m=1
    }

    #[test]
    fn beam_conserva_piu_cammini_superstiti() {
        // Adapter con due rami: 0 → 1 e 0 → 2, entrambi a pari costo.
        struct ForkAdapter;
        impl GraphAdapter for ForkAdapter {
            fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)> {
                match node {
                    0 => vec![(1, state(1.0, 0.0, 0.0)), (2, state(1.0, 0.0, 0.0))],
                    _ => vec![],
                }
            }
        }
        let solver = BeamSolver::new(ForkAdapter, SolverConfig {
            horizon: 2,
            ..Default::default()
        });
        let paths = solver.solve(0, state(0.0, 0.0, 0.0));
        // Due cammini superstiti, entrambi preservati (nessun troncamento).
        assert_eq!(paths.len(), 2);
        let mut nodes: Vec<FactId> = paths.iter().map(|p| p.nodes[0]).collect();
        nodes.sort();
        assert_eq!(nodes, vec![1, 2]);
    }

    #[test]
    fn beam_esclude_chi_non_puo_vincere() {
        // Due rami: 1 con costo basso (ampiezza alta), 2 con costo altissimo.
        // Il ramo 2 non può vincere e viene potato.
        struct SkewedAdapter;
        impl GraphAdapter for SkewedAdapter {
            fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)> {
                match node {
                    0 => vec![
                        (1, state(1.0, 0.0, 0.0)),        // Δv=1 → costo 1
                        (2, state(100.0, 0.0, 0.0)),      // Δv=100 → costo 10000
                    ],
                    _ => vec![],
                }
            }
        }
        let solver = BeamSolver::new(SkewedAdapter, SolverConfig {
            horizon: 2,
            kappa: 1.0,
            ..Default::default()
        });
        let paths = solver.solve(0, state(0.0, 0.0, 0.0));
        // Solo il ramo a costo basso sopravvive.
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].nodes, vec![1]);
    }
}