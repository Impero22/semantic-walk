//! # semantic-combiner
//!
//! Il combinatore trivettoriale normalizzato.
//!
//! Combina tre metriche di similarità — **dense**, **sparse** e **colbert** —
//! in un unico punteggio, rispettando l'invariante fondamentale del progetto
//! `semantic-geo`:
//!
//! > **Coerenza Pareto** — se il fatto A domina il fatto C su *tutti* gli assi
//! > (ogni metrica di A è ≥ quella di C), allora A vince *sempre*, con
//! > qualunque combinazione di pesi.
//!
//! Il combinatore è una funzione matematica pura: nessuno stato, nessuna
//! allocazione, nessun I/O. È la prima pietra del progetto.
//!
//! ## Normalizzazione degli assi
//!
//! Le tre metriche nascono su scale diverse:
//!
//! * **dense** — similarità coseno in `[-1, 1]`
//! * **sparse** — similarità in `[0, +inf)` (non limitata superiormente)
//! * **colbert** — similarità media sui token in `[-1, 1]`
//!
//! Per combinarle in modo coerente, le si normalizza tutte in `[0, 1]`:
//!
//! * `dense` e `colbert`: `(x + 1) / 2`
//! * `sparse`: `1 - e^(-λ · s)` (sigmoide esponenziale, `λ > 0`)

/// Struttura che incapsula le tre metriche normalizzate in `[0, 1]`.
///
/// I campi sono pubblici ma i valori sono *garantiti* in `[0, 1]`:
/// la costruzione avviene solo tramite [`NormalizedAxes::normalize`],
/// che applica le trasformazioni e satura i valori fuori range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedAxes {
    /// Similarità densa normalizzata in `[0, 1]`.
    pub dense: f64,
    /// Similarità sparsa normalizzata in `[0, 1]`.
    pub sparse: f64,
    /// Similarità colbert normalizzata in `[0, 1]`.
    pub colbert: f64,
}

impl NormalizedAxes {
    /// Applica le trasformazioni di normalizzazione alle metriche grezze.
    ///
    /// * `dense` / `colbert` in `[-1, 1]` → `(x + 1) / 2`
    /// * `sparse` in `[0, +inf)` → `1 - e^(-λ·s)`
    ///
    /// I valori fuori range vengono saturati in `[0, 1]`, **tranne i `NaN`**.
    ///
    /// * `+inf` → `1.0` — l'asintoto superiore, un limite legittimo.
    /// * `-inf` → `0.0` — l'asintoto inferiore, un limite legittimo.
    /// * `NaN` → **propaga** — non è un valore estremo, è l'assenza di valore.
    ///   Saturarlo a `0.0` sarebbe un giudizio fabbricato di lontananza; il
    ///   combinatore non decide per l'ignoto, lo segnala. La geometria che
    ///   non sa rispondere non mente: si ritira.
    pub fn normalize(
        dense: f64,
        sparse: f64,
        colbert: f64,
        sparse_lambda: f64,
    ) -> Self {
        let dense = saturate01((dense + 1.0) / 2.0);
        let sparse = saturate01(1.0 - (-sparse_lambda * sparse).exp());
        let colbert = saturate01((colbert + 1.0) / 2.0);
        NormalizedAxes {
            dense,
            sparse,
            colbert,
        }
    }
}

/// Satura un valore in `[0, 1]`, **preservando i `NaN`**.
///
/// * `+inf` → `1.0` — un valore infinitamente grande è il massimo possibile.
/// * `-inf` → `0.0` — un valore infinitamente piccolo è il minimo possibile.
/// * `NaN` → resta `NaN` — l'assenza di valore non è un estremo, è un
///   "non so". Saturarlo a `0.0` trasformerebbe l'incertezza in un falso
///   giudizio di lontananza. L'ignoto viaggia e chi lo consuma si ritira.
fn saturate01(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if x.is_infinite() {
        return if x.is_sign_positive() { 1.0 } else { 0.0 };
    }
    x.clamp(0.0, 1.0)
}

/// Il risultato di un confronto Pareto tra due fatti.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParetoOrder {
    /// `a` domina `b` su tutti gli assi (almeno uno strettamente).
    ADominatesB,
    /// `b` domina `a` su tutti gli assi (almeno uno strettamente).
    BDominatesA,
    /// Nessuna dominanza: i due fatti sono incomparabili (o uguali).
    Incomparable,
}

/// Confronta due punti secondo la dominanza di Pareto.
///
/// Il confronto è binario: `lhs` e `rhs` sono due assi normalizzati qualsiasi,
/// senza un centro comune (il caso "rispetto a un centro B" appartiene al
/// grafo, dove il combinatore arriva già sanificato). `lhs` domina `rhs` se
/// `lhs.dense >= rhs.dense`, `lhs.sparse >= rhs.sparse` e
/// `lhs.colbert >= rhs.colbert`, con almeno una disuguaglianza stretta.
pub fn pareto_compare(lhs: &NormalizedAxes, rhs: &NormalizedAxes) -> ParetoOrder {
    let lhs_ge_rhs =
        lhs.dense >= rhs.dense && lhs.sparse >= rhs.sparse && lhs.colbert >= rhs.colbert;
    let rhs_ge_lhs =
        rhs.dense >= lhs.dense && rhs.sparse >= lhs.sparse && rhs.colbert >= lhs.colbert;

    let lhs_gt_rhs =
        lhs.dense > rhs.dense || lhs.sparse > rhs.sparse || lhs.colbert > rhs.colbert;
    let rhs_gt_lhs =
        rhs.dense > lhs.dense || rhs.sparse > lhs.sparse || rhs.colbert > lhs.colbert;

    match (lhs_ge_rhs, rhs_ge_lhs, lhs_gt_rhs, rhs_gt_lhs) {
        (true, _, true, _) => ParetoOrder::ADominatesB,
        (_, true, _, true) => ParetoOrder::BDominatesA,
        _ => ParetoOrder::Incomparable,
    }
}

/// Combina le tre metriche normalizzate in un punteggio unico.
///
/// I pesi devono essere non-negativi e sommare a 1. Il risultato è in `[0, 1]`.
///
/// ## Invariante di coerenza Pareto
///
/// Se `a` domina `b` su tutti gli assi, allora per *qualunque* vettore di pesi
/// validi `combine(a, w) >= combine(b, w)`, con disuguaglianza stretta se la
/// dominanza è stretta. Questa proprietà è testata da [`proptest`] in
/// `tests/pareto.rs`.
pub fn combine(axes: &NormalizedAxes, weights: [f64; 3]) -> f64 {
    debug_assert!(
        weights.iter().all(|w| w.is_finite() && *w >= 0.0),
        "i pesi devono essere finiti e non-negativi"
    );
    let sum: f64 = weights.iter().sum();
    debug_assert!(
        (sum - 1.0).abs() < 1e-9,
        "i pesi devono sommare a 1 (somma = {sum})"
    );

    axes.dense * weights[0] + axes.sparse * weights[1] + axes.colbert * weights[2]
}

/// Pesi calibrati congiuntamente dal corpus CSV (p95 @ 90% saturazione).
///
/// Calibrati insieme a [`LAMBDA_CALIBRATO`]: il canale sparse — prima
/// schiacciato a ~3.5% dal lambda troppo basso — è ora il più discriminante.
pub const PESI_CALIBRATI: [f64; 3] = [0.215, 0.552, 0.233];

/// Lambda calibrato per la saturazione esponenziale dello sparse.
///
/// Con λ = 10.64, il p95 della distribuzione satura al 90%: la scala dello
/// sparse viene usata per intero, invece di essere schiacciata in [0, 0.05].
pub const LAMBDA_CALIBRATO: f64 = 10.64;

/// Factory: il combinatore con i pesi calibrati di default.
///
/// Ritorna i pesi tricanale come array, pronto per [`combine`]. Il lambda
/// calibrato è [`LAMBDA_CALIBRATO`], da passare a [`NormalizedAxes::normalize`].
pub fn combiner_tricanale() -> [f64; 3] {
    PESI_CALIBRATI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_mappa_in_01() {
        // dense e colbert: [-1,1] -> [0,1]
        let axes = NormalizedAxes::normalize(1.0, 0.0, 1.0, 1.0);
        assert!((axes.dense - 1.0).abs() < 1e-12);
        assert!((axes.colbert - 1.0).abs() < 1e-12);
        // sparse: 1 - e^(-1*0) = 0
        assert!(axes.sparse.abs() < 1e-12);

        let axes = NormalizedAxes::normalize(-1.0, 0.0, -1.0, 1.0);
        assert!(axes.dense.abs() < 1e-12);
        assert!(axes.colbert.abs() < 1e-12);
    }

    #[test]
    fn sparse_satura_asintoticamente_a_1() {
        // sparse molto grande -> 1 - e^(-λ·s) -> 1
        let axes = NormalizedAxes::normalize(0.0, 100.0, 0.0, 1.0);
        assert!((axes.sparse - 1.0).abs() < 1e-12);
    }

    #[test]
    fn inf_satura_agli_asintoti() {
        // +inf -> 1.0 (asintoto superiore), -inf -> 0.0 (asintoto inferiore).
        let axes = NormalizedAxes::normalize(f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, 1.0);
        assert_eq!(axes.dense, 1.0);
        assert_eq!(axes.sparse, 0.0);
        assert_eq!(axes.colbert, 1.0);
    }

    #[test]
    fn nan_propaga_come_ignoto() {
        // NaN non è un estremo: è l'assenza di valore. Non si satura a 0.0
        // (sarebbe un giudizio fabbricato di lontananza) ma propaga come
        // "non so", perché chi lo consuma possa ritirarsi onestamente.
        let axes = NormalizedAxes::normalize(f64::NAN, f64::NAN, f64::NAN, 1.0);
        assert!(axes.dense.is_nan());
        assert!(axes.sparse.is_nan());
        assert!(axes.colbert.is_nan());

        // Un singolo asse ignoto non contagia gli altri.
        let axes = NormalizedAxes::normalize(0.5, f64::NAN, 0.5, 1.0);
        assert!((axes.dense - 0.75).abs() < 1e-12); // (0.5+1)/2 = 0.75
        assert!(axes.sparse.is_nan());
        assert!((axes.colbert - 0.75).abs() < 1e-12);
    }

    #[test]
    fn pareto_dominanza_semplice() {
        let a = NormalizedAxes {
            dense: 0.9,
            sparse: 0.8,
            colbert: 0.7,
        };
        let b = NormalizedAxes {
            dense: 0.5,
            sparse: 0.5,
            colbert: 0.5,
        };
        assert_eq!(pareto_compare(&a, &b), ParetoOrder::ADominatesB);
        assert_eq!(pareto_compare(&b, &a), ParetoOrder::BDominatesA);
    }

    #[test]
    fn pareto_incomparabile() {
        // a vince su dense ma perde su sparse
        let a = NormalizedAxes {
            dense: 0.9,
            sparse: 0.1,
            colbert: 0.5,
        };
        let b = NormalizedAxes {
            dense: 0.1,
            sparse: 0.9,
            colbert: 0.5,
        };
        assert_eq!(pareto_compare(&a, &b), ParetoOrder::Incomparable);
    }

    #[test]
    fn combine_rispetta_pesi() {
        let axes = NormalizedAxes {
            dense: 1.0,
            sparse: 0.0,
            colbert: 0.0,
        };
        // tutto su dense
        let score = combine(&axes, [1.0, 0.0, 0.0]);
        assert!((score - 1.0).abs() < 1e-12);

        // tutto su sparse (che è 0)
        let score = combine(&axes, [0.0, 1.0, 0.0]);
        assert!(score.abs() < 1e-12);
    }
}
#[cfg(test)]
mod tests_calibrati {
    use super::*;

    #[test]
    fn factory_ritorna_pesi_calibrati() {
        let w = combiner_tricanale();
        assert!((w[0] - 0.215).abs() < 1e-9);
        assert!((w[1] - 0.552).abs() < 1e-9);
        assert!((w[2] - 0.233).abs() < 1e-9);
        assert!((LAMBDA_CALIBRATO - 10.64).abs() < 1e-9);
    }
}

/// Identificatore univoco di un fatto o nodo nel grafo semantico.
pub type FactId = u64;

/// Alias per il punteggio trivettoriale combinato.
pub type TrivectorScore = f64;
