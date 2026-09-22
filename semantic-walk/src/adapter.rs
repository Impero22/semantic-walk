//! # adapter — l'adattatore JSON del server vetta-semantic
//!
//! Il ponte che chiude l'anello: trasforma la **risposta JSON grezza** degli
//! endpoint del server (`POST /colbert/encode?tokens=1` e
//! `POST /ordered-sparse?format=frames`) nelle strutture del contratto di
//! `semantic-walk` — `RawColbertTrajectory` e `RawWalk` — che `parse.rs` poi
//! converte in `CrispTrajectory` e `OrderedSparseSequence`.
//!
//! ## Perché esiste questo modulo
//!
//! `semantic-walk` è un crate **puro**: non fa richieste HTTP, non conosce
//! serde, non sa nulla della rete. I tipi `RawColbertTrajectory` e `RawWalk`
//! sono intermedî: chi li costruisce dalla risposta JSON del server non
//! esisteva ancora. Questo modulo è quell'anello mancante.
//!
//! L'adattatore è **dipendente dal formato**: conosce i nomi esatti dei campi
//! esposti dal server (`results`, `multivector`, `n_tokens`, `dim`, `tokens`,
//! `ids`, `weights`, `positions`, `status`, `n`) e li mappa nelle strutture
//! del contratto. Il contratto semantico resta in `parse.rs`; qui c'è solo la
//! traslazione di forma, senza logica di cammino.
//!
//! ## La forma flat
//!
//! Il server espone l'ordered-sparse in due forme: `flat`
//! (`{n, ids, weights, positions, status}` — quattro array paralleli, un
//! token per posizione) e `frames` (regruppata per posizione). Il nostro
//! `walk_to_sequence` assume un solo token per posizione
//! (`vec![(id, w)]` per frame), quindi la **flat** è la forma naturale.
//! L'adattatore consumerà la flat.

use crate::parse::{RawColbertTrajectory, RawWalk, WalkWeight};
use crate::ordered_sparse::TokenId;
use serde::Deserialize;

/// La risposta completa del server: un involucro con un array di risultati.
///
/// Sia `/colbert/encode` che `/ordered-sparse` restituiscono un oggetto con
/// una chiave `results` contenente un array (uno per ogni testo inviato).
#[derive(Debug, Clone, Deserialize)]
pub struct ServerResponse<T> {
    pub results: Vec<T>,
}

/// Un singolo risultato dell'endpoint matrice ColBERT per-token.
///
/// Mappa la risposta di `POST /colbert/encode?tokens=1`:
///
/// ```jsonc
/// { "multivector": [[0.056, ...], ...], "n_tokens": 5,
///   "dim": 1024, "tokens": ["<s>", "▁il", "▁gat", "to", "</s>"] }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ColbertResultJson {
    /// La matrice per-token: `multivector[i]` è il vettore del token `i`.
    #[serde(rename = "multivector")]
    pub embeddings: Vec<Vec<f64>>,
    /// Il numero di token (deve coincidere con le righe della matrice).
    #[serde(rename = "n_tokens")]
    pub n_tokens: usize,
    /// La dimensione comune dei vettori (per vetta-semantic: 1024).
    #[serde(rename = "dim")]
    pub dimension: usize,
    /// I token nell'ordine esatto in cui compaiono nel testo.
    ///
    /// **Opzionale**: il server reale non lo restituisce (solo `multivector`,
    /// `n_tokens`, `dim`). Quando assente l'adattatore genera i placeholder
    /// da `n_tokens`. La verità di conteggio resta `n_tokens`.
    #[serde(default)]
    pub tokens: Option<Vec<String>>,
}

/// Un singolo risultato dell'endpoint ordered-sparse, forma flat.
///
/// Mappa la risposta di `POST /ordered-sparse?format=frames` (forma flat):
///
/// ```jsonc
/// { "n": 5,
///   "ids":      [0, 211, 27294, 45, 2],
///   "weights":  [0.196189, 0.197522, 0.277092, -0.013, 0.201],
///   "positions":[0, 1, 2, 3, 4],
///   "status":   [0, 0, 0, 1, 0] }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct WalkResultJson {
    /// Il numero di passi (lunghezza degli array paralleli).
    #[serde(rename = "n")]
    pub count: usize,
    /// Token id XLM-R toccato in quel passo.
    pub ids: Vec<TokenId>,
    /// Peso firmato di quell'occorrenza (`w ≤ 0` è un verdetto).
    #[serde(rename = "weights")]
    pub w: Vec<WalkWeight>,
    /// Posizione del token, strettamente crescente.
    #[serde(rename = "positions")]
    pub pos: Vec<u32>,
    /// Stato del passo: `0` emesso · `1` soppresso · `2` padding.
    #[serde(rename = "status")]
    pub st: Vec<u8>,
}

/// L'errore dell'adattatore: un dato che non rispetta il formato del server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// La risposta non contiene alcun risultato (`results` vuoto).
    RisultatoAssente,
    /// Il numero di passi dichiarato (`n`) non coincide con la lunghezza
    /// effettiva degli array paralleli del walk.
    ConteggioIncoerente,
    /// Il numero di token dichiarato (`n_tokens`) non coincide con la
    /// lunghezza effettiva di `tokens` o `multivector`.
    ConteggioTokenIncoerente,
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::RisultatoAssente => {
                write!(f, "la risposta del server non contiene risultati")
            }
            AdapterError::ConteggioIncoerente => {
                write!(f, "il campo n non coincide con la lunghezza degli array del walk")
            }
            AdapterError::ConteggioTokenIncoerente => {
                write!(f, "il campo n_tokens non coincide con la lunghezza di tokens/multivector")
            }
        }
    }
}

impl std::error::Error for AdapterError {}

/// Costruisce una `RawColbertTrajectory` dalla risposta JSON del server.
///
/// Estrae il primo risultato dell'array `results` e verifica che il conteggio
/// dei token dichiarato (`n_tokens`) coincida con la lunghezza effettiva di
/// `tokens` e `multivector`. La coerenza dimensionale e la biiezione col walk
/// sono demandate a `parse.rs` (`colbert_to_trajectory` e
/// `verifica_coerenza_posizionale`).
pub fn colbert_json_to_raw(
    response: &ServerResponse<ColbertResultJson>,
    sequence_id: &str,
) -> Result<RawColbertTrajectory, AdapterError> {
    let result = response
        .results
        .first()
        .ok_or(AdapterError::RisultatoAssente)?;

    // Il server reale non restituisce `tokens`: lo deriviamo da `n_tokens`.
    // Se presente, verifichiamo che coincida con il conteggio dichiarato.
    let tokens = match &result.tokens {
        Some(toks) if toks.len() != result.n_tokens => {
            return Err(AdapterError::ConteggioTokenIncoerente);
        }
        Some(toks) => toks.clone(),
        None => vec![String::new(); result.n_tokens],
    };

    if result.embeddings.len() != result.n_tokens {
        return Err(AdapterError::ConteggioTokenIncoerente);
    }

    Ok(RawColbertTrajectory {
        sequence_id: sequence_id.to_string(),
        tokens,
        embeddings: result.embeddings.clone(),
    })
}

/// Costruisce una `RawWalk` dalla risposta JSON flat del server.
///
/// Estrae il primo risultato dell'array `results` e verifica che il conteggio
/// dichiarato (`n`) coincida con la lunghezza effettiva dei quattro array
/// paralleli. L'allineamento e la stretta crescenza di `pos` sono demandati a
/// `parse.rs` (`walk_to_sequence`).
pub fn walk_json_to_raw(
    response: &ServerResponse<WalkResultJson>,
    sequence_id: &str,
) -> Result<RawWalk, AdapterError> {
    let result = response
        .results
        .first()
        .ok_or(AdapterError::RisultatoAssente)?;

    if result.ids.len() != result.count
        || result.w.len() != result.count
        || result.pos.len() != result.count
        || result.st.len() != result.count
    {
        return Err(AdapterError::ConteggioIncoerente);
    }

    Ok(RawWalk {
        sequence_id: sequence_id.to_string(),
        ids: result.ids.clone(),
        w: result.w.clone(),
        pos: result.pos.clone(),
        st: result.st.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un risultato ColBERT JSON ben formato.
    fn colbert_json(n_tokens: usize, dim: usize) -> ColbertResultJson {
        ColbertResultJson {
            embeddings: (0..n_tokens)
                .map(|i| {
                    let mut v = vec![0.0; dim];
                    v[i % dim] = 1.0;
                    v
                })
                .collect(),
            n_tokens,
            dimension: dim,
            tokens: Some((0..n_tokens).map(|i| format!("tok_{i}")).collect()),
        }
    }

    /// Un risultato walk JSON flat ben formato.
    fn walk_json(t: usize) -> WalkResultJson {
        WalkResultJson {
            count: t,
            ids: (0..t).map(|i| (i + 4) as TokenId).collect(),
            w: (0..t).map(|i| 0.1 + i as f32).collect(),
            pos: (0..t).map(|i| i as u32).collect(),
            st: vec![0; t],
        }
    }

    #[test]
    fn colbert_json_ben_formato_costruisce_raw() {
        let resp = ServerResponse {
            results: vec![colbert_json(4, 1024)],
        };
        let raw = colbert_json_to_raw(&resp, "fatto_test").unwrap();
        assert_eq!(raw.sequence_id, "fatto_test");
        assert_eq!(raw.tokens.len(), 4);
        assert_eq!(raw.embeddings.len(), 4);
        // La matrice è preservata nell'ordine.
        assert_eq!(raw.embeddings[3], resp.results[0].embeddings[3]);
    }

    #[test]
    fn colbert_json_risultato_assente_rifiutato() {
        let resp: ServerResponse<ColbertResultJson> = ServerResponse { results: vec![] };
        assert_eq!(
            colbert_json_to_raw(&resp, "fatto_test"),
            Err(AdapterError::RisultatoAssente)
        );
    }

    #[test]
    fn colbert_json_conteggio_token_incoerente_rifiutato() {
        // n_tokens dichiara 4 ma tokens ne ha 3 (quando presenti).
        let mut result = colbert_json(4, 1024);
        result.tokens = Some(result.tokens.unwrap().into_iter().take(3).collect());
        let resp = ServerResponse { results: vec![result] };
        assert_eq!(
            colbert_json_to_raw(&resp, "fatto_test"),
            Err(AdapterError::ConteggioTokenIncoerente)
        );
    }

    #[test]
    fn colbert_json_conteggio_multivector_incoerente_rifiutato() {
        // n_tokens dichiara 4 ma embeddings ne ha 3 (il caso che il server
        // reale può produrre se n_tokens mente).
        let mut result = colbert_json(4, 1024);
        result.embeddings.pop();
        result.tokens = None;
        let resp = ServerResponse { results: vec![result] };
        assert_eq!(
            colbert_json_to_raw(&resp, "fatto_test"),
            Err(AdapterError::ConteggioTokenIncoerente)
        );
    }

    #[test]
    fn colbert_json_senza_tokens_deriva_placeholder() {
        // Il server reale non restituisce `tokens`: solo multivector,
        // n_tokens, dim. L'adattatore deve gestirlo senza errore.
        let result = ColbertResultJson {
            embeddings: vec![vec![0.1; 1024]; 4],
            n_tokens: 4,
            dimension: 1024,
            tokens: None,
        };
        let resp = ServerResponse { results: vec![result] };
        let raw = colbert_json_to_raw(&resp, "fatto_test").unwrap();
        assert_eq!(raw.tokens.len(), 4);
        assert_eq!(raw.embeddings.len(), 4);
        // I placeholder sono stringhe vuote, allineati a n_tokens.
        assert!(raw.tokens.iter().all(|t| t.is_empty()));
    }

    #[test]
    fn walk_json_ben_formato_costruisce_raw() {
        let resp = ServerResponse {
            results: vec![walk_json(4)],
        };
        let raw = walk_json_to_raw(&resp, "fatto_test").unwrap();
        assert_eq!(raw.sequence_id, "fatto_test");
        assert_eq!(raw.ids.len(), 4);
        assert_eq!(raw.w.len(), 4);
        assert_eq!(raw.pos.len(), 4);
        assert_eq!(raw.st.len(), 4);
        // Il mapping dei nomi: weights→w, positions→pos, status→st.
        assert_eq!(raw.pos[3], 3);
        assert_eq!(raw.st[2], 0);
    }

    #[test]
    fn walk_json_risultato_assente_rifiutato() {
        let resp: ServerResponse<WalkResultJson> = ServerResponse { results: vec![] };
        assert_eq!(
            walk_json_to_raw(&resp, "fatto_test"),
            Err(AdapterError::RisultatoAssente)
        );
    }

    #[test]
    fn walk_json_conteggio_incoerente_rifiutato() {
        // n dichiara 4 ma ids ne ha 3.
        let mut result = walk_json(4);
        result.ids.pop();
        let resp = ServerResponse { results: vec![result] };
        assert_eq!(
            walk_json_to_raw(&resp, "fatto_test"),
            Err(AdapterError::ConteggioIncoerente)
        );
    }

    #[test]
    fn adapter_poi_parse_anello_completo() {
        // L'adattatore produce RawWalk/RawColbertTrajectory che parse.rs
        // converte nel contratto del cammino. L'anello è chiuso.
        let colbert_resp = ServerResponse {
            results: vec![colbert_json(3, 1024)],
        };
        let walk_resp = ServerResponse {
            results: vec![walk_json(3)],
        };

        let raw_colbert = colbert_json_to_raw(&colbert_resp, "fatto_test").unwrap();
        let raw_walk = walk_json_to_raw(&walk_resp, "fatto_test").unwrap();

        // Coerenza posizionale: 3 passi emessi e 3 righe ColBERT.
        crate::parse::verifica_coerenza_posizionale(&raw_walk, raw_colbert.embeddings.len())
            .unwrap();

        // Conversione nel contratto del cammino.
        let traj = crate::parse::colbert_to_trajectory(&raw_colbert).unwrap();
        let seq = crate::parse::walk_to_sequence(&raw_walk).unwrap();
        assert_eq!(traj.len(), 3);
        assert_eq!(seq.num_positions(), 3);
    }
}