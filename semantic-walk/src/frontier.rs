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
//! disegno**: escludendo gli autoloop (`u ≠ v`), l'incremento inerziale sulle
//! adiacenze reali del grafo è strettamente positivo (`action > 0`). Il lower
//! bound **esatto** non esiste in generale (le componenti cinematiche f32
//! reali possono avvicinarsi a zero arbitrariamente): il `MIN_STEP` usato
//! per la potatura è il minimo **empirico osservato**, misurato a runtime
//! sugli archi reali — non una costante arbitraria né un valore "garantito".
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
//! 2. **`ΔS_min` è il minimo empirico osservato** — non un lower bound
//!    teorico "garantito" (le componenti cinematiche reali possono
//!    avvicinarsi a zero arbitrariamente): è misurato a runtime sugli archi
//!    reali del grafo, e in debug un assert verifica che ogni arco traversato
//!    abbia costo ≥ ΔS_min.
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
pub const PRUNE_EPSILON: f32 = 0.00001;

/// Un nodo della frontiera di ricerca del cammino.
///
/// Porta con sé lo stato cinematico, il costo inerziale cumulato fino a qui,
/// l'ampiezza corrente, la sequenza di nodi visitati e i bound di esclusione
/// calcolati dai minimi.
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
    /// La sequenza di nodi visitati fino a questo passo (storia del cammino).
    pub nodes: Vec<FactId>,
    /// Il costo minimo inerziale per passo (ΔS_min per-livello).
    ///
    /// Non è un lower bound teorico garantito da `u ≠ v`: le componenti
    /// cinematiche f32 reali possono differire di quantità arbitrariamente
    /// piccole anche tra stati distinti. È il minimo **empirico osservato**
    /// sugli archi reali del grafo, misurato a runtime (lezione
    /// SPARSE_EPSILON: il bound deve stare dove il fenomeno è, non sotto).
    pub min_step: f32,
    /// L'ampiezza corrente del nodo, `exp(−S_cum / κ)`.
    pub amplitude: f32,
}

impl FrontierNode {
    /// Costruisce un nodo radice della frontiera.
    ///
    /// La radice ha costo cumulato zero, profondità zero, ampiezza unitaria
    /// e una storia che contiene il solo nodo radice.
    pub fn root(node_id: FactId, state: KinematicState, min_step: f32) -> Self {
        Self {
            node_id,
            state,
            cum_cost: 0.0,
            depth: 0,
            nodes: vec![node_id],
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
    /// compiere wait-action né riattraversare uno stato identico. Il vincolo
    /// `u ≠ v` garantisce `action > 0` (stati distinti → almeno una componente
    /// diversa), ma **non** un valore positivo fissato: le componenti reali
    /// possono produrre azioni arbitrariamente piccole. Il lower bound esatto
    /// non esiste in generale — esiste il minimo empirico osservato, che va
    /// misurato e non assunto.
    pub fn extend(
        &self,
        next_id: FactId,
        next_state: KinematicState,
        beta: f32,
        gamma: f32,
        c_sem: f32,
        m_sem: f32,
        min_step: f32,
        kappa: f32,
    ) -> Option<Self> {
        // Ciclo-detection: non riattraversare mai un nodo già visitato nel
        // cammino corrente, qualunque sia lo stato con cui ci si arriverebbe.
        // Il controllo è fuori dal blocco dello stato identico: il caso comune
        // è riattraversare un nodo con uno stato DIVERSO (arrivandoci da un
        // percorso alternativo), e va intercettato anche quello.
        if self.nodes.contains(&next_id) {
            return None;
        }
        if next_state == self.state {
            // u ≠ v imposto: nessuna wait-action, nessuno stato identico.
            return None;
        }
        let action = self.state.inertial_action(&next_state, beta, gamma, c_sem, m_sem);
        debug_assert!(
            action >= min_step - PRUNE_EPSILON,
            "arco traversato con costo {action} sotto ΔS_min {min_step}"
        );
        let mut nodes = self.nodes.clone();
        nodes.push(next_id);
        let cum_cost = self.cum_cost + action;
        Some(Self {
            node_id: next_id,
            state: next_state,
            cum_cost,
            depth: self.depth + 1,
            nodes,
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
        let threshold = incumbent_upper * (1.0 - PRUNE_EPSILON);
        self.buffer.retain(|n| {
            let upper = n.upper_bound_amplitude(self.horizon, self.kappa);
            // Parità esatta o fascia d'incertezza: preservato.
            upper >= threshold
        });
    }
}

/// Metriche di profilazione per-livello dell'azione inerziale.
///
/// Strumento di **diagnostica** (non di potatura): misura la distribuzione
/// dell'incremento inerziale `ΔS` sugli archi che la frontiera attraversa a
/// un dato livello, senza esporre il pruning a decisioni locali non monotone.
///
/// Il bound di potatura resta ancorato al `MIN_STEP` globale (lower bound
/// teorico garantito da `u ≠ v`); queste metriche servono a capire dove il
/// grafo è stretto e dove degrada, non a decidere cosa escludere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelDiagnostics {
    /// Il numero di archi osservati in questo livello.
    pub count: usize,
    /// Il minimo incremento inerziale osservato.
    pub min: f32,
    /// Il massimo incremento inerziale osservato.
    pub max: f32,
    /// La media degli incrementi inerziali osservati.
    pub mean: f32,
}

impl LevelDiagnostics {
    /// Calcola le metriche da un iteratore di incrementi inerziali.
    ///
    /// Restituisce `None` se l'iteratore è vuoto (nessun arco attraversato).
    pub fn from_actions(actions: impl IntoIterator<Item = f32>) -> Option<Self> {
        let mut count = 0usize;
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        let mut sum = 0.0f32;
        for a in actions {
            count += 1;
            if a < min {
                min = a;
            }
            if a > max {
                max = a;
            }
            sum += a;
        }
        if count == 0 {
            return None;
        }
        Some(Self {
            count,
            min,
            max,
            mean: sum / count as f32,
        })
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
        let child = parent.extend(1, state(1.0, 0.0, 0.0), 1.0, 1.0, 100.0, 1.0, 0.5, 2.0);
        assert!(child.is_none(), "stato identico al padre non deve estendere");
    }

    #[test]
    fn u_ne_v_stato_distinto_estende() {
        // Un figlio con stato distinto dal padre estende correttamente.
        let parent = FrontierNode::root(0, state(1.0, 0.0, 0.0), 0.5);
        let child = parent
            .extend(1, state(3.0, 0.0, 0.0), 1.0, 1.0, 100.0, 1.0, 0.5, 2.0)
            .expect("stato distinto deve estendere");
        // m·c²·(γ−1) con Δv=2, c=100, m=1: γ−1≈0.0002 → cum_cost ≈ 2.0002,
        // depth = 1
        assert!((child.cum_cost - 2.0002).abs() < 1e-3);
        assert_eq!(child.depth, 1);
        assert!((child.amplitude - (-2.0002f32 / 2.0).exp()).abs() < 1e-3);
    }

    #[test]
    fn lower_bound_cost_accumula_min_step_restante() {
        let node = FrontierNode {
            node_id: 0,
            state: state(1.0, 0.0, 0.0),
            cum_cost: 5.0,
            depth: 2,
            nodes: vec![0],
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
            nodes: vec![0, 1],
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

    #[test]
    fn diagnostica_calcola_min_max_media() {
        // 3 archi: 0.5, 1.0, 1.5 → min 0.5, max 1.5, media 1.0
        let d = LevelDiagnostics::from_actions([0.5, 1.0, 1.5]).expect("3 archi");
        assert_eq!(d.count, 3);
        assert!((d.min - 0.5).abs() < 1e-6);
        assert!((d.max - 1.5).abs() < 1e-6);
        assert!((d.mean - 1.0).abs() < 1e-6);
    }

    #[test]
    fn diagnostica_su_nessun_arco_restituisce_none() {
        // Nessun arco attraversato → nessuna metrica.
        assert!(LevelDiagnostics::from_actions(Vec::<f32>::new()).is_none());
    }
}
