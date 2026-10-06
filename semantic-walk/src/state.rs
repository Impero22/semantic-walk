//! # state — la convenzione canonica dello stato cinematico
//!
//! Il modulo che rende **eseguibile** la decisione di contratto presa con
//! Camillo (06/10): `derive_state` è la convenzione canonica per costruire il
//! [`KinematicState`] che alimenta il `BeamSolver`.
//!
//! ## Perché questo modulo esiste
//!
//! Prima di questo modulo, `derive_state` viveva in **due copie identiche**:
//! una nel benchmark `bench_frontier_throughput.rs` e una nel test di
//! integrazione `beam_solver_su_dataset_b.rs`. Il contratto era dichiarato ma
//! **non eseguibile dal codice di produzione** — solo i test e i benchmark
//! lo conoscevano. Questo modulo promuove la convenzione a **API di
//! libreria**: chi costruisce il grafo di prossimità per il beam la usa
//! direttamente, senza duplicare la geometria locale del frame.
//!
//! ## La filosofia (decisione di contratto, 06/10)
//!
//! Lo stato del nodo è derivato dalla **geometria locale del frame**:
//!
//! * `velocity` — la norma del frame (la "forza" del vettore semantico).
//! * `acceleration` — il delta di norma rispetto al frame precedente
//!   (lo sforzo applicato lungo la traiettoria).
//! * `curvature` — l'angolo di deviazione rispetto al frame precedente
//!   (quanto si curva la traiettoria).
//!
//! Questa convenzione **preserva la varianza tra i nodi**: ogni nodo porta
//! la propria dinamica locale reale, senza spalmarla sugli archi adiacenti.
//! È la scelta che vince sulla media delle sonde (ora declassata a strategia
//! secondaria in `graph_adapter`), che appiattiva gli stati rendendo il beam
//! incapace di distinguere le traiettorie.
//!
//! ## Nota sul primo frame
//!
//! Per `i == 0` non esiste un frame precedente: la norma è usata come
//! riferimento a sé stessa (`acceleration = 0`) e la curvatura è nulla
//! (`curvature = 0`). Il primo frame è l'origine della traiettoria — nessuna
//! deviazione possibile.

use crate::KinematicState;

/// La norma euclidea di un frame (vettore semantico denso).
pub fn frame_norm(v: &[f64]) -> f32 {
    let s: f64 = v.iter().map(|x| x * x).sum();
    s.sqrt() as f32
}

/// Il coseno dell'angolo tra due frame.
///
/// Restituisce `0.0` se uno dei due frame ha norma nulla (vettore degenere):
/// in quel caso l'angolo non è definito e la deviazione è convenzionalmente
/// nulla.
pub fn cosine(a: &[f64], b: &[f64]) -> f32 {
    let na = frame_norm(a) as f64;
    let nb = frame_norm(b) as f64;
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    (dot / (na * nb)) as f32
}

/// L'angolo (in radianti) tra due frame.
///
/// Il coseno è clampato a `[-1.0, 1.0]` prima dell'`acos` per robustezza
/// numerica (il prodotto scalare normalizzato può superare di poco i limiti
/// per errori di floating-point).
pub fn angle_between(a: &[f64], b: &[f64]) -> f32 {
    let c = cosine(a, b).clamp(-1.0, 1.0);
    c.acos()
}

/// Deriva lo stato cinematico di un frame dalla **geometria locale**
/// (convenzione canonica del contratto, 06/10).
///
/// * `traj` — la traiettoria completa di cui il frame fa parte.
/// * `i` — l'indice del frame corrente nella traiettoria.
/// * `frame` — il frame corrente (deve essere `traj[i]`).
///
/// La geometria locale è:
///
/// * `velocity = ||frame||` — la norma del frame.
/// * `acceleration = ||frame|| - ||frame_{i-1}||` — il delta di norma
///   (0 per il primo frame).
/// * `curvature = angolo(frame_{i-1}, frame)` — la deviazione angolare
///   (0 per il primo frame).
pub fn derive_state(traj: &[Vec<f64>], i: usize, frame: &[f64]) -> KinematicState {
    let norm = frame_norm(frame);
    let prev_norm = if i > 0 { frame_norm(&traj[i - 1]) } else { norm };
    let acc = norm - prev_norm;
    let curvature = if i > 0 {
        angle_between(&traj[i - 1], frame)
    } else {
        0.0
    };
    KinematicState { velocity: norm, acceleration: acc, curvature }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(xs: &[f64]) -> Vec<f64> {
        xs.to_vec()
    }

    #[test]
    fn primo_frame_origine_senza_deviazione() {
        let traj = vec![v(&[1.0, 0.0])];
        let s = derive_state(&traj, 0, &traj[0]);
        assert_eq!(s.velocity, 1.0);
        assert_eq!(s.acceleration, 0.0);
        assert_eq!(s.curvature, 0.0);
    }

    #[test]
    fn norma_del_frame_e_velocita() {
        let traj = vec![v(&[0.0, 0.0]), v(&[3.0, 4.0])];
        let s = derive_state(&traj, 1, &traj[1]);
        assert!((s.velocity - 5.0).abs() < 1e-5, "norma 3-4-5 attesa, got {}", s.velocity);
    }

    #[test]
    fn accelerazione_delta_di_norma() {
        let traj = vec![v(&[1.0, 0.0]), v(&[3.0, 0.0])];
        let s = derive_state(&traj, 1, &traj[1]);
        assert!((s.acceleration - 2.0).abs() < 1e-5, "acc attesa 2.0, got {}", s.acceleration);
    }

    #[test]
    fn curvatura_angolo_tra_frame() {
        // 90° tra (1,0) e (0,1).
        let traj = vec![v(&[1.0, 0.0]), v(&[0.0, 1.0])];
        let s = derive_state(&traj, 1, &traj[1]);
        assert!((s.curvature - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn frame_degenere_curvatura_pigreco_mezzi() {
        // `cosine` restituisce 0.0 per frame a norma nulla (vettore degenere),
        // quindi `angle_between` produce `acos(0.0) = π/2`. È il comportamento
        // coerente con la funzione `cosine` esistente (già usata dal benchmark
        // e dal test di integrazione): la deviazione di un frame nullo è
        // convenzionalmente "ortogonale", non nulla.
        let traj = vec![v(&[1.0, 0.0]), v(&[0.0, 0.0])];
        let s = derive_state(&traj, 1, &traj[1]);
        assert!(
            (s.curvature - std::f32::consts::FRAC_PI_2).abs() < 1e-5,
            "frame nullo → curvatura π/2 (cosine degenere → 0.0), got {}",
            s.curvature
        );
    }
}