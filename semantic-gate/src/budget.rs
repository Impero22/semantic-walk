//! # budget — il misuratore di budget
//!
//! Il gate deve decidere **sotto i 10 ms**. Il budget è la deadline entro
//! cui la decisione deve essere presa; se scade, il gate si ritira
//! (permissivo) e segnala `Verdict::Timeout`.
//!
//! ## Zero allocazioni
//!
//! Il misuratore lavora su stack: nessuna allocazione, nessuna syscall.
//! Usa `std::time::Instant`, che è la lettura dell'orologio più economica
//! disponibile in Rust.

use std::time::Instant;

/// Un misuratore di budget basato su una deadline.
///
/// Si costruisce con un budget in nanosecondi; al momento della decisione
/// si verifica se la deadline è scaduta.
pub struct Budget {
    /// La deadline assoluta (istante oltre il quale il budget è esaurito).
    deadline: Instant,
    /// Il budget originale in nanosecondi (per ispezione).
    budget_ns: u64,
}

impl Budget {
    /// Crea un nuovo misuratore di budget.
    ///
    /// * `budget_ns` — il tempo massimo ammesso per la decisione, in
    ///   nanosecondi. Deve essere > 0; se è 0, il budget è già scaduto.
    pub fn new(budget_ns: u64) -> Self {
        let now = Instant::now();
        let deadline = if budget_ns == 0 {
            now // budget zero: deadline immediata (già scaduta)
        } else {
            now + std::time::Duration::from_nanos(budget_ns)
        };
        Budget { deadline, budget_ns }
    }

    /// Il budget originale in nanosecondi.
    pub fn budget_ns(&self) -> u64 {
        self.budget_ns
    }

    /// Il tempo residuo in nanosecondi (0 se la deadline è scaduta).
    pub fn residuo_ns(&self) -> u64 {
        let now = Instant::now();
        if now >= self.deadline {
            0
        } else {
            (self.deadline - now).as_nanos() as u64
        }
    }

    /// `true` se il budget è esaurito (la deadline è scaduta).
    pub fn esaurito(&self) -> bool {
        Instant::now() >= self.deadline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn budget_non_esaurito_subito() {
        let b = Budget::new(1_000_000_000); // 1 secondo
        assert!(!b.esaurito());
        assert!(b.residuo_ns() > 0);
    }

    #[test]
    fn budget_zero_esaurito_subito() {
        let b = Budget::new(0);
        assert!(b.esaurito());
        assert_eq!(b.residuo_ns(), 0);
    }

    #[test]
    fn budget_scade_dopo_attesa() {
        let b = Budget::new(10_000_000); // 10 ms
        thread::sleep(Duration::from_millis(20));
        assert!(b.esaurito());
        assert_eq!(b.residuo_ns(), 0);
    }
}
