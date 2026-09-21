//! # parse — il ponte dai dati grezzi del server al contratto del cammino
//!
//! La porta che trasforma le risposte del server CrispEmbed (quando gli
//! endpoint per-token saranno esposti) nelle strutture del contratto di
//! `semantic-walk`: `CrispTrajectory` (matrice densa per-token) e
//! `OrderedSparseSequence` (attivazioni sparse in ordine posizionale).
//!
//! ## Perché esiste questo modulo
//!
//! Il server CrispEmbed oggi espone `/api/embeddings` (dense di frase,
//! mean-pooling) e `/sparse` (mappa non ordinata token→peso). Nessuno dei
//! due basta a costruire una traiettoria per-token. Gli endpoint che servono
//! sono due:
//!
//! * la **matrice ColBERT per-token** (T×1024, una riga per token in ordine),
//! * la **head ordered-sparse con campo `walk`** (sequenza ordinata di token
//!   con i loro pesi, posizione per posizione).
//!
//! Quando Federico li esporrà, questo modulo li ingoia. Nel frattempo il
//! parsing è definito e testato su dati sintetici realistici, così l'aggancio
//! al server vero sarà immediato: basta sostituire la sorgente dati.
//!
//! ## Agnostico al formato JSON esatto
//!
//! Le struct di input qui sotto rappresentano il **contenuto semantico** di
//! ciò che il server restituirà, non il suo JSON esatto. Se Federico sceglie
//! un formato diverso (nomi di campo, nesting), si adatta solo il parsing
//! JSON — la trasformazione in `CrispTrajectory`/`OrderedSparseSequence`
//! resta identica. È il livello che non va riscritto quando il contratto
//! HTTP si precisa.

use crate::ingest::CrispTrajectory;
use crate::ordered_sparse::{OrderedSparseSequence, TokenId};

/// La risposta grezza dell'endpoint matrice ColBERT per-token.
///
/// Un testo, la sua sequenza di token nell'ordine esatto, e per ogni token
/// il vettore D-dimensionale.
#[derive(Debug, Clone, PartialEq)]
pub struct RawColbertTrajectory {
    /// L'identificativo della sequenza (es. id del fatto).
    pub sequence_id: String,
    /// I token nell'ordine esatto in cui compaiono nel testo.
    pub tokens: Vec<String>,
    /// La matrice per-token: `embeddings[i]` è il vettore del token `i`.
    pub embeddings: Vec<Vec<f64>>,
}

/// La risposta grezza dell'endpoint ordered-sparse con campo `walk`.
///
/// Per ogni posizione, la lista delle attivazioni sparse `(token_id, peso)`
/// dei token attivi in quella posizione.
#[derive(Debug, Clone, PartialEq)]
pub struct RawSparseWalk {
    /// L'identificativo della sequenza.
    pub sequence_id: String,
    /// Il walk: per ogni posizione, le coppie (token_id, peso) attive.
    ///
    /// `frames[i]` è l'insieme delle attivazioni sparse alla posizione `i`.
    pub frames: Vec<Vec<(TokenId, f32)>>,
}

/// Errore di parsing: un dato grezzo che non rispetta il contratto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// La matrice per-token ha vettori di dimensioni incoerenti, o la
    /// dimensione dichiarata non corrisponde.
    DimensioneIncoerente,
    /// Il numero di token non corrisponde al numero di righe della matrice.
    TokenMatriceDisallineati,
    /// Il walk ordered-sparse contiene una posizione vuota.
    ///
    /// Il contratto di `OrderedSparseSequence::from_frames` rifiuta le
    /// posizioni vuote (un token senza attivazioni sparse è ambiguo). Questo
    /// è un punto di contratto aperto con Camillo: nel mondo reale un token
    /// può non avere attivazioni sparse, e la gestione va decisa insieme.
    PosizioneVuota,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::DimensioneIncoerente => {
                write!(f, "dimensione dei vettori incoerente nella matrice per-token")
            }
            ParseError::TokenMatriceDisallineati => {
                write!(f, "numero di token diverso dal numero di righe della matrice")
            }
            ParseError::PosizioneVuota => {
                write!(f, "posizione vuota nel walk ordered-sparse")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Costruisce una `CrispTrajectory` dalla risposta grezza della matrice
/// ColBERT per-token.
///
/// Verifica la coerenza: ogni riga deve avere la stessa dimensione, e il
/// numero di token deve combaciare col numero di righe. La dimensione è
/// dedotta dalla prima riga (o è zero per una traiettoria vuota).
pub fn colbert_to_trajectory(raw: &RawColbertTrajectory) -> Result<CrispTrajectory, ParseError> {
    // Coerenza del numero di token con il numero di righe.
    if raw.tokens.len() != raw.embeddings.len() {
        return Err(ParseError::TokenMatriceDisallineati);
    }

    // Coerenza dimensionale: tutte le righe uguali.
    let dim = match raw.embeddings.first() {
        Some(first) => first.len(),
        None => 0,
    };
    for emb in &raw.embeddings {
        if emb.len() != dim {
            return Err(ParseError::DimensioneIncoerente);
        }
    }

    // `CrispTrajectory::new` verifica ulteriormente (dimensione > 0).
    CrispTrajectory::new(raw.sequence_id.clone(), dim, raw.embeddings.clone())
        .map_err(|_| ParseError::DimensioneIncoerente)
}

/// Costruisce una `OrderedSparseSequence` dalla risposta grezza del walk
/// ordered-sparse.
///
/// Delega il calcolo della firma globale e la verifica di coerenza a
/// `OrderedSparseSequence::from_frames`. La posizione vuota è rifiutata
/// (contratto del modulo); il punto è aperto con Camillo.
pub fn sparse_walk_to_sequence(raw: &RawSparseWalk) -> Result<OrderedSparseSequence, ParseError> {
    OrderedSparseSequence::from_frames(&raw.frames)
        .map_err(|err| match err {
            "Una posizione della sequenza non può essere vuota" => ParseError::PosizioneVuota,
            _ => ParseError::PosizioneVuota,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Una matrice per-token sintetica coerente.
    fn raw_colbert(n_tokens: usize, dim: usize) -> RawColbertTrajectory {
        RawColbertTrajectory {
            sequence_id: "fatto_test".into(),
            tokens: (0..n_tokens).map(|i| format!("tok_{i}")).collect(),
            embeddings: (0..n_tokens)
                .map(|i| {
                    let mut v = vec![0.0; dim];
                    v[i % dim] = 1.0;
                    v
                })
                .collect(),
        }
    }

    #[test]
    fn colbert_ben_formato_costruisce_traiettoria() {
        let raw = raw_colbert(4, 1024);
        let traj = colbert_to_trajectory(&raw).unwrap();
        assert_eq!(traj.sequence_id, "fatto_test");
        assert_eq!(traj.dimension, 1024);
        assert_eq!(traj.len(), 4);
        // L'ordine è preservato: la traiettoria è una sequenza, non una borsa.
        assert_eq!(traj.embeddings[0], raw.embeddings[0]);
        assert_eq!(traj.embeddings[3], raw.embeddings[3]);
    }

    #[test]
    fn colbert_token_matrice_disallineati_rifiutato() {
        // 4 token ma 3 righe: dato corrotto.
        let mut raw = raw_colbert(4, 1024);
        raw.embeddings.pop();
        assert_eq!(
            colbert_to_trajectory(&raw),
            Err(ParseError::TokenMatriceDisallineati)
        );
    }

    #[test]
    fn colbert_dimensione_incoerente_rifiutata() {
        // Una riga con dimensione diversa: matrice corrotta.
        let mut raw = raw_colbert(3, 1024);
        raw.embeddings[1] = vec![0.0; 512];
        assert_eq!(
            colbert_to_trajectory(&raw),
            Err(ParseError::DimensioneIncoerente)
        );
    }

    #[test]
    fn colbert_traiettoria_vuota_rifiutata() {
        // Zero token e zero righe: `CrispTrajectory::new` rifiuta dim 0.
        let raw = raw_colbert(0, 0);
        assert_eq!(
            colbert_to_trajectory(&raw),
            Err(ParseError::DimensioneIncoerente)
        );
    }

    #[test]
    fn sparse_walk_ben_formato_costruisce_sequenza() {
        // Due posizioni, ognuna con le sue attivazioni sparse.
        let raw = RawSparseWalk {
            sequence_id: "walk_test".into(),
            frames: vec![
                vec![(10, 0.5), (20, 0.3)],
                vec![(30, 0.9)],
            ],
        };
        let seq = sparse_walk_to_sequence(&raw).unwrap();
        assert_eq!(seq.num_positions(), 2);
        assert_eq!(seq.tokens_at(0), &[10, 20]);
        assert_eq!(seq.weights_at(0), &[0.5, 0.3]);
        assert_eq!(seq.tokens_at(1), &[30]);
        assert_eq!(seq.weights_at(1), &[0.9]);
    }

    #[test]
    fn sparse_walk_posizione_vuota_rifiutata() {
        // La seconda posizione è vuota: contratto del modulo la rifiuta.
        let raw = RawSparseWalk {
            sequence_id: "walk_test".into(),
            frames: vec![vec![(10, 0.5)], vec![]],
        };
        assert_eq!(
            sparse_walk_to_sequence(&raw),
            Err(ParseError::PosizioneVuota)
        );
    }

    #[test]
    fn sparse_walk_peso_non_finito_rifiutato() {
        // Peso NaN: dato corrotto, rifiutato da from_frames.
        let raw = RawSparseWalk {
            sequence_id: "walk_test".into(),
            frames: vec![vec![(10, f32::NAN)]],
        };
        assert!(sparse_walk_to_sequence(&raw).is_err());
    }

    #[test]
    fn sparse_walk_vuoto_rifiutato() {
        // Nessuna posizione: sequenza vuota, rifiutata.
        let raw = RawSparseWalk {
            sequence_id: "walk_test".into(),
            frames: vec![],
        };
        assert!(sparse_walk_to_sequence(&raw).is_err());
    }
}