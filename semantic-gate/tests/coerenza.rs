//! # Test di proprietà del gate — coerenza con il combinatore
//!
//! La proprietà fondamentale del gate, dal design:
//!
//! > **Il gate non blocca mai un candidato che domina la soglia su entrambe
//! > le sonde** (coerenza con il combinatore).
//!
//! In pratica: se un candidato ha un punteggio economico *almeno* pari alla
//! soglia, il gate deve lasciarlo passare. Se il punteggio è *strettamente*
//! sopra la soglia, il gate deve sempre passarlo, con qualunque coppia di
//! sonde.
//!
//! Questa proprietà è testata con `proptest` su migliaia di casi generati,
//! esattamente come la coerenza Pareto del combinatore.

use proptest::prelude::*;
use semantic_gate::{Gate, GateConfig, Verdict};
use std::time::{Duration, Instant};

/// Genera una coppia di sonde (dense, sparse) in range realistici.
fn arb_sonde() -> impl Strategy<Value = (f64, f64)> {
    // dense in [-1, 1], sparse in [0, 10] (sparse oltre ~10 satura a ~1)
    (-1.0f64..=1.0, 0.0f64..=10.0)
}

/// Genera una configurazione di gate valida (pesi 50/50, soglia in [0,1]).
fn arb_gate() -> impl Strategy<Value = Gate> {
    (0.0f64..=1.0).prop_map(|soglia| {
        Gate::new(GateConfig {
            pesi_dense: 0.5,
            pesi_sparse: 0.5,
            soglia,
            budget_ns: 10_000_000,
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Se il punteggio economico è ≥ soglia, il gate passa SEMPRE.
    /// (Con pesi 50/50 e colbert a 0, il punteggio è (dense_n + sparse_n)/2.)
    #[test]
    fn sopra_soglia_non_blocca_mai(
        (dense, sparse) in arb_sonde(),
        soglia in 0.0f64..=1.0,
    ) {
        let g = Gate::new(GateConfig {
            pesi_dense: 0.5,
            pesi_sparse: 0.5,
            soglia,
            budget_ns: 10_000_000,
        });

        // Normalizza come fa la sonda: dense (x+1)/2, sparse 1-e^(-s).
        let dense_n = (dense + 1.0) / 2.0;
        let sparse_n = 1.0 - (-sparse).exp();
        let score = 0.5 * dense_n + 0.5 * sparse_n;

        // Se il punteggio è sopra la soglia, il verdetto NON è mai Blocca.
        // (Può essere Passa o Timeout, mai Blocca.)
        if score >= soglia {
            let deadline = Instant::now() + Duration::from_secs(60);
            let verdetto = g.decide(dense, sparse, deadline);
            prop_assert_ne!(verdetto, Verdict::Blocca);
        }
    }

    /// Con soglia a 0, tutto passa (nessun falso negativo) — permissività.
    #[test]
    fn soglia_zero_permissivo((dense, sparse) in arb_sonde()) {
        let g = Gate::new(GateConfig {
            pesi_dense: 0.5,
            pesi_sparse: 0.5,
            soglia: 0.0,
            budget_ns: 10_000_000,
        });
        let deadline = Instant::now() + Duration::from_secs(60);
        prop_assert_eq!(g.decide(dense, sparse, deadline), Verdict::Passa);
    }

    /// Il verdetto non è mai Blocca quando la sonda è in alto su entrambi
    /// gli assi (coerenza con la dominanza Pareto del combinatore).
    #[test]
    fn dominanza_su_entrambe_le_sonde_non_blocca(gate in arb_gate()) {
        // dense e sparse entrambe al massimo → punteggio 1.0 ≥ qualsiasi soglia
        let deadline = Instant::now() + Duration::from_secs(60);
        prop_assert_ne!(gate.decide(1.0, 100.0, deadline), Verdict::Blocca);
    }
}
