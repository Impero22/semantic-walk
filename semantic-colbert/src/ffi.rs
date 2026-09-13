//! # Strato FFI verso CrispEmbed (motore C++)
//!
//! Questo modulo definisce il **contratto C-ABI** tra l'ecosistema Rust e il
//! motore C++ di CrispEmbed, e il wrapper Rust `CrispEmbedScorer` che
//! implementa `ColbertScorer` delegando al motore.
//!
//! ## Contratto C-ABI
//!
//! Il motore C++ espone una funzione con firma:
//!
//! ```c
//! // Calcola la MaxSim tra le matrici di token di query e documento.
//! // I token sono buffer contigui `f32` di lunghezza `dim` ciascuno.
//! // Restituisce il punteggio in [-1.0, 1.0], o NaN per input invalidi.
//! float crispembed_maxsim(
//!     const float* query_tokens, size_t query_n,      // query: query_n × dim
//!     const float* doc_tokens,   size_t doc_n,        // doc:    doc_n   × dim
//!     size_t dim
//! );
//! ```
//!
//! ## Allocazione contigua
//!
//! L'input Rust è `&[&[f64]]` — un vettore di puntatori a vettori. La FFI
//! richiede buffer contigui `f32`. Il wrapper incapsula la conversione:
//! appiattisce le righe in un unico buffer contiguo e lo passa al motore.
//! Questa conversione è l'unico punto di allocazione del percorso FFI.
//!
//! ## Onestà geometrica
//!
//! Il contratto è identico al provider nativo: input invalidi → `f64::NAN`.
//! Se la libreria C++ non è caricata (funzione assente), il wrapper si
//! ritira restituendo `f64::NAN` — meglio un punteggio ignoto che un
//! punteggio falso.

use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::ColbertScorer;

/// Nome simbolico della funzione C-ABI esposta dal motore C++ CrispEmbed.
///
/// Verificato dal vivo su .18 (13/09/26): il simbolo reale è
/// `crispembed_colbert_score`, NON `crispembed_maxsim` (che non esiste).
pub const CRISPEMBED_COLBERT_SCORE_SYMBOL: &str = "crispembed_colbert_score";

/// Tipo della funzione C-ABI.
///
/// * `query_vecs` — buffer contiguo `f32` dei token della query (`n_query × dim`).
/// * `n_query` — numero di token della query.
/// * `doc_vecs` — buffer contiguo `f32` dei token del documento (`n_doc × dim`).
/// * `n_doc` — numero di token del documento.
/// * `dim` — dimensione di ogni vettore di token.
///
/// Nota ABI: il motore C++ usa `int` (32-bit) per le dimensioni, non
/// `usize`/`size_t` (64-bit). La firma rispecchia l'header `crispembed.h`.
///
/// Restituisce il punteggio MaxSim (late interaction), più alto = più
/// rilevante. Q e D devono essere embedding per-token L2-normalizzati.
pub type CrispEmbedColbertScoreFn = unsafe extern "C" fn(
    query_vecs: *const f32,
    n_query: i32,
    doc_vecs: *const f32,
    n_doc: i32,
    dim: i32,
) -> f32;

/// Provider FFI verso il motore C++ CrispEmbed.
///
/// Incapsula il puntatore alla funzione C-ABI (caricata dinamicamente) e la
/// conversione dei token Rust in buffer contigui. Se la funzione non è
/// disponibile, `compute_maxsim` restituisce `f64::NAN` (ritiro geometrico).
#[derive(Debug)]
pub struct CrispEmbedScorer {
    /// Puntatore alla funzione C-ABI caricata (null se non disponibile).
    func: AtomicPtr<c_void>,
}

impl CrispEmbedScorer {
    /// Crea un nuovo scorer FFI dal puntatore alla funzione C-ABI.
    ///
    /// Se `func` è null (libreria non caricata), lo scorer si comporta come
    /// un ritiro: ogni chiamata restituisce `f64::NAN`.
    pub fn new(func: *const c_void) -> Self {
        Self {
            func: AtomicPtr::new(func as *mut c_void),
        }
    }

    /// Crea uno scorer "vuoto" (funzione non caricata) — utile per i test
    /// del contratto e per il fallback permissivo.
    pub fn unloaded() -> Self {
        Self::new(std::ptr::null())
    }

    /// Restituisce il puntatore alla funzione C-ABI, o `None` se non caricata.
    fn raw_func(&self) -> Option<CrispEmbedColbertScoreFn> {
        let ptr = self.func.load(Ordering::Relaxed);
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { std::mem::transmute(ptr) })
        }
    }
}

impl Default for CrispEmbedScorer {
    fn default() -> Self {
        Self::unloaded()
    }
}

impl ColbertScorer for CrispEmbedScorer {
    fn compute_maxsim(&self, query_tokens: &[&[f64]], doc_tokens: &[&[f64]]) -> f64 {
        // Ritiro geometrico: se la libreria non è caricata, meglio un
        // punteggio ignoto che un punteggio falso.
        let func = match self.raw_func() {
            Some(f) => f,
            None => return f64::NAN,
        };

        // Validazione input (onestà geometrica, identica al provider nativo).
        if query_tokens.is_empty() || doc_tokens.is_empty() {
            return f64::NAN;
        }
        let dim = query_tokens[0].len();
        if dim == 0 {
            return f64::NAN;
        }
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

        // Conversione in buffer contigui f32 (unico punto di allocazione FFI).
        // La conversione da f64 a f32 è una perdita di precisione accettata
        // dal contratto C-ABI: il motore C++ lavora in f32.
        let query_flat: Vec<f32> = query_tokens
            .iter()
            .flat_map(|row| row.iter().map(|&x| x as f32))
            .collect();
        let doc_flat: Vec<f32> = doc_tokens
            .iter()
            .flat_map(|row| row.iter().map(|&x| x as f32))
            .collect();

        // Chiamata al motore C++. Le dimensioni sono `int` (32-bit) per
        // contratto ABI; i valori realistici (token per query/doc) stanno
        // ampiamente dentro i32, quindi la conversione è sicura.
        let score = unsafe {
            func(
                query_flat.as_ptr(),
                query_tokens.len() as i32,
                doc_flat.as_ptr(),
                doc_tokens.len() as i32,
                dim as i32,
            )
        };

        // Il motore restituisce f32; convertiamo in f64 mantenendo il
        // contratto di ritiro: NaN in, NaN out.
        score as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColbertScorer;

    /// Implementazione di test della C-ABI: replica la MaxSim in f32,
    /// esattamente come farebbe il motore C++ reale. Firma allineata
    /// all'header `crispembed.h` (dimensioni `int`, non `usize`).
    unsafe extern "C" fn mock_crispembed_colbert_score(
        query_tokens: *const f32,
        query_n: i32,
        doc_tokens: *const f32,
        doc_n: i32,
        dim: i32,
    ) -> f32 {
        if query_tokens.is_null() || doc_tokens.is_null() || query_n <= 0 || doc_n <= 0 || dim <= 0
        {
            return f32::NAN;
        }

        let (query_n, doc_n, dim) = (query_n as usize, doc_n as usize, dim as usize);
        let q_slice = std::slice::from_raw_parts(query_tokens, query_n * dim);
        let d_slice = std::slice::from_raw_parts(doc_tokens, doc_n * dim);

        let mut somma_massimi = 0.0f32;
        for i in 0..query_n {
            let q_row = &q_slice[i * dim..(i + 1) * dim];
            let norm_q = q_row.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm_q == 0.0 || norm_q.is_nan() {
                return f32::NAN;
            }

            let mut max_sim = f32::NEG_INFINITY;
            for j in 0..doc_n {
                let d_row = &d_slice[j * dim..(j + 1) * dim];
                let norm_d = d_row.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_d == 0.0 || norm_d.is_nan() {
                    return f32::NAN;
                }

                let dot: f32 = q_row.iter().zip(d_row.iter()).map(|(a, b)| a * b).sum();
                let cos = (dot / (norm_q * norm_d)).clamp(-1.0, 1.0);
                if cos > max_sim {
                    max_sim = cos;
                }
            }
            if max_sim.is_nan() || max_sim.is_infinite() {
                return f32::NAN;
            }
            somma_massimi += max_sim;
        }

        somma_massimi / (query_n as f32)
    }

    fn scorer_caricato() -> CrispEmbedScorer {
        CrispEmbedScorer::new(mock_crispembed_colbert_score as *const c_void)
    }

    #[test]
    fn ffi_scorer_token_identici() {
        let scorer = scorer_caricato();
        let q = vec![vec![1.0, 0.0]];
        let d = vec![vec![1.0, 0.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        let score = scorer.compute_maxsim(&q_refs, &d_refs);
        assert!((score - 1.0).abs() < 1e-5, "score = {score}");
    }

    #[test]
    fn ffi_scorer_seleziona_massimo_per_token() {
        let scorer = scorer_caricato();
        let q = vec![vec![1.0, 0.0]];
        let d = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        let score = scorer.compute_maxsim(&q_refs, &d_refs);
        assert!((score - 1.0).abs() < 1e-5, "score = {score}");
    }

    #[test]
    fn ffi_scorer_input_invalido_ritorna_nan() {
        let scorer = scorer_caricato();
        let q = vec![vec![1.0, 0.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let vuoto: Vec<&[f64]> = vec![];

        assert!(scorer.compute_maxsim(&q_refs, &vuoto).is_nan());
        assert!(scorer.compute_maxsim(&vuoto, &q_refs).is_nan());

        let d_dis = vec![vec![1.0, 0.0, 0.0]];
        let d_dis_refs: Vec<&[f64]> = d_dis.iter().map(|v| v.as_slice()).collect();
        assert!(scorer.compute_maxsim(&q_refs, &d_dis_refs).is_nan());
    }

    #[test]
    fn ffi_scorer_non_caricato_si_ritira() {
        let scorer = CrispEmbedScorer::unloaded();
        let q = vec![vec![1.0, 0.0]];
        let d = vec![vec![1.0, 0.0]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        // Libreria non caricata → ritiro geometrico (NaN), mai punteggio falso.
        assert!(scorer.compute_maxsim(&q_refs, &d_refs).is_nan());
    }

    #[test]
    fn ffi_scorer_coerente_con_nativo() {
        // Il mock replica la MaxSim in f32: su token non degeneri deve
        // coincidere (entro la precisione f32) con il provider nativo f64.
        let ffi = scorer_caricato();
        let native = crate::NativeMaxSim;

        let q = vec![vec![0.8, 0.6], vec![0.2, 0.9]];
        let d = vec![vec![0.7, 0.7], vec![0.1, 0.95]];
        let q_refs: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_refs: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        let s_ffi = ffi.compute_maxsim(&q_refs, &d_refs);
        let s_native = native.compute_maxsim(&q_refs, &d_refs);
        assert!(
            (s_ffi - s_native).abs() < 1e-4,
            "ffi={s_ffi} native={s_native}"
        );
    }
}