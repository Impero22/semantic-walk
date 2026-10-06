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
    /// $$S_{Inertial} = m_{sem} c_{sem}^2 (\gamma_{rel} - 1) + \beta \cdot \Delta a^2 + \gamma \cdot \vert \Delta \kappa \vert$$
    ///
    /// con $\Delta v = v_{next} - v_{self}$, $\Delta a = a_{next} - a_{self}$,
    /// $\Delta \kappa = \kappa_{next} - \kappa_{self}$ e
    /// $\gamma_{rel} = 1/\sqrt{1 - (\Delta v/c_{sem})^2}$.
    ///
    /// I coefficienti calibrano la rigidità della camminata:
    ///
    /// * $m_{sem}$ — massa semantica: costo degli scatti di velocità
    ///   (per $\Delta v \ll c_{sem}$ si riduce a $\tfrac{1}{2} m_{sem} \Delta v^2$).
    /// * $\beta$ — costo del jerk (cambi bruschi di accelerazione).
    /// * $\gamma$ — costo delle curve (deviazione dalla geodetica).
    /// * $c_{sem}$ — velocità semantica massima: barriera oltre la quale ogni
    ///   scatto è vietato (l'arco viene abbattuto).
    ///
    /// Il risultato è **sempre non negativo** (somma di quadrati e valori
    /// assoluti): l'azione inerziale è un costo, mai un guadagno.
    ///
    /// Uno stato identico al successivo produce azione zero — il cammino
    /// rettilineo uniforme è la traiettoria più economica, come nella fisica.
    ///
    /// # Inerzia relativistica (Fase 1)
    ///
    /// Il termine di velocità è relativistico: uno scatto di velocità
    /// $\Delta v$ che superi o eguagli la velocità semantica massima
    /// $c_{sem}$ è vietato — l'arco viene abbattuto a `f32::INFINITY`
    /// (guardia PRIMA di ogni aritmetica floating-point, lezione finding #6:
    /// per $\Delta v > c_{sem}$ la quantità $1 - (\Delta v/c_{sem})^2$
    /// diventerebbe negativa e `sqrt` produrrebbe `NaN`, infiltrandosi nella
    /// frontiera e rompendo il determinismo della potatura).
    ///
    /// Il termine relativistico è calcolato sulla **variazione** $\Delta v$,
    /// non sulla velocità assoluta: preserva il principio del cammino
    /// rettilineo uniforme economico (stato identico → azione zero) e la
    /// simmetria del cammino. Per piccoli scatti ($\Delta v \ll c_{sem}$)
    /// si riduce all'energia cinetica classica: il termine
    /// $m_{sem} c_{sem}^2 (\gamma_{rel} - 1) \approx \tfrac{1}{2} m_{sem}
    /// \Delta v^2$ — la massa semantica $m_{sem}$ gioca il ruolo di
    /// coefficiente cinetico (½·m·Δv²), unificando in un solo parametro il
    /// costo classico dello scatto e la barriera relativistica.
    ///
    /// Se `c_sem <= 0.0` (velocità massima non fisica), ogni scatto è
    /// abbattuto: la guardia copre anche questo caso degenere.
    pub fn inertial_action(
        &self,
        next: &KinematicState,
        beta: f32,
        gamma: f32,
        c_sem: f32,
        m_sem: f32,
    ) -> f32 {
        let dv = next.velocity - self.velocity;
        let da = next.acceleration - self.acceleration;
        let dk = next.curvature - self.curvature;

        // Guardia rigida PRIMA di ogni aritmetica: scatto vietato o velocità
        // massima non fisica → arco abbattuto a costo infinito.
        if dv.abs() >= c_sem || c_sem <= 0.0 {
            return f32::INFINITY;
        }

        let ratio = dv / c_sem;
        // Forma numericamente stabile di γ_rel − 1 (evita la cancellazione
        // catastrofica del regime classico in f32):
        //
        //     γ_rel − 1 = r² / (1 + √(1 − r²))    con r = Δv / c_sem
        //
        // Algebricamente identica a 1/√(1−r²) − 1, ma per Δv ≪ c_sem il
        // denominatore → 2 e il termine → ½·r², quindi
        // m·c²·(γ_rel−1) → ½·m·Δv² (energia cinetica classica) senza
        // sottoflow né distorsione in f32.
        let r2 = ratio * ratio;
        let gamma_minus_1 = r2 / (1.0 + (1.0 - r2).sqrt());
        let rel = m_sem * c_sem * c_sem * gamma_minus_1;
        rel + beta * da * da + gamma * dk.abs()
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
        // Con il termine relativistico sulla VARIAZIONE dv, stato identico
        // (dv=0) → γ_rel=1 → termine 0. Principio preservato.
        let s = state(1.0, 0.0, 0.0);
        assert!((s.inertial_action(&s, 1.0, 1.0, 100.0, 1.0)).abs() < 1e-6);
    }

    #[test]
    fn azione_misura_deviazione_di_velocita() {
        // Solo la velocità cambia: l'azione dipende dal termine relativistico.
        // c_sem=100, Δv=2, m=1: γ−1≈0.0002 → m·c²·(γ−1) ≈ 2.0 (non più α·Δv²=8).
        let prev = state(1.0, 0.0, 0.0);
        let next = state(3.0, 0.0, 0.0); // Δv = 2
        let s = prev.inertial_action(&next, 1.0, 1.0, 100.0, 1.0);
        // m·c²·(γ−1) = 2.0002 (β·Δa² = 0, γ·|Δκ| = 0)
        assert!((s - 2.0002).abs() < 1e-3);
    }

    #[test]
    fn azione_misura_deviazione_di_accelerazione() {
        // Solo l'accelerazione cambia: dipende solo da β (jerk).
        let prev = state(0.0, 1.0, 0.0);
        let next = state(0.0, 4.0, 0.0); // Δa = 3
        let s = prev.inertial_action(&next, 2.0, 1.0, 100.0, 1.0);
        // β·Δa² = 2·9 = 18
        assert!((s - 18.0).abs() < 1e-3);
    }

    #[test]
    fn azione_misura_deviazione_di_curvatura() {
        // Solo la curvatura cambia: dipende da γ e dal valore assoluto.
        let prev = state(0.0, 0.0, 1.0);
        let next = state(0.0, 0.0, -2.0); // Δκ = -3, |Δκ| = 3
        let s = prev.inertial_action(&next, 1.0, 4.0, 100.0, 1.0);
        // γ·|Δκ| = 4·3 = 12
        assert!((s - 12.0).abs() < 1e-3);
    }

    #[test]
    fn azione_sempre_non_negativa() {
        // Somma di quadrati e valori assoluti: mai negativa, qualunque input.
        let prev = state(-3.0, 2.5, -1.0);
        let next = state(4.0, -1.5, 0.5);
        let s = prev.inertial_action(&next, 1.3, 2.1, 100.0, 1.0);
        assert!(s >= 0.0);
    }

    #[test]
    fn guardia_scatto_oltre_c_sem_abbattuto_a_infinito() {
        // Lezione finding #6: la guardia PRIMA dell'aritmetica. Uno scatto
        // che supera o eguaglia c_sem è vietato → f32::INFINITY, mai NaN.
        let prev = state(0.0, 0.0, 0.0);
        let next = state(10.0, 0.0, 0.0); // Δv = 10 >= c_sem = 5
        let s = prev.inertial_action(&next, 1.0, 1.0, 5.0, 1.0);
        assert_eq!(s, f32::INFINITY, "scatto a c_sem deve essere abbattuto");
    }

    #[test]
    fn guardia_c_sem_non_fisico_abbattuto_a_infinito() {
        // c_sem <= 0.0 è non fisico: ogni scatto è vietato.
        let prev = state(0.0, 0.0, 0.0);
        let next = state(1.0, 0.0, 0.0);
        assert_eq!(prev.inertial_action(&next, 1.0, 1.0, 0.0, 1.0), f32::INFINITY);
        assert_eq!(prev.inertial_action(&next, 1.0, 1.0, -1.0, 1.0), f32::INFINITY);
    }

    #[test]
    fn termine_relativistico_cresce_con_lo_scatto() {
        // La barriera c_sem: avvicinarsi a c_sem costa sempre di più.
        // Confronto scatti di ampiezza diversa, stessa massa.
        let prev = state(0.0, 0.0, 0.0);
        let c_sem = 10.0;
        let m = 1.0;
        let small = prev.inertial_action(&state(1.0, 0.0, 0.0), 0.0, 0.0, c_sem, m);
        let large = prev.inertial_action(&state(8.0, 0.0, 0.0), 0.0, 0.0, c_sem, m);
        assert!(large > small, "scatto più grande deve costare di più");
        // Divergenza: avvicinandosi a c_sem il costo cresce (forma stabile,
        // divergenza più dolce di 1/√(1−r²) puro: con r=0.99 il termine è
        // ~2.15× quello di r=0.8, non 10× come nel regime α·Δv²).
        let near = prev.inertial_action(&state(9.9, 0.0, 0.0), 0.0, 0.0, c_sem, m);
        assert!(near > large * 2.0, "divergenza vicino a c_sem attesa");
    }

    #[test]
    fn limite_classico_per_piccoli_scatti() {
        // Per Δv << c_sem, m·c²·(γ−1) ≈ ½·m·Δv². Verifica la riduzione
        // all'energia cinetica classica (α gioca il ruolo di ½·m).
        let prev = state(0.0, 0.0, 0.0);
        let next = state(0.01, 0.0, 0.0); // Δv = 0.01, c_sem = 100
        let s = prev.inertial_action(&next, 0.0, 0.0, 100.0, 2.0);
        // ½·m·Δv² = ½·2·(0.01)² = 1e-4
        assert!((s - 1e-4).abs() < 1e-8, "limite classico: {s}");
    }

    proptest::proptest! {
        /// Proprietà fondamentale: l'azione inerziale non è mai negativa,
        /// qualunque siano gli stati e i coefficienti non negativi.
        #[test]
        fn inertial_action_mai_negativa(
            v1 in -10.0_f32..10.0, a1 in -10.0_f32..10.0, k1 in -10.0_f32..10.0,
            v2 in -10.0_f32..10.0, a2 in -10.0_f32..10.0, k2 in -10.0_f32..10.0,
            beta in 0.0_f32..5.0, gamma in 0.0_f32..5.0,
            c_sem in 0.5_f32..10.0, m_sem in 0.0_f32..5.0,
        ) {
            let prev = state(v1, a1, k1);
            let next = state(v2, a2, k2);
            let s = prev.inertial_action(&next, beta, gamma, c_sem, m_sem);
            assert!(s >= 0.0, "azione negativa: {s}");
        }

        /// Proprietà di monotonia: aumentare una componente di deviazione
        /// (a parità delle altre) non può ridurre l'azione.
        #[test]
        fn inertial_action_monotona_in_dv(
            v1 in -10.0_f32..10.0, a in -10.0_f32..10.0, k in -10.0_f32..10.0,
            dv in 0.0_f32..5.0,
        ) {
            let prev = state(v1, a, k);
            let next_small = state(v1 + dv, a, k);
            let next_big = state(v1 + dv + 1.0, a, k);
            let s_small = prev.inertial_action(&next_small, 1.0, 1.0, 100.0, 1.0);
            let s_big = prev.inertial_action(&next_big, 1.0, 1.0, 100.0, 1.0);
            assert!(s_big >= s_small, "azione decrescente all'aumentare di Δv");
        }
    }
}
pub mod dtw;

pub mod ingest;

pub mod ordered_sparse;

pub mod adapter;

pub mod parse;

pub mod frontier;

pub mod solver;

pub mod graph_adapter;

pub mod state;
