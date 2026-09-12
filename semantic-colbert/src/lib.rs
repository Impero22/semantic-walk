//! # semantic-colbert — la stanza dell'interazione tardiva
//!
//! Questo crate implementa l'operatore **MaxSim** per il reranking ad alta
//! precisione basato su architettura ColBERT (Late Interaction).
//!
//! A differenza dei modelli single-vector che comprimono l'intero testo in
//! un singolo punto nello spazio semantico, ColBERT preserva la matrice dei
//! vettori dei singoli token $Q \in \mathbb{R}^{N \times d}$ (query) e
//! $D \in \mathbb{R}^{M \times d}$ (documento).
//!
//! ## Operatore MaxSim
//!
//! Per ogni token della query $q_i$, si individua il massimo prodotto scalare
//! con i token del documento $d_j$. Il punteggio finale è la media sui token
//! della query, normalizzata nell'intervallo `[-1, 1]`:
//!
//! $$S(Q, D) = \frac{1}{N} \sum_{i=1}^{N} \max_{j=1}^{M} \left( \frac{q_i \cdot d_j}{\|q_i\|_2 \|d_j\|_2} \right)$$
//!
//! ## Invarianti e Garanzie
//!
//! * **Zero Allocazioni**: L'elaborazione opera direttamente su slice `&[&[f64]]`
//!   o buffer contigui nello stack/heap senza allocare vettori temporanei.
//! * **Onestà Geometrica**: Input vuoti, dimensioni incompatibili o valori `NaN`
//!   fanno ritirare l'operatore restituendo `f64::NAN`, permettendo al
//!   `semantic-gate` e al `semantic-combiner` di gestire l'incertezza in modo
//!   permissivo.
//!
//! ## Strato FFI e Trait Abstraction (piano condiviso con Camillo, 13/09/26)
//!
//! Per collegare il motore C++ **CrispEmbed** senza intaccare il percorso
//! critico, questo crate espone:
//!
//! * **`ColbertScorer`** — trait che astrae la sorgente dei punteggi MaxSim.
//!   Il provider nativo (`NativeMaxSim`) implementa l'operatore in Rust puro;
//!   il provider FFI (`CrispEmbedScorer`, in `ffi.rs`) delega al motore C++.
//! * **`maxsim`** — resta la free function nativa (il proxy attuale), usata
//!   dai test e dal percorso non iniettato.
//!
//! Il contratto è identico per ogni implementazione: input `&[&[f64]]`,
//! output in `[-1.0, 1.0]`, `f64::NAN` per input invalidi (onestà geometrica).

pub mod ffi;

/// Calcola la similarità MaxSim tra le matrici di token della query e del documento.
///
/// * `query_tokens` — Matrice $N \times d$ dei vettori di token della query.
/// * `doc_tokens` — Matrice $M \times d$ dei vettori di token del documento.
///
/// Restituisce un punteggio in `[-1.0, 1.0]`. Se l'input è invalido (matrici vuote,
/// dimensioni disallineate o `NaN`), restituisce `f64::NAN`.
pub fn maxsim(query_tokens: &[&[f64]], doc_tokens: &[&[f64]]) -> f64 {
    if query_tokens.is_empty() || doc_tokens.is_empty() {
        return f64::NAN;
    }

    let dim = query_tokens[0].len();
    if dim == 0 {
        return f64::NAN;
    }

    // Verifica consistenza delle dimensioni su tutti i token
    for q in query_tokens {
        if q.len() != dim {
            return f64::NAN;
        }
    }
    for d in doc_tokens {
        if d.len() != dim {
            return f64::NAN;
        }
    }

    let mut somma_massimi = 0.0;

    for q in query_tokens {
        let norm_q = norm_l2(q);
        if norm_q == 0.0 || norm_q.is_nan() {
            return f64::NAN;
        }

        let mut max_sim = f64::NEG_INFINITY;

        for d in doc_tokens {
            let norm_d = norm_l2(d);
            if norm_d == 0.0 || norm_d.is_nan() {
                return f64::NAN;
            }

            let dot = dot_product(q, d);
            if dot.is_nan() {
                return f64::NAN;
            }

            let cos_sim = (dot / (norm_q * norm_d)).clamp(-1.0, 1.0);
            if cos_sim > max_sim {
                max_sim = cos_sim;
            }
        }

        if max_sim.is_infinite() || max_sim.is_nan() {
            return f64::NAN;
        }

        somma_massimi += max_sim;
    }

    somma_massimi / (query_tokens.len() as f64)
}

/// Astrae la sorgente del punteggio MaxSim.
///
/// Il trait definisce il **contratto** tra l'ecosistema Rust e qualsiasi
/// motore di Late Interaction — oggi il provider nativo, domani la FFI C++
/// di CrispEmbed. Ogni implementazione deve rispettare l'onestà geometrica:
/// input invalidi → `f64::NAN`, output in `[-1.0, 1.0]`.
pub trait ColbertScorer: std::fmt::Debug {
    /// Calcola la MaxSim tra le matrici di token di query e documento.
    ///
    /// * `query_tokens` — Matrice $N \times d$ dei vettori di token della query.
    /// * `doc_tokens` — Matrice $M \times d$ dei vettori di token del documento.
    ///
    /// Restituisce un punteggio in `[-1.0, 1.0]`, oppure `f64::NAN` se
    /// l'input è invalido (matrici vuote, dimensioni disallineate, `NaN`).
    fn compute_maxsim(&self, query_tokens: &[&[f64]], doc_tokens: &[&[f64]]) -> f64;
}

/// Provider nativo: implementa `ColbertScorer` delegando alla free function
/// `maxsim`. È il percorso attuale (proxy), usato dai test e dal flusso non
/// iniettato.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeMaxSim;

impl ColbertScorer for NativeMaxSim {
    fn compute_maxsim(&self, query_tokens: &[&[f64]], doc_tokens: &[&[f64]]) -> f64 {
        maxsim(query_tokens, doc_tokens)
    }
}

#[inline]
fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[inline]
fn norm_l2(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maxsim_identico() {
        let q = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();

        let score = maxsim(&q_refs, &q_refs);
        assert!((score - 1.0).abs() < 1e-12);
    }

    #[test]
    fn maxsim_ortogonale() {
        let q = vec![vec![1.0, 0.0]];
        let d = vec![vec![0.0, 1.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        let score = maxsim(&q_refs, &d_refs);
        assert!(score.abs() < 1e-12);
    }

    #[test]
    fn maxsim_seleziona_massimo_per_token() {
        let q = vec![vec![1.0, 0.0]];
        // Il secondo token del doc è identico al token della query
        let d = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        let score = maxsim(&q_refs, &d_refs);
        assert!((score - 1.0).abs() < 1e-12);
    }

    #[test]
    fn input_invalido_ritorna_nan() {
        let q = vec![vec![1.0, 0.0]];
        let vuoto: Vec<&[f64]> = vec![];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();

        assert!(maxsim(&q_refs, &vuoto).is_nan());
        assert!(maxsim(&vuoto, &q_refs).is_nan());
    }

    #[test]
    fn dimensione_disallineata_ritorna_nan() {
        let q = vec![vec![1.0, 0.0]];
        let d = vec![vec![1.0, 0.0, 0.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        assert!(maxsim(&q_refs, &d_refs).is_nan());
    }

    // --- Test del trait ColbertScorer ---

    #[test]
    fn native_scorer_rispetta_il_contratto() {
        let scorer = NativeMaxSim;

        // Caso valido: token identici → 1.0
        let q = vec![vec![1.0, 0.0]];
        let d = vec![vec![1.0, 0.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();
        let score = scorer.compute_maxsim(&q_refs, &d_refs);
        assert!((score - 1.0).abs() < 1e-12);

        // Input vuoto → NaN (onestà geometrica)
        let vuoto: Vec<&[f64]> = vec![];
        assert!(scorer.compute_maxsim(&q_refs, &vuoto).is_nan());
        assert!(scorer.compute_maxsim(&vuoto, &d_refs).is_nan());

        // Dimensioni disallineate → NaN
        let d_dis = vec![vec![1.0, 0.0, 0.0]];
        let d_dis_refs: Vec<&[f64]> = d_dis.iter().map(|v| v.as_slice()).collect();
        assert!(scorer.compute_maxsim(&q_refs, &d_dis_refs).is_nan());
    }

    #[test]
    fn native_scorer_coerente_con_free_function() {
        let scorer = NativeMaxSim;
        let q = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let d = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        let via_trait = scorer.compute_maxsim(&q_refs, &d_refs);
        let via_free = maxsim(&q_refs, &d_refs);
        assert_eq!(via_trait, via_free);
    }
}