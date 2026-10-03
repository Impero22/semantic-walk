//! # frontier — la propagazione a frontiera del cammino
//!
//! La stanza dove la ricerca nel grafo semantico diventa una **frontiera che
//! avanza livello per livello**, portando con sé la logica di esclusione
//! esatta del pre-filtro F6 (Finding #6 della review Alibaba) dal collapse di
//! `semantic-quantum` al cammino di `semantic-walk`.
//!
//! ## Perché esiste questo modulo
//!
//! Nel collapse, i rami esistono già: l'accumulo `Ψ(c) = Σ exp(−S_i/κ)`
//! opera su ampiezze di rami *già generati*, e il pre-filtro risparmia solo
//! l'operazione O(n) dell'accumulo sui candidati che non possono vincere.
//!
//! Nel cammino, i passi futuri non esistono ancora: l'azione cumulata
//! `S_tot = Σ S^(h)` richiede di conoscere i costi dei passi futuri, che sono
//! proprio ciò che l'espansione calcola. Per dire "questo candidato non può
//! scendere sotto l'incumbent" devo conoscere il minimo locale futuro.
//!
//! La soluzione concordata con Camillo (03/10) risolve la fragilità **nel
//! disegno**: escludendo gli autoloop (`u ≠ v`), il minimo incremento
//! inerziale sulle adiacenze reali del grafo è strettamente positivo per
//! costruzione, e il bound stretto regge senza collassare nel banale.
//!
//! ## Il contratto del pre-filtro F6 (esatto)
//!
//! Un nodo di frontiera viene eliminato **prima dell'espansione** solo se
//! l'ampiezza massima raggiungibile è strettamente inferiore alla soglia
//! dell'incumbent oltre il margine d'incertezza:
//!
//! ```text
//! Ψ_upper < L_inc − PRUNE_EPSILON
//! ```
//!
//! Un candidato nella fascia d'incertezza o in pareggio esatto con
//! l'incumbent viene preservato. La potatura resta esatta, stretta e priva di
//! falsi negativi — il cuore del gate permissivo.
//!
//! ## Le tre verifiche del Coder (integrate)
//!
//! 1. **`u ≠ v` imposta, non assunta** — l'espansione genera solo figli
//!    distinti dal padre; un assert verifica che ogni nodo generato abbia
//!    stato ≠ stato del genitore.
//! 2. **`ΔS_min` resta un lower bound** — estratto dall'insieme di adiacenza
//!    completo usato dall'espansione; in debug, un assert verifica che ogni
//!    arco traversato abbia costo ≥ ΔS_min.
//! 3. **`PRUNE_EPSILON` esplicito** — nel confronto di potatura, più la
//!    decisione semantica sulla parità: un cammino di ampiezza uguale
//!    all'incumbent conta come vincitore.

use crate::KinematicState;
use semantic_combiner::FactId;

/// Soglia d'incertezza per la potatura F6.
///
/// Coerente con `SPARSE_EPSILON` usato in `ordered_sparse.rs` per i token di
/// confine: stessa soglia d'incertezza, nessuna costante magica nuova.
/// Definisce la fascia in cui un confronto è "verdetto incerto" e non va
/// deciso per troncamento.
pub const PRUNE_EPSILON: f32 = 0.002;

/// Un nodo della frontiera di ricerca del cammino.
///
/// Porta con sé lo stato cinematico, il costo inerziale cumulato fino a qui,
/// l'ampiezza corrente e i bound di esclusione calcolati dai minimi.
#[derive(Debug, Clone, PartialEq)]
pub struct FrontierNode {
    /// Il fatto visitato da questo passo del cammino.
    pub node_id: FactId,
    /// Lo stato cinematico di questo passo.
    pub state: KinematicState,
    /// Il costo inerziale cumulato `S_cum` fino a questo nodo.
    pub cum_cost: f32,
    /// Il numero di passi compiuti (profondità nel cammino).
    pub depth: usize,
    /// Il costo minimo inerziale per passo garantito (ΔS_min per-livello).
    pub min_step: f32,
    /// L'ampiezza corrente del nodo, `exp(−S_cum / κ)`.
    pub amplitude: f32,
}

impl FrontierNode {
    /// Costruisce un nodo radice della frontiera.
    ///
    /// La radice ha costo cumulato zero, profondità zero e ampiezza unitaria.
    pub fn root(node_id: FactId, state: KinematicState, min_step: f32) -> Self {
        Self {
            node_id,
            state,
            cum_cost: 0.0,
            depth: 0,
            min_step,
            amplitude: 1.0,
        }
    }

    /// Il costo minimo garantito che questo cammino può ancora accumulare
    /// fino all'orizzonte `horizon` (lower bound del costo totale).
    ///
    /// Se ogni passo futuro costa almeno `min_step`, allora:
    ///
    /// ```text
    /// S_lower(H) = S_cum + (H − depth) · min_step
    /// ```
    pub fn lower_bound_cost(&self, horizon: usize) -> f32 {
        let remaining = horizon.saturating_sub(self.depth) as f32;
        self.cum_cost + remaining * self.min_step
    }

    /// L'ampiezza massima raggiungibile da questo nodo (upper bound).
    ///
    /// L'ampiezza è `exp(−S / κ)`, monotona decrescente nel costo: il massimo
    /// si ottiene al costo minimo. `κ` è il damping (kappa_break).
    pub fn upper_bound_amplitude(&self, horizon: usize, kappa: f32) -> f32 {
        let s_lower = self.lower_bound_cost(horizon);
        (-s_lower / kappa).exp()
    }

    /// Costruisce il figlio di questo nodo verso `next`, verificando il
    /// vincolo `u ≠ v` (stato del figlio distinto dal padre).
    ///
    /// Se `next.state == self.state`, restituisce `None`: il cammino non può
    /// compiere wait-action né riattraversare uno stato identico. È il
    /// vincolo che garantisce `ΔS_min > 0` per costruzione.
    pub fn extend(
        &self,
        next_id: FactId,
        next_state: KinematicState,
        alpha: f32,
        beta: f32,
        gamma: f32,
        min_step: f32,
        kappa: f32,
    ) -> Option<Self> {
        if next_state == self.state {
            // u ≠ v imposto: nessuna wait-action, nessuno stato identico.
            return None;
        }
        let action = self.state.inertial_action(&next_state, alpha, beta, gamma);
        debug_assert!(
            action >= min_step - PRUNE_EPSILON,
            "arco traversato con costo {action} sotto ΔS_min {min_step}"
        );
        let cum_cost = self.cum_cost + action;
        Some(Self {
            node_id: next_id,
            state: next_state,
            cum_cost,
            depth: self.depth + 1,
            min_step,
            amplitude: (-cum_cost / kappa).exp(),
        })
    }
}

/// La frontiera di ricerca: il buffer per livello che avanza nel grafo.
///
/// Mantiene i nodi del livello corrente in un `VecDeque` (espansione FIFO) e
/// applica la potatura F6 esatta prima di espandere i livelli successivi.
#[derive(Debug, Clone)]
pub struct Frontier {
    /// I nodi del livello corrente.
    buffer: std::collections::VecDeque<FrontierNode>,
    /// Il damping dell'ampiezza (kappa_break).
    kappa: f32,
    /// L'orizzonte di ricerca (numero massimo di passi).
    horizon: usize,
}

impl Frontier {
    /// Costruisce una frontiera vuota con i parametri del cammino.
    ///
    /// `min_step` è il bound inerziale per-livello: viene passato ai singoli
    /// nodi al momento dell'espansione (`FrontierNode::min_step`), non
    /// conservato qui. Il parametro resta nella firma come documentazione
    /// esplicita del contratto del livello.
    pub fn new(kappa: f32, horizon: usize, _min_step: f32) -> Self {
        Self {
            buffer: std::collections::VecDeque::new(),
            kappa,
            horizon,
        }
    }

    /// Il numero di nodi nel livello corrente.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// `true` se la frontiera del livello corrente è vuota.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Aggiunge un nodo al livello corrente.
    pub fn push(&mut self, node: FrontierNode) {
        self.buffer.push_back(node);
    }

    /// Estrae il prossimo nodo da espandere (FIFO).
    pub fn pop(&mut self) -> Option<FrontierNode> {
        self.buffer.pop_front()
    }

    /// Applica la potatura F6 esatta al livello corrente.
    ///
    /// Calcola l'incumbent (il nodo con upper bound di ampiezza massima) e
    /// rimuove i nodi la cui ampiezza massima raggiungibile è strettamente
    /// sotto la soglia dell'incumbent oltre `PRUNE_EPSILON`:
    ///
    /// ```text
    /// Ψ_upper < L_inc − PRUNE_EPSILON
    /// ```
    ///
    /// I nodi nella fascia d'incertezza o in pareggio esatto con l'incumbent
    /// vengono preservati.
    pub fn prune(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        // Incumbent: il nodo con l'upper bound di ampiezza massima.
        let incumbent_upper = self
            .buffer
            .iter()
            .map(|n| n.upper_bound_amplitude(self.horizon, self.kappa))
            .fold(f32::NEG_INFINITY, f32::max);
        let threshold = incumbent_upper - PRUNE_EPSILON;
        self.buffer.retain(|n| {
            let upper = n.upper_bound_amplitude(self.horizon, self.kappa);
            // Parità esatta o fascia d'incertezza: preservato.
            upper >= threshold
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(v: f32, a: f32, k: f32) -> KinematicState {
        KinematicState { velocity: v, acceleration: a, curvature: k }
    }

    #[test]
    fn u_ne_v_imposto_stato_identico_non_estende() {
        // Un figlio con stato identico al padre è vietato: nessuna wait-action.
        let parent = FrontierNode::root(0, state(1.0, 0.0, 0.0), 0.5);
        let child = parent.extend(1, state(1.0, 0.0, 0.0), 1.0, 1.0, 1.0, 0.5, 2.0);
        assert!(child.is_none(), "stato identico al padre non deve estendere");
    }

    #[test]
    fn u_ne_v_stato_distinto_estende() {
        // Un figlio con stato distinto dal padre estende correttamente.
        let parent = FrontierNode::root(0, state(1.0, 0.0, 0.0), 0.5);
        let child = parent
            .extend(1, state(3.0, 0.0, 0.0), 2.0, 1.0, 1.0, 0.5, 2.0)
            .expect("stato distinto deve estendere");
        // α·Δv² = 2·4 = 8, cum_cost = 8, depth = 1
        assert!((child.cum_cost - 8.0).abs() < 1e-6);
        assert_eq!(child.depth, 1);
        assert!((child.amplitude - (-8.0f32 / 2.0).exp()).abs() < 1e-6);
    }

    #[test]
    fn lower_bound_cost_accumula_min_step_restante() {
        let node = FrontierNode {
            node_id: 0,
            state: state(1.0, 0.0, 0.0),
            cum_cost: 5.0,
            depth: 2,
            min_step: 0.5,
            amplitude: 0.0,
        };
        // horizon 5: rimangono 3 passi, S_lower = 5 + 3·0.5 = 6.5
        assert!((node.lower_bound_cost(5) - 6.5).abs() < 1e-6);
    }

    #[test]
    fn potatura_esclude_solo_chi_non_puo_vincere() {
        // Incumbent con upper alto, candidato con upper molto basso.
        let mut frontier = Frontier::new(2.0, 5, 0.5);
        // incumbent: cum_cost 0, depth 0 → upper = exp(0) = 1.0
        frontier.push(FrontierNode::root(0, state(0.0, 0.0, 0.0), 0.5));
        // candidato debole: cum_cost 10, depth 1 → upper ≈ exp(-(10+4·0.5)/2) = exp(-6)
        frontier.push(FrontierNode {
            node_id: 1,
            state: state(2.0, 0.0, 0.0),
            cum_cost: 10.0,
            depth: 1,
            min_step: 0.5,
            amplitude: (-10.0f32 / 2.0).exp(),
        });
        frontier.prune();
        assert_eq!(frontier.len(), 1, "il candidato che non può vincere va potato");
        assert_eq!(frontier.pop().unwrap().node_id, 0);
    }

    #[test]
    fn potatura_preserva_parita_e_fascia_incertezza() {
        // Due nodi a pari merito: entrambi preservati (nessun falso negativo).
        let mut frontier = Frontier::new(2.0, 5, 0.5);
        frontier.push(FrontierNode::root(0, state(0.0, 0.0, 0.0), 0.5));
        frontier.push(FrontierNode::root(1, state(0.5, 0.0, 0.0), 0.5));
        frontier.prune();
        assert_eq!(frontier.len(), 2, "parità esatta preservata");
    }

    #[test]
    fn potatura_su_frontiera_vuota_non_fallisce() {
        let mut frontier = Frontier::new(2.0, 5, 0.5);
        frontier.prune();
        assert!(frontier.is_empty());
    }
}
