//! # sonda — la combinazione economica dense+sparse
//!
//! Il gate usa **solo le due metriche economiche** — dense e sparse — per
//! decidere se vale la pena spendere il matching completo (colbert).
//!
//! Il colbert è posto a `0` nel calcolo economico: non è ancora stato
//! calcolato, e il gate non deve indovinarlo. È il riflesso che filtra,
//! non il giudice che condanna.
//!
//! ## Zero allocazioni
//!
//! Questa sonda lavora su stack: nessuna `Vec`, nessuna `String`, nessun
//! `Box` nel percorso critico. Tutto è `f64` su stack e un buffer di pesi
//! preallocato passato per riferimento.

use semantic_combiner::{combine, NormalizedAxes};

/// Combina le due sonde economiche (dense, sparse) in un punteggio unico.
///
/// Il colbert è posto a `0`: non è ancora stato calcolato, e il gate non
/// deve indovinarlo. I pesi della sonda (`pesi_dense`, `pesi_sparse`)
/// devono essere non-negativi e sommare a 1 (insieme al peso colbert, che
/// in questa fase è implicitamente `0`).
///
/// ## Ritiro per onestà
///
/// Se i pesi non sono validi (negativi, o somma non unitaria), la sonda
/// non azzarda un giudizio: ritorna `f64::NAN`, e il gate interpreterà il
/// `NaN` come *permissivo* (lascia passare). La geometria che non sa
/// rispondere non mente: si ritira.
pub fn sonda_economica(
    dense: f64,
    sparse: f64,
    pesi_dense: f64,
    pesi_sparse: f64,
) -> f64 {
    // Validazione dei pesi: devono essere finiti, non-negativi e sommare a 1
    // (insieme al peso colbert, implicito a 0 in questa fase).
    let pesi_validi = pesi_dense.is_finite()
        && pesi_sparse.is_finite()
        && pesi_dense >= 0.0
        && pesi_sparse >= 0.0
        && (pesi_dense + pesi_sparse - 1.0).abs() < 1e-9;

    if !pesi_validi {
        return f64::NAN;
    }

    // Il colbert è posto a 0: non è ancora stato calcolato.
    // Il lambda calibrato (10.64) è lo stesso del giudizio completo: il
    // riflesso economico e il giudizio parlano la stessa lingua.
    let axes = NormalizedAxes::normalize(dense, sparse, 0.0, semantic_combiner::LAMBDA_CALIBRATO);
    combine(&axes, [pesi_dense, pesi_sparse, 0.0])
}

/// Verifica se un punteggio economico è *valido* (non `NaN`).
///
/// Il gate usa questa funzione per decidere se può fidarsi del punteggio:
/// un `NaN` significa che la sonda si è ritirata, e il gate deve lasciare
/// passare (permissivo).
pub fn punteggio_valido(score: f64) -> bool {
    !score.is_nan()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sonda_rispetta_pesi() {
        // Tutto su dense: dense alto -> punteggio alto.
        let score = sonda_economica(1.0, 0.0, 1.0, 0.0);
        assert!((score - 1.0).abs() < 1e-12);

        // Tutto su sparse: sparse alto -> punteggio alto.
        let score = sonda_economica(0.0, 100.0, 0.0, 1.0);
        assert!((score - 1.0).abs() < 1e-12);
    }

    #[test]
    fn sonda_pesi_invalidi_si_ritira() {
        // Pesi negativi: NaN (permissivo).
        let score = sonda_economica(1.0, 1.0, -1.0, 0.0);
        assert!(score.is_nan());

        // Somma non unitaria (0.7 + 0.7 = 1.4): NaN.
        let score = sonda_economica(1.0, 1.0, 0.7, 0.7);
        assert!(score.is_nan());
    }

    #[test]
    fn sonda_colbert_ignorato() {
        // Il colbert è sempre 0 nella sonda: non deve influenzare.
        // dense=0 → normalizzato 0.5, sparse=0 → 0.0, colbert=0 → 0.5.
        // Con pesi 50/50 (colbert a 0): 0.5*0.5 + 0.5*0.0 = 0.25.
        let score = sonda_economica(0.0, 0.0, 0.5, 0.5);
        assert!((score - 0.25).abs() < 1e-12);

        // dense massimo (→1.0), sparse minimo (→0.0): 0.5*1.0 + 0.5*0.0 = 0.5.
        let score = sonda_economica(1.0, 0.0, 0.5, 0.5);
        assert!((score - 0.5).abs() < 1e-12);
    }

    #[test]
    fn sonda_nan_da_input_si_ritira() {
        // Un NaN in ingresso (es. coseno di un vettore nullo) non è un valore
        // estremo: è l'assenza di valore. Il combinatore ora lo lascia
        // propagare invece di saturarlo a 0.0, quindi la sonda lo riceve e
        // si ritira (NaN). Il gate interpreterà questo come permissivo.
        let score = sonda_economica(f64::NAN, 0.0, 0.5, 0.5);
        assert!(score.is_nan());

        // Anche un NaN su sparse fa ritirare la sonda.
        let score = sonda_economica(1.0, f64::NAN, 0.5, 0.5);
        assert!(score.is_nan());
    }
}
