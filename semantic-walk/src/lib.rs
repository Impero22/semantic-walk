//! # semantic-walk
//!
//! La stanza del **cammino cinematico** del progetto `semantic-geo`.
//!
//! Dove la navigazione dello spazio semantico diventa una traiettoria: ogni
//! passo ha uno stato cinematico (velocità, accelerazione, curvatura) e il
//! costo del cammino misura la *penalità di deviazione* tra uno stato e il
//! successivo.
//!
//! ## La cinematica del senso
//!
//! Il cuore del crate è [`KinematicState::inertial_action`], che calcola
//! l'azione inerziale $S_{Inertial}$ tra due stati consecutivi:
//!
//! $$S_{Inertial} = \alpha \cdot \Vert \Delta v \Vert^2 + \beta \cdot \Vert \Delta a \Vert^2 + \gamma \cdot \vert \Delta \kappa \vert$$
//!
//! * **$\Delta v$** — variazione di velocità: lo sforzo di
//!   accelerazione/decelerazione nel campo semantico.
//! * **$\Delta a$** — variazione di accelerazione: il *jerk*, lo strappo
//!   semantico, che penalizza i cambi bruschi di direzione.
//! * **$\Delta \kappa$** — variazione di curvatura: la deviazione dalla
//!   traiettoria geodetica nello spazio del grafo.
//!
//! I tre parametri $(\alpha, \beta, \gamma)$ calibrano la *rigidità della
//! camminata*: un $\alpha$ alto rende costosi gli scatti, un $\beta$ alto
//! rende costosi gli strappi, un $\gamma$ alto rende costose le curve.

use semantic_combiner::FactId;

/// Lo stato cinematico di un passo della traiettoria semantica.
///
/// Le tre grandezze descrivono il *moto* nel campo semantico:
///
/// * `velocity` — la velocità di avanzamento (quanto il significato si muove
///   per unità di cammino).
/// * `acceleration` — la variazione di velocità (lo sforzo applicato).
/// * `curvature` — la curvatura locale della traiettoria (quanto si devia
///   dalla linea retta/geodetica).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KinematicState {
    pub velocity: f32,
    pub acceleration: f32,
    pub curvature: f32,
}

impl Default for KinematicState {
    fn default() -> Self {
        Self { velocity: 0.0, acceleration: 0.0, curvature: 0.0 }
    }
}

impl KinematicState {
    /// Calcola l'**azione inerziale** $S_{Inertial}$ tra questo stato e uno
    /// successivo.
    ///
    /// La formula (formalizzata da Camillo) misura la penalità di deviazione
    /// della traiettoria:
    ///
    /// $$S_{Inertial} = \alpha \cdot \Delta v^2 + \beta \cdot \Delta a^2 + \gamma \cdot \vert \Delta \kappa \vert$$
    ///
    /// con $\Delta v = v_{next} - v_{self}$, $\Delta a = a_{next} - a_{self}$,
    /// $\Delta \kappa = \kappa_{next} - \kappa_{self}$.
    ///
    /// I tre coefficienti calibrano la rigidità della camminata:
    ///
    /// * $\alpha$ — costo degli scatti di velocità (accelerazione/decelerazione).
    /// * $\beta$ — costo del jerk (cambi bruschi di accelerazione).
    /// * $\gamma$ — costo delle curve (deviazione dalla geodetica).
    ///
    /// Il risultato è **sempre non negativo** (somma di quadrati e valori
    /// assoluti): l'azione inerziale è un costo, mai un guadagno.
    ///
    /// Uno stato identico al successivo produce azione zero — il cammino
    /// rettilineo uniforme è la traiettoria più economica, come nella fisica.
    pub fn inertial_action(
        &self,
        next: &KinematicState,
        alpha: f32,
        beta: f32,
        gamma: f32,
    ) -> f32 {
        let dv = next.velocity - self.velocity;
        let da = next.acceleration - self.acceleration;
        let dk = next.curvature - self.curvature;
        alpha * dv * dv + beta * da * da + gamma * dk.abs()
    }
}

/// Un passo della traiettoria: il nodo visitato e il suo stato cinematico.
#[derive(Debug, Clone)]
pub struct TrajectoryStep {
    pub node_id: FactId,
    pub state: KinematicState,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(v: f32, a: f32, k: f32) -> KinematicState {
        KinematicState { velocity: v, acceleration: a, curvature: k }
    }

    #[test]
    fn azione_zero_per_stato_identico() {
        // Cammino rettilineo uniforme: nessuna deviazione, azione zero.
        let s = state(1.0, 0.0, 0.0);
        assert!((s.inertial_action(&s, 1.0, 1.0, 1.0)).abs() < 1e-6);
    }

    #[test]
    fn azione_misura_deviazione_di_velocita() {
        // Solo la velocità cambia: l'azione deve dipendere solo da α.
        let prev = state(1.0, 0.0, 0.0);
        let next = state(3.0, 0.0, 0.0); // Δv = 2
        let s = prev.inertial_action(&next, 2.0, 1.0, 1.0);
        // α·Δv² = 2·4 = 8
        assert!((s - 8.0).abs() < 1e-6);
    }

    #[test]
    fn azione_misura_deviazione_di_accelerazione() {
        // Solo l'accelerazione cambia: dipende solo da β (jerk).
        let prev = state(0.0, 1.0, 0.0);
        let next = state(0.0, 4.0, 0.0); // Δa = 3
        let s = prev.inertial_action(&next, 1.0, 2.0, 1.0);
        // β·Δa² = 2·9 = 18
        assert!((s - 18.0).abs() < 1e-6);
    }

    #[test]
    fn azione_misura_deviazione_di_curvatura() {
        // Solo la curvatura cambia: dipende da γ e dal valore assoluto.
        let prev = state(0.0, 0.0, 1.0);
        let next = state(0.0, 0.0, -2.0); // Δκ = -3, |Δκ| = 3
        let s = prev.inertial_action(&next, 1.0, 1.0, 4.0);
        // γ·|Δκ| = 4·3 = 12
        assert!((s - 12.0).abs() < 1e-6);
    }

    #[test]
    fn azione_sempre_non_negativa() {
        // Somma di quadrati e valori assoluti: mai negativa, qualunque input.
        let prev = state(-3.0, 2.5, -1.0);
        let next = state(4.0, -1.5, 0.5);
        let s = prev.inertial_action(&next, 0.7, 1.3, 2.1);
        assert!(s >= 0.0);
    }

    proptest::proptest! {
        /// Proprietà fondamentale: l'azione inerziale non è mai negativa,
        /// qualunque siano gli stati e i coefficienti non negativi.
        #[test]
        fn inertial_action_mai_negativa(
            v1 in -10.0_f32..10.0, a1 in -10.0_f32..10.0, k1 in -10.0_f32..10.0,
            v2 in -10.0_f32..10.0, a2 in -10.0_f32..10.0, k2 in -10.0_f32..10.0,
            alpha in 0.0_f32..5.0, beta in 0.0_f32..5.0, gamma in 0.0_f32..5.0,
        ) {
            let prev = state(v1, a1, k1);
            let next = state(v2, a2, k2);
            let s = prev.inertial_action(&next, alpha, beta, gamma);
            assert!(s >= 0.0, "azione negativa: {s}");
        }

        /// Proprietà di monotonia: aumentare una componente di deviazione
        /// (a parità delle altre) non può ridurre l'azione.
        #[test]
        fn inertial_action_monotona_in_dv(
            v1 in -10.0_f32..10.0, a in -10.0_f32..10.0, k in -10.0_f32..10.0,
            dv in 0.0_f32..5.0, alpha in 0.0_f32..5.0,
        ) {
            let prev = state(v1, a, k);
            let next_small = state(v1 + dv, a, k);
            let next_big = state(v1 + dv + 1.0, a, k);
            let s_small = prev.inertial_action(&next_small, alpha, 1.0, 1.0);
            let s_big = prev.inertial_action(&next_big, alpha, 1.0, 1.0);
            assert!(s_big >= s_small, "azione decrescente all'aumentare di Δv");
        }
    }
}
pub mod dtw;

pub mod ingest;

pub mod ordered_sparse;

pub mod adapter;

pub mod parse;
