//! # integrazione_end_to_end — il banco di prova del cammino completo
//!
//! Test di integrazione che attraversa l'intero percorso del semantic-walk:
//!
//! ```text
//! ingest (CrispTrajectory, D=1024)
//!     → gate (Gate::decide, sonda economica dense+sparse)
//!     → DTW (KinematicAligner::align, distanza coseno normalizzata)
//!     → payload (TrajectoryAlignment: punteggio, token di divergenza)
//! ```
//!
//! Usa vettori **dimensionalmente coerenti con CrispEmbed reale** (D=1024),
//! non vettori a 4 dimensioni dei test unitari. L'obiettivo è validare il
//! comportamento *semantico* del cammino: due traiettorie che descrivono lo
//! stesso concetto devono produrre un punteggio basso, due che divergono un
//! punteggio alto. Questo prepara il banco di prova per quando i vettori
//! reali di CrispEmbed arriveranno: l'aggancio sarà immediato.

use std::time::{Duration, Instant};

use semantic_gate::{Gate, GateConfig, Verdict};
use semantic_walk::ingest::{align_crisp_trajectories, gate_crisp_alignment, CrispTrajectory};

/// La dimensione reale degli embedding CrispEmbed.
const D: usize = 1024;

/// Costruisce un vettore D-dimensionale con un "seme" concettuale.
///
/// Ogni vettore è una base ortonormale sparsa: poche componenti attive su
/// assi canonici diversi a seconda del seme. Vettori con lo stesso seme sono
/// identici (stesso cammino); con semi diversi sono ortogonali (divergenti).
fn vettore(seed: usize, attivo: usize) -> Vec<f64> {
    let mut v = vec![0.0; D];
    // Componenti attive su assi canonici sparsi, con ampiezza normalizzata
    // a norma unitaria. L'asse dipende dal seme e dalla posizione.
    for k in 0..attivo {
        let idx = (seed * 37 + k * 101) % D;
        v[idx] = 1.0;
    }
    // Normalizza a norma unitaria: la distanza coseno dipende solo
    // dall'angolo, non dalla scala.
    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}

/// Una traiettoria "di cammino": una sequenza ordinata di passi concettuali.
///
/// `semi` è la sequenza dei semi dei singoli passi. Due traiettorie con la
/// stessa sequenza di semi sono lo stesso cammino.
fn traiettoria(id: &str, semi: &[usize], passi: usize) -> CrispTrajectory {
    let emb: Vec<Vec<f64>> = semi
        .iter()
        .map(|&s| vettore(s, passi))
        .collect();
    CrispTrajectory::new(id, D, emb).unwrap()
}

/// Un gate permissivo con soglia bassa: lascia passare quasi tutto, così il
/// test verifica il DTW e non il filtro.
fn gate_permissivo() -> Gate {
    Gate::new(GateConfig {
        pesi_dense: 0.5,
        pesi_sparse: 0.5,
        soglia: 0.0, // passa tutto
        budget_ns: 10_000_000,
    })
}

/// Un gate restrittivo con soglia altissima: blocca tutto.
fn gate_restrittivo() -> Gate {
    Gate::new(GateConfig {
        pesi_dense: 0.5,
        pesi_sparse: 0.5,
        soglia: 0.99, // blocca tutto
        budget_ns: 10_000_000,
    })
}

#[test]
fn percorso_completo_traiettorie_identiche() {
    // Due traiettorie identiche: lo stesso cammino semantico.
    let semi = [3, 7, 11, 5];
    let a = traiettoria("fatto_a", &semi, 4);
    let b = traiettoria("fatto_b", &semi, 4);

    // 1. Gate permissivo: passa.
    let gate = gate_permissivo();
    let deadline = Instant::now() + Duration::from_secs(60);
    let res = gate_crisp_alignment(&gate, &a, &b, 1.0, 1.0, deadline, 4).unwrap();

    // 2. Il gate lascia passare e il DTW produce un allineamento.
    let alignment = res.expect("il gate permissivo deve lasciar passare traiettorie identiche");
    assert!(
        alignment.normalized_score < 1e-12,
        "traiettorie identiche: punteggio atteso ~0, ottenuto {}",
        alignment.normalized_score
    );
    assert_eq!(alignment.divergence_token, 0.0);
    assert!(!alignment.warp_path.is_empty());
}

#[test]
fn percorso_completo_traiettorie_divergenti() {
    // Due traiettorie su semi completamente diversi: cammini ortogonali.
    let a = traiettoria("fatto_a", &[1, 2, 3, 4], 4);
    let b = traiettoria("fatto_b", &[50, 60, 70, 80], 4);

    let gate = gate_permissivo();
    let deadline = Instant::now() + Duration::from_secs(60);
    let res = gate_crisp_alignment(&gate, &a, &b, 0.1, 0.1, deadline, 4).unwrap();
    let alignment = res.expect("il gate permissivo lascia passare anche traiettorie divergenti");

    // Traiettorie ortogonali: distanza coseno ~1, punteggio alto.
    // Nota: `divergence_token` misura la divergenza *strutturale* (differenza
    // di lunghezza), non quella semantica — qui le due traiettorie hanno la
    // stessa lunghezza, quindi il token resta 0. La divergenza semantica è
    // catturata dal `normalized_score`.
    assert!(
        alignment.normalized_score > 0.9,
        "traiettorie divergenti: punteggio atteso alto, ottenuto {}",
        alignment.normalized_score
    );
}

#[test]
fn il_dtw_distingue_simile_da_divergente() {
    // Il cuore semantico: il punteggio DTW deve separare i due casi.
    let simili = {
        let a = traiettoria("a", &[2, 4, 6], 3);
        let b = traiettoria("b", &[2, 4, 6], 3);
        align_crisp_trajectories(&a, &b, 3).unwrap().normalized_score
    };
    let divergenti = {
        let a = traiettoria("a", &[2, 4, 6], 3);
        let b = traiettoria("b", &[90, 91, 92], 3);
        align_crisp_trajectories(&a, &b, 3).unwrap().normalized_score
    };

    assert!(
        simili < divergenti,
        "il DTW deve separare: simili={} deve essere < divergenti={}",
        simili,
        divergenti
    );
    assert!(simili < 1e-12, "simili atteso ~0, ottenuto {}", simili);
    assert!(divergenti > 0.9, "divergenti atteso alto, ottenuto {}", divergenti);
}

#[test]
fn il_divergence_token_misura_la_divergenza_strutturale() {
    // `divergence_token` = |n - m| / path_len: cattura la differenza di
    // *lunghezza* tra le traiettorie, non la divergenza semantica dei
    // contenuti. Due cammini di lunghezza diversa devono produrlo positivo.
    let a = traiettoria("a", &[1, 2, 3], 3);
    let b = traiettoria("b", &[1, 2, 3, 4, 5], 5);

    let alignment = align_crisp_trajectories(&a, &b, 3).unwrap();
    assert!(
        alignment.divergence_token > 0.0,
        "lunghezze diverse: token di divergenza atteso positivo, ottenuto {}",
        alignment.divergence_token
    );

    // Due cammini della stessa lunghezza (anche semanticamente diversi)
    // hanno token di divergenza strutturale nullo.
    let c = traiettoria("c", &[1, 2, 3], 3);
    let d = traiettoria("d", &[90, 91, 92], 3);
    let alignment_uguale = align_crisp_trajectories(&c, &d, 3).unwrap();
    assert_eq!(alignment_uguale.divergence_token, 0.0);
}

#[test]
fn il_gate_restrittivo_risparmia_il_dtw() {
    // Un gate restrittivo blocca il confronto prima del DTW: risparmio di costo.
    let a = traiettoria("a", &[1, 2, 3], 3);
    let b = traiettoria("b", &[1, 2, 3], 3);

    let gate = gate_restrittivo();
    let deadline = Instant::now() + Duration::from_secs(60);
    let res = gate_crisp_alignment(&gate, &a, &b, 0.1, 0.1, deadline, 3).unwrap();

    // Anche traiettorie identiche, il gate restrittivo le scarta: il costo
    // del DTW non viene speso per candidati che non meritano.
    assert!(res.is_none(), "il gate restrittivo deve bloccare e non spendere il DTW");
}

#[test]
fn il_gate_permissivo_ritorno_timeout_procede() {
    // Verifica il contratto di ritiro: Verdict::Timeout deve procedere comunque.
    let a = traiettoria("a", &[1, 2, 3], 3);
    let b = traiettoria("b", &[1, 2, 3], 3);

    let gate = gate_permissivo();
    // Deadline già scaduto: il gate si ritira permissivamente (Timeout).
    let deadline = Instant::now() - Duration::from_secs(1);
    let res = gate_crisp_alignment(&gate, &a, &b, 1.0, 1.0, deadline, 3).unwrap();

    // Timeout = riflesso che si ritira, ma il cammino procede comunque.
    let alignment = res.expect("Timeout deve essere permissivo e procedere");
    assert!(alignment.normalized_score < 1e-12);
}

#[test]
fn dimensione_diversa_rifiutata_da_entrambe_le_porte() {
    // Il contratto rigido: due traiettorie con dimensioni diverse sono un
    // dato corrotto, rifiutato sia dall'allineamento diretto sia dal gate.
    let a = CrispTrajectory::new("a", D, vec![vettore(1, 3)]).unwrap();
    let b = CrispTrajectory::new("b", 512, vec![vec![1.0; 512]]).unwrap();

    assert!(align_crisp_trajectories(&a, &b, 2).is_err());

    let gate = gate_permissivo();
    let deadline = Instant::now() + Duration::from_secs(60);
    assert!(gate_crisp_alignment(&gate, &a, &b, 1.0, 1.0, deadline, 2).is_err());
}

#[test]
fn verdetto_gate_diretto_coerente_col_percorso() {
    // Verifica diretta del Verdict del gate, senza il percorso ingest.
    // Serve come sonda di coerenza: il gate da solo decide come atteso.
    let gate = gate_permissivo();
    let deadline = Instant::now() + Duration::from_secs(60);
    assert_eq!(gate.decide(1.0, 1.0, deadline), Verdict::Passa);

    let gate_r = gate_restrittivo();
    assert_eq!(gate_r.decide(0.1, 0.1, deadline), Verdict::Blocca);

    let scaduto = Instant::now() - Duration::from_secs(1);
    assert_eq!(gate.decide(1.0, 1.0, scaduto), Verdict::Timeout);
}
