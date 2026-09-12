//! # Test di proprietà: Coerenza Pareto
//!
//! Questa è la **prima pietra** del progetto `semantic-geo`.
//!
//! L'invariante fondamentale: se il fatto A domina il fatto C su tutti gli
//! assi (ogni metrica di A è ≥ quella di C, almeno una strettamente), allora
//! A vince *sempre*, con qualunque combinazione di pesi.
//!
//! `proptest` genera migliaia di combinazioni di metriche e pesi casuali e
//! verifica che l'invariante regga. Se la proprietà è vera per tutti i casi
//! generati, è vera per il combinatore — non per un esempio scelto a mano.

use proptest::prelude::*;
use semantic_combiner::{combine, pareto_compare, NormalizedAxes, ParetoOrder};

/// Genera un vettore di pesi non-negativi che sommano a 1.
fn arb_weights() -> impl Strategy<Value = [f64; 3]> {
    // Tre valori in [0,1], normalizzati per la loro somma.
    // La somma è sempre > 0 (probabilità zero che siano tutti 0.0),
    // quindi i pesi risultanti sono in [0,1] e sommano a 1.
    (0.0f64..1.0, 0.0f64..1.0, 0.0f64..1.0).prop_map(|(a, b, c)| {
        let sum = a + b + c;
        // Se per assurdo la somma fosse 0 (tutti e tre a 0.0), ripiega su pesi equi.
        if sum <= 0.0 {
            return [1.0 / 3.0; 3];
        }
        [a / sum, b / sum, c / sum]
    })
}

/// Genera assi normalizzati validi in `[0, 1]`.
fn arb_axes() -> impl Strategy<Value = NormalizedAxes> {
    (0.0f64..1.0, 0.0f64..1.0, 0.0f64..1.0).prop_map(|(dense, sparse, colbert)| {
        NormalizedAxes {
            dense,
            sparse,
            colbert,
        }
    })
}

proptest! {
    /// **Coerenza Pareto**: se A domina B su tutti gli assi, allora
    /// `combine(A, w) >= combine(B, w)` per qualunque vettore di pesi `w`.
    #[test]
    fn pareto_dominanza_implica_ordine_su_qualunque_peso(
        a in arb_axes(),
        b in arb_axes(),
        w in arb_weights(),
    ) {
        // Forziamo la dominanza: A prende il massimo componente-per-componente
        // rispetto a B. Così A domina B su tutti gli assi per costruzione.
        let a_dominante = NormalizedAxes {
            dense: a.dense.max(b.dense),
            sparse: a.sparse.max(b.sparse),
            colbert: a.colbert.max(b.colbert),
        };

        // A domina B (o è uguale) su tutti gli assi.
        assert!(
            a_dominante.dense >= b.dense
                && a_dominante.sparse >= b.sparse
                && a_dominante.colbert >= b.colbert
        );

        // Quindi il punteggio combinato di A deve essere >= quello di B,
        // per QUALUNQUE vettore di pesi.
        let score_a = combine(&a_dominante, w);
        let score_b = combine(&b, w);

        prop_assert!(score_a >= score_b, "Pareto violato: A domina B ma score_a={score_a} < score_b={score_b} con pesi {w:?}");
    }

    /// **Antisimmetria del confronto Pareto**: il risultato è specchiato.
    ///
    /// Se `pareto_compare(a, b) == ADominatesB` (a domina b), allora
    /// `pareto_compare(b, a) == BDominatesA` (b è dominato da a). E se i due
    /// fatti sono incomparabili in un verso, lo sono anche nell'altro.
    #[test]
    fn pareto_antisimmetrico(
        a in arb_axes(),
        b in arb_axes(),
    ) {
        let ab = pareto_compare(&a, &b);
        let ba = pareto_compare(&b, &a);

        match ab {
            ParetoOrder::ADominatesB => prop_assert_eq!(ba, ParetoOrder::BDominatesA),
            ParetoOrder::BDominatesA => prop_assert_eq!(ba, ParetoOrder::ADominatesB),
            ParetoOrder::Incomparable => prop_assert_eq!(ba, ParetoOrder::Incomparable),
        }
    }

    /// **Punteggio in [0,1]**: il combinatore non esce mai dall'intervallo.
    #[test]
    fn punteggio_sempre_in_01(
        a in arb_axes(),
        w in arb_weights(),
    ) {
        let score = combine(&a, w);
        prop_assert!((0.0..=1.0).contains(&score), "score={score} fuori da [0,1]");
    }

    /// **Monotonicità stretta**: se miglioro un singolo asse di `a` (lasciando
    /// fissi gli altri e `b`), l'ordine non si inverte mai.
    ///
    /// È la controparte graduale del Pareto: il Pareto è l'assoluto ("se sei
    /// superiore su tutto, vinci sempre"), la monotonicità è il graduale ("se
    /// ti muovi nella direzione giusta, non peggiori mai").
    #[test]
    fn monotonicita_stretta(
        a in arb_axes(),
        b in arb_axes(),
        w in arb_weights(),
        delta in 0.0f64..0.5,
    ) {
        let score_b = combine(&b, w);
        let base = combine(&a, w);

        // Miglioriamo un singolo asse di `a` di `delta` (saturato a 1.0).
        let migliorato = NormalizedAxes {
            dense: (a.dense + delta).min(1.0),
            sparse: a.sparse,
            colbert: a.colbert,
        };
        let score_migliorato = combine(&migliorato, w);

        // Migliorando un asse non si peggiora mai rispetto a prima.
        prop_assert!(
            score_migliorato >= base,
            "monotonicità violata: {score_migliorato} < {base}"
        );

        // E se `a` era già davanti a `b`, migliorandolo resta davanti.
        if base >= score_b {
            prop_assert!(
                score_migliorato >= score_b,
                "monotonicità violata rispetto a b: {score_migliorato} < {score_b}"
            );
        }
    }

    /// **Integrità dei pesi nulli**: se un peso è zero, la metrica
    /// corrispondente deve scomparire dal computo senza lasciare traccia.
    ///
    /// Due assi che differiscono *solo* sull'asse il cui peso è zero devono
    /// produrre lo stesso punteggio combinato.
    #[test]
    fn peso_nullo_azzera_linfluenza(
        a in arb_axes(),
        delta in 0.0f64..0.5,
    ) {
        // Peso tutto su dense, zero su sparse e colbert.
        let w = [1.0, 0.0, 0.0];

        // `b` differisce da `a` solo su sparse e colbert (assi a peso zero).
        let b = NormalizedAxes {
            dense: a.dense,
            sparse: (a.sparse + delta).min(1.0),
            colbert: (a.colbert + delta).min(1.0),
        };

        // Il punteggio combinato deve essere identico: gli assi a peso zero
        // non devono influenzare il risultato.
        let score_a = combine(&a, w);
        let score_b = combine(&b, w);
        prop_assert!(
            (score_a - score_b).abs() < 1e-12,
            "peso nullo non rispettato: score_a={score_a} != score_b={score_b}"
        );
    }
}