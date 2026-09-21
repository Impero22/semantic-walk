//! # parse — il ponte dai dati grezzi del server al contratto del cammino
//!
//! La porta che trasforma i dati del server CrispEmbed nelle strutture del
//! contratto di `semantic-walk`: `CrispTrajectory` (matrice densa per-token)
//! e `OrderedSparseSequence` (attivazioni sparse in ordine posizionale).
//!
//! ## Perché esiste questo modulo
//!
//! Il server CrispEmbed salva per ogni fatto il `walk` (quattro array
//! `ids/w/pos/st` allineati per indice, una voce per occorrenza) e la matrice
//! ColBERT per-token come multivector in Qdrant. Questo modulo trasforma quei
//! dati grezzi nel contratto del cammino:
//!
//! * `RawWalk` → `OrderedSparseSequence` (canale sparso ordinato), applicando
//!   il filtro d'igiene obbligatorio (`st == 0` e `id >= 4`).
//! * `RawColbertTrajectory` → `CrispTrajectory` (canale denso per-token).
//!
//! ## Il filtro d'igiene
//!
//! Il `walk` grezzo contiene rumore che va gittato via PRIMA di ogni calcolo:
//!
//! * i passi soppressi (`st == 1`) e il padding (`st == 2`) non contano;
//! * i token speciali (`id 0/2/3`) escono marcati `st == 0` con peso positivo,
//!   come se fossero parole vere — in ogni cammino osservato la posizione 0 ha
//!   `id 0` e peso ~0.18. Se ci si fida solo dello `st`, si contano i segni di
//!   punteggiatura tra le parole.
//!
//! La regola, dal documento del Coder (7/09): **tieni i passi con `st == 0` e
//! `id >= 4`**. Qualsiasi altro uso del `walk` conta rumore come segnale.

use crate::ingest::CrispTrajectory;
use crate::ordered_sparse::{OrderedSparseSequence, TokenId};

/// Il peso firmato di un passo del `walk`.
///
/// `w > 0` → il peso conta, il token contribuisce. `w ≤ 0` → non è rumore, è
/// un **verdetto**: il modello ha guardato il token e ha dichiarato che non
/// doveva contare (una soppressione attiva).
pub type WalkWeight = f32;

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

/// La risposta grezza del `walk` di un fatto.
///
/// Quattro array della stessa lunghezza `T`, **allineati per indice**: una
/// voce per occorrenza (token del testo), nell'ordine di apparizione, senza
/// dedup — la stessa parola contata due volte esce due volte con pesi diversi.
#[derive(Debug, Clone, PartialEq)]
pub struct RawWalk {
    /// L'identificativo della sequenza (es. id del fatto).
    pub sequence_id: String,
    /// Token id XLM-R (~250k vocabolario) toccato in quel passo.
    pub ids: Vec<TokenId>,
    /// Peso di quell'occorrenza, **firmato** (`w > 0` conta; `w ≤ 0` è un
    /// verdetto, non rumore).
    pub w: Vec<WalkWeight>,
    /// Posizione del token nell'input dell'encoder, **strettamente crescente**.
    pub pos: Vec<u32>,
    /// Stato del passo: `0` emesso · `1` soppresso · `2` padding.
    pub st: Vec<u8>,
}

/// Errore di parsing: un dato grezzo che non rispetta il contratto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// La matrice per-token ha vettori di dimensioni incoerenti, o la
    /// dimensione dichiarata non corrisponde.
    DimensioneIncoerente,
    /// Il numero di token non corrisponde al numero di righe della matrice.
    TokenMatriceDisallineati,
    /// Gli array del `walk` non hanno la stessa lunghezza, o un `pos` non è
    /// strettamente crescente.
    WalkDisallineato,
    /// Dopo il filtro d'igiene (`st == 0` e `id >= 4`) non resta nessun passo.
    ///
    /// Il contratto di `OrderedSparseSequence` rifiuta le sequenze vuote: un
    /// testo senza token significativi è un dato ambiguo, non una traiettoria.
    WalkVuotoDopoFiltro,
    /// Il numero di passi emessi dal `walk` non coincide con il numero di righe
    /// della matrice ColBERT.
    ///
    /// Entrambi rappresentano la stessa traiettoria (canale denso e canale
    /// sparso); se i due conteggi divergono il dato è corrotto.
    CoerenzaPosizionaleFallita,
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
            ParseError::WalkDisallineato => {
                write!(f, "array del walk non allineati o posizione non crescente")
            }
            ParseError::WalkVuotoDopoFiltro => {
                write!(f, "nessun passo significativo dopo il filtro d'igiene (st==0 e id>=4)")
            }
            ParseError::CoerenzaPosizionaleFallita => {
                write!(f, "passi del walk diversi dalle righe della matrice ColBERT")
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

/// Applica il filtro d'igiene a un `walk` grezzo, restituendo i token
/// significativi nell'ordine del cammino.
///
/// Tiene i passi con `st == 0` (emesso) e `id >= 4` (non speciale), come da
/// documento del Coder. I pesi sono i `w` originali, **firmati**: un `w ≤ 0`
/// è un verdetto di soppressione, non rumore da gittare.
fn walk_filtra_igiene(raw: &RawWalk) -> Vec<(TokenId, WalkWeight)> {
    raw.ids
        .iter()
        .zip(raw.w.iter())
        .zip(raw.st.iter())
        .filter_map(|((&id, &w), &st)| {
            // Filtro d'igiene: emesso (st==0) e non speciale (id>=4).
            if st == 0 && id >= 4 {
                Some((id, w))
            } else {
                None
            }
        })
        .collect()
}

/// Costruisce una `OrderedSparseSequence` dal `walk` grezzo di un fatto.
///
/// Applica il filtro d'igiene (`st == 0` e `id >= 4`), verifica l'allineamento
/// degli array e la stretta crescenza di `pos`, poi delega la costruzione a
/// `OrderedSparseSequence::from_frames`.
///
/// Il walk reale è già una sequenza posizionale: ogni passo emesso diventa una
/// posizione della sequenza ordinata. Il filtro d'igiene è la selezione dei
/// passi che contano davvero.
pub fn walk_to_sequence(raw: &RawWalk) -> Result<OrderedSparseSequence, ParseError> {
    // Allineamento: quattro array della stessa lunghezza.
    let t = raw.ids.len();
    if raw.w.len() != t || raw.pos.len() != t || raw.st.len() != t {
        return Err(ParseError::WalkDisallineato);
    }

    // Stretta crescenza di pos: l'ordine del cammino deve essere un ordine.
    for i in 1..t {
        if raw.pos[i] <= raw.pos[i - 1] {
            return Err(ParseError::WalkDisallineato);
        }
    }

    // Filtro d'igiene: solo i passi emessi e non speciali contano.
    let significativi = walk_filtra_igiene(raw);
    if significativi.is_empty() {
        return Err(ParseError::WalkVuotoDopoFiltro);
    }

    // Ogni passo emesso è una posizione della sequenza ordinata. Il walk reale
    // ha un solo token per posizione (una voce per occorrenza), quindi ogni
    // frame ha esattamente un elemento.
    let frames: Vec<Vec<(TokenId, WalkWeight)>> = significativi
        .into_iter()
        .map(|(id, w)| vec![(id, w)])
        .collect();

    OrderedSparseSequence::from_frames(&frames)
        .map_err(|_| ParseError::WalkVuotoDopoFiltro)
}

/// Verifica la coerenza posizionale tra il `walk` e la matrice ColBERT.
///
/// Il numero di passi emessi dal `walk` (dopo il filtro d'igiene) deve
/// coincidere con il numero di righe della matrice ColBERT: entrambi
/// rappresentano la stessa traiettoria, vista nel canale denso e in quello
/// sparso. Se i due conteggi divergono, il dato è corrotto.
pub fn verifica_coerenza_posizionale(
    raw_walk: &RawWalk,
    colbert_righe: usize,
) -> Result<(), ParseError> {
    let passi = walk_filtra_igiene(raw_walk).len();
    if passi != colbert_righe {
        return Err(ParseError::CoerenzaPosizionaleFallita);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un `walk` sintetico ben formato: `t` passi emessi, id crescenti da 4.
    fn raw_walk(t: usize) -> RawWalk {
        RawWalk {
            sequence_id: "fatto_test".into(),
            ids: (0..t).map(|i| (i + 4) as TokenId).collect(),
            w: (0..t).map(|i| 0.1 + i as f32).collect(),
            pos: (0..t).map(|i| i as u32).collect(),
            st: vec![0; t],
        }
    }

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
    fn walk_ben_formato_costruisce_sequenza() {
        // Quattro passi emessi, id da 4 in su: la sequenza ha 4 posizioni.
        let raw = raw_walk(4);
        let seq = walk_to_sequence(&raw).unwrap();
        assert_eq!(seq.num_positions(), 4);
        // L'ordine del cammino è preservato: ogni posizione ha il suo token.
        for i in 0..4 {
            assert_eq!(seq.tokens_at(i), &[(i + 4) as TokenId]);
            assert_eq!(seq.weights_at(i), &[0.1 + i as f32]);
        }
    }

    #[test]
    fn walk_filtro_igiene_scarta_soppressi_e_speciali() {
        // Passi: id 0 (speciale, st=0), id 5 (soppresso, st=1), id 6 (emesso).
        let raw = RawWalk {
            sequence_id: "fatto_test".into(),
            ids: vec![0, 5, 6],
            w: vec![0.18, 0.5, 0.7],
            pos: vec![0, 1, 2],
            st: vec![0, 1, 0],
        };
        let seq = walk_to_sequence(&raw).unwrap();
        // Solo il passo id 6 (emesso e non speciale) sopravvive.
        assert_eq!(seq.num_positions(), 1);
        assert_eq!(seq.tokens_at(0), &[6]);
        assert_eq!(seq.weights_at(0), &[0.7]);
    }

    #[test]
    fn walk_filtro_igiene_gitta_padding() {
        // Il padding (st==2) non conta, anche con id >= 4.
        let raw = RawWalk {
            sequence_id: "fatto_test".into(),
            ids: vec![4, 5, 6],
            w: vec![0.1, 0.2, 0.3],
            pos: vec![0, 1, 2],
            st: vec![0, 2, 0],
        };
        let seq = walk_to_sequence(&raw).unwrap();
        // Il passo con st==2 (padding) viene gittato; restano 4 e 6.
        assert_eq!(seq.num_positions(), 2);
        assert_eq!(seq.tokens_at(0), &[4]);
        assert_eq!(seq.tokens_at(1), &[6]);
    }

    #[test]
    fn walk_allineamento_violato_rifiutato() {
        // Array di lunghezze diverse: dato corrotto.
        let raw = RawWalk {
            sequence_id: "fatto_test".into(),
            ids: vec![4, 5, 6],
            w: vec![0.1, 0.2], // più corto di ids
            pos: vec![0, 1, 2],
            st: vec![0, 0, 0],
        };
        assert_eq!(
            walk_to_sequence(&raw),
            Err(ParseError::WalkDisallineato)
        );
    }

    #[test]
    fn walk_posizione_non_crescente_rifiutata() {
        // pos non strettamente crescente: l'ordine del cammino è violato.
        let raw = RawWalk {
            sequence_id: "fatto_test".into(),
            ids: vec![4, 5, 6],
            w: vec![0.1, 0.2, 0.3],
            pos: vec![0, 1, 1], // 1 ripetuta
            st: vec![0, 0, 0],
        };
        assert_eq!(
            walk_to_sequence(&raw),
            Err(ParseError::WalkDisallineato)
        );
    }

    #[test]
    fn walk_vuoto_dopo_filtro_rifiutato() {
        // Tutti i passi soppressi: nessun token significativo.
        let raw = RawWalk {
            sequence_id: "fatto_test".into(),
            ids: vec![4, 5, 6],
            w: vec![0.1, 0.2, 0.3],
            pos: vec![0, 1, 2],
            st: vec![1, 1, 1], // tutti soppressi
        };
        assert_eq!(
            walk_to_sequence(&raw),
            Err(ParseError::WalkVuotoDopoFiltro)
        );
    }

    #[test]
    fn walk_vuoto_per_speciali_rifiutato() {
        // Tutti token speciali (id < 4) anche se emessi: nessun token valido.
        let raw = RawWalk {
            sequence_id: "fatto_test".into(),
            ids: vec![0, 2, 3],
            w: vec![0.18, 0.1, 0.1],
            pos: vec![0, 1, 2],
            st: vec![0, 0, 0],
        };
        assert_eq!(
            walk_to_sequence(&raw),
            Err(ParseError::WalkVuotoDopoFiltro)
        );
    }

    #[test]
    fn coerenza_posizionale_ok() {
        // 3 passi emessi e 3 righe ColBERT: coerenti.
        let raw = raw_walk(3);
        assert!(verifica_coerenza_posizionale(&raw, 3).is_ok());
    }

    #[test]
    fn coerenza_posizionale_divergente_rifiutata() {
        // 3 passi emessi ma 4 righe ColBERT: il dato è corrotto.
        let raw = raw_walk(3);
        assert_eq!(
            verifica_coerenza_posizionale(&raw, 4),
            Err(ParseError::CoerenzaPosizionaleFallita)
        );
    }

    #[test]
    fn coerenza_posizionale_conta_solo_emessi() {
        // Il walk ha 4 passi ma 1 è soppresso: contano 3, come la matrice.
        let raw = RawWalk {
            sequence_id: "fatto_test".into(),
            ids: vec![4, 5, 6, 7],
            w: vec![0.1, 0.2, 0.3, 0.4],
            pos: vec![0, 1, 2, 3],
            st: vec![0, 0, 0, 1], // l'ultimo è soppresso
        };
        assert!(verifica_coerenza_posizionale(&raw, 3).is_ok());
        // Con 4 righe invece fallisce: i soppressi non contano come passi.
        assert_eq!(
            verifica_coerenza_posizionale(&raw, 4),
            Err(ParseError::CoerenzaPosizionaleFallita)
        );
    }
}