//! # semantic-gate — il riflesso pre-inferenziale
//!
//! Il gate sta **prima** dell'inferenza vera e propria, e risponde a una
//! domanda economica:
//!
//! > Dato un input e un candidato (un fatto della memoria), *vale la pena*
//! > spendere il matching completo per questo candidato?
//!
//! Il matching completo — la similarità colbert, che confronta token per
//! token con MaxSim — è il costo alto. Il gate è la prima linea: veloce,
//! parziale, che **non mente ma si ritira**.
//!
//! ## Le tre metriche, una gerarchia di costo
//!
//! | metrica | costo | ruolo |
//! |---------|-------|-------|
//! | dense   | basso  | sonda primaria |
//! | sparse  | medio  | sonda secondaria |
//! | colbert | alto   | matching completo (dopo il gate) |
//!
//! Il gate usa **solo dense e sparse** — le metriche economiche — per
//! decidere. Il colbert resta al matching completo, che il gate alimenta.
//!
//! ## L'onestà del riflesso
//!
//! Il gate è **permissivo** per natura: meglio far passare un candidato che
//! il matching completo poi scarterà (spreco di un colbert) che bloccare un
//! candidato che sarebbe stato rilevante (perdita di un ricordo).
//!
//! > Meglio un riflesso parziale che nessun riflesso — e meglio un colbert
//! > sprecato che un ricordo perso.
//!
//! Il `Verdict::Timeout` è il riflesso che si ritira: se il budget scade,
//! non si azzarda un giudizio — si lascia passare (permissivo) e si segnala
//! che il budget non è bastato.

mod budget;
mod sonda;

use std::time::Instant;

pub use budget::Budget;
pub use sonda::{punteggio_valido, sonda_economica};

/// Configurazione del gate.
///
/// I pesi della sonda (quanto pesa dense vs sparse) e la soglia sono
/// parametri calibrati — probabilmente da dati reali, quando la memoria
/// sarà popolata.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GateConfig {
    /// Peso della sonda densa nella combinazione economica.
    pub pesi_dense: f64,
    /// Peso della sonda sparsa nella combinazione economica.
    pub pesi_sparse: f64,
    /// Soglia in `[0, 1]`: sopra si passa, sotto si blocca.
    pub soglia: f64,
    /// Budget per la decisione, in nanosecondi.
    pub budget_ns: u64,
}

impl Default for GateConfig {
    fn default() -> Self {
        GateConfig {
            pesi_dense: 0.5,
            pesi_sparse: 0.5,
            soglia: 0.5,
            budget_ns: 10_000_000, // 10 ms
        }
    }
}

/// L'esito di una decisione del gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Vai al matching completo (colbert): il candidato merita il costo.
    Passa,
    /// Scarta: non spendere il colbert per questo candidato.
    Blocca,
    /// Budget esaurito: il riflesso si ritira (permissivo).
    Timeout,
}

/// Il gate pre-inferenziale.
///
/// Immutabile: la configurazione è fissa e la decisione è una funzione
/// pura dello stato. Nessuna allocazione nel percorso critico.
#[derive(Debug, Clone, Copy)]
pub struct Gate {
    config: GateConfig,
}

impl Gate {
    /// Costruisce un gate con la configurazione data.
    pub fn new(config: GateConfig) -> Self {
        Gate { config }
    }

    /// Costruisce un gate con la configurazione di default.
    pub fn default() -> Self {
        Gate {
            config: GateConfig::default(),
        }
    }

    /// La configurazione del gate.
    pub fn config(&self) -> &GateConfig {
        &self.config
    }

    /// Decide se vale la pena spendere il matching completo per un candidato.
    ///
    /// * `dense` — similarità densa grezza in `[-1, 1]`.
    /// * `sparse` — similarità sparsa grezza in `[0, +inf)`.
    /// * `deadline` — l'istante entro cui la decisione deve essere presa.
    ///
    /// ## Semantica del verdetto
    ///
    /// * **Passa** — il punteggio economico è ≥ soglia: vai al colbert.
    /// * **Blocca** — il punteggio economico è < soglia: non spendere.
    /// * **Timeout** — il budget è scaduto, o la sonda si è ritirata
    ///   (pesi invalidi → `NaN`): lascia passare (permissivo) e segnala.
    ///
    /// Il gate è **permissivo**: ogni incertezza (budget scaduto, sonda
    /// ritirata) risolve in *Passa*, mai in *Blocca*. Meglio un colbert
    /// sprecato che un ricordo perso.
    pub fn decide(&self, dense: f64, sparse: f64, deadline: Instant) -> Verdict {
        // Il budget è la prima guardia: se è già scaduto, ci si ritira
        // subito, senza nemmeno calcolare la sonda.
        if Instant::now() >= deadline {
            return Verdict::Timeout;
        }

        // La sonda economica: dense + sparse, colbert a 0.
        let score = sonda_economica(
            dense,
            sparse,
            self.config.pesi_dense,
            self.config.pesi_sparse,
        );

        // Verifica il budget *dopo* il calcolo della sonda: se è scaduto
        // mentre calcolavamo, ci si ritira (permissivo).
        if Instant::now() >= deadline {
            return Verdict::Timeout;
        }

        // Se la sonda si è ritirata (pesi invalidi → NaN), si è permissivi:
        // si lascia passare, perché non si ha un giudizio affidabile.
        if !punteggio_valido(score) {
            return Verdict::Passa;
        }

        if score >= self.config.soglia {
            Verdict::Passa
        } else {
            Verdict::Blocca
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn gate() -> Gate {
        Gate::new(GateConfig {
            pesi_dense: 0.5,
            pesi_sparse: 0.5,
            soglia: 0.5,
            budget_ns: 10_000_000,
        })
    }

    fn deadline_futuro() -> Instant {
        Instant::now() + Duration::from_secs(60)
    }

    #[test]
    fn soglia_zero_tutto_passa() {
        // Permissività: con soglia a 0, nessun falso negativo.
        let g = Gate::new(GateConfig {
            soglia: 0.0,
            ..GateConfig::default()
        });
        assert_eq!(g.decide(0.0, 0.0, deadline_futuro()), Verdict::Passa);
        assert_eq!(g.decide(-1.0, 0.0, deadline_futuro()), Verdict::Passa);
    }

    #[test]
    fn sopra_soglia_passa() {
        let g = gate();
        // dense=1 (→1.0), sparse=0 (→0.0), pesi 50/50 → 0.5 ≥ 0.5 → Passa
        assert_eq!(g.decide(1.0, 0.0, deadline_futuro()), Verdict::Passa);
    }

    #[test]
    fn sotto_soglia_blocca() {
        let g = gate();
        // dense=0 (→0.5), sparse=0 (→0.0), pesi 50/50 → 0.25 < 0.5 → Blocca
        assert_eq!(g.decide(0.0, 0.0, deadline_futuro()), Verdict::Blocca);
    }

    #[test]
    fn budget_scaduto_timeout() {
        let g = gate();
        // Deadline nel passato: budget già scaduto → Timeout (permissivo).
        let passato = Instant::now() - Duration::from_secs(1);
        assert_eq!(g.decide(0.0, 0.0, passato), Verdict::Timeout);
    }

    #[test]
    fn sonda_ritirata_permissiva() {
        // Pesi invalidi (somma non unitaria) → sonda NaN → permissivo Passa.
        let g = Gate::new(GateConfig {
            pesi_dense: 0.5,
            pesi_sparse: 0.5,
            soglia: 0.5,
            budget_ns: 10_000_000,
        });
        // Qui i pesi sono validi (0.5+0.5=1), quindi la sonda non si ritira.
        // Per testare il ritiro, servono pesi invalidi, ma il gate li
        // accetta solo se passati in config. Il gate non valida i pesi in
        // configurazione (lo fa la sonda). Quindi costruiamo un gate con
        // pesi invalidi direttamente.
        let g_invalido = Gate::new(GateConfig {
            pesi_dense: 0.7,
            pesi_sparse: 0.7, // somma 1.4 ≠ 1 → sonda NaN
            soglia: 0.5,
            budget_ns: 10_000_000,
        });
        assert_eq!(
            g_invalido.decide(0.0, 0.0, deadline_futuro()),
            Verdict::Passa
        );
        // Il gate valido non è toccato.
        assert_eq!(g.decide(0.0, 0.0, deadline_futuro()), Verdict::Blocca);
    }

    #[test]
    fn input_nan_permissivo() {
        // Un input NaN (es. coseno di un vettore nullo) non è un valore
        // estremo: è l'assenza di valore. Prima della correzione, il
        // combinatore lo saturava a 0.0 e il gate bloccava — un falso
        // giudizio di lontananza. Ora il NaN propaga fino alla sonda, che
        // si ritira, e il gate risponde permissivo: Passa, mai Blocca.
        // Meglio un colbert sprecato che un ricordo perso.
        let g = gate();
        assert_eq!(g.decide(f64::NAN, 0.0, deadline_futuro()), Verdict::Passa);
        assert_eq!(g.decide(0.0, f64::NAN, deadline_futuro()), Verdict::Passa);
    }
}
