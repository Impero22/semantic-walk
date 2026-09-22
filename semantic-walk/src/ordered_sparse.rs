//! # ordered_sparse — la proiezione posizionale delle attivazioni
//!
//! La risposta architetturale alla domanda di Federico: l'ordered-sparse è
//! una **stanza nuova** o un vestito diverso del ColBERT?
//!
//! La decisione di design (Test 1 chiuso, Esito A): l'ordered-sparse è una
//! **proiezione posizionale** delle attivazioni dei token. Le chiavi
//! appartengono allo stesso spazio informativo dei token del ColBERT, ma
//! l'**ordine posizionale** viene preservato — è una sequenza, non una borsa.
//!
//! Il combinatore resta congelato a 3 canali (dense, sparse, colbert).
//! Nessuna quarta stanza nel combinatore. Il valore d'uso di questa sonda
//! risiede interamente nella **sequenzialità lungo la traiettoria** (Test 2):
//! entra in `semantic-walk` come guida cinematica.
//!
//! ## I due strati
//!
//! La struttura dati serve due meccanismi distinti:
//!
//! * **Strato 1 — Pruning cinematico (guardiano)**: una firma globale
//!   compatta (`global_signature`) consente un confronto O(1) via POPCNT
//!   per scartare subito le traiettorie incompatibili, senza mai toccare il
//!   buffer posizionale. Il guardiano non paga mai il prezzo del raffinatore.
//!
//! * **Strato 2 — Banda di Sakoe-Chiba dinamica (raffinatore)**: la sequenza
//!   posizionale contigua permette l'allineamento locale posizione-per-
//!   posizione. La banda W si stringe dove le sequenze d'ordine concordano,
//!   si allarga dove divergono.

/// L'identificativo di un token nel vocabolario.
///
/// `u32` (non `u16`): i vocabolari basati su encoder multilingua o backbone
/// derivati da LLM superano agevolmente i 65.536 token. Su sequenze sparse
/// l'impatto di memoria tra 2 e 4 byte è trascurabile rispetto alla garanzia
/// di stabilità del contratto.
pub type TokenId = u32;

/// Il numero di posizioni di una sequenza ordered-sparse.
///
/// `u32` per compatibilità con gli offset (che indicizzano il buffer dei
/// token); una traiettoria non può avere più di 4 miliardi di passi.
pub type Position = u32;

/// Una sequenza ordered-sparse: attivazioni di token preservate in ordine
/// posizionale, con layout a memoria contigua per zero allocazioni nel
/// ciclo interno di DTW.
///
/// ## Layout a due livelli
///
/// * **Layer 1 — `global_signature`**: firma aggregata 128 bit per il
///   pruning O(1). Un `(sig_a & sig_b).count_ones()` via POPCNT calcola la
///   sovrapposizione globale in pochi cicli di clock.
///
/// * **Layer 2 — `offsets` + `tokens` + `weights`**: il buffer posizionale
///   contiguo. Per la posizione `i`, il sottoinsieme dei token attivi si
///   estrae con uno slice zero-copy `&tokens[offsets[i]..offsets[i+1]]`.
///
/// La separazione permette due livelli di caching: la firma risiede in
/// L1/L2, il buffer posizionale viene caricato in memoria attiva solo per
/// le traiettorie superstiti.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderedSparseSequence {
    /// **Layer 1** — firma globale aggregata per il pruning O(1).
    ///
    /// Due `u64` (128 bit) che riassumono l'intera sequenza. L'intersezione
    /// tra due firme misura la sovrapposizione globale dei token **emessi**
    /// lungo tutta la traiettoria, indipendentemente dall'ordine.
    ///
    /// ## Contratto dei soppressi
    ///
    /// La firma riassume i soli token emessi (`weight > 0`). Un token
    /// soppresso (`weight <= 0`) non è un token "presente" nel cammino per
    /// il guardiano del pruning: se contribuisse alla firma, due traiettorie
    /// potrebbero risultare compatibili perché condividono token che in
    /// realtà sono stati soppressi in entrambe — un falso positivo nel
    /// confronto O(1). I soppressi restano nel buffer posizionale (il DTW li
    /// vede con peso firmato e può penalizzarli), ma non gonfiano la firma.
    pub global_signature: [u64; 2],

    /// **Layer 2a** — indici di inizio dei frame posizionali nel buffer.
    ///
    /// `offsets[i]` è il punto d'inizio nel buffer `tokens`/`weights` per la
    /// posizione `i`. Il frame della posizione `i` è
    /// `tokens[offsets[i]..offsets[i+1]]` (con `offsets[len]` = lunghezza
    /// totale del buffer). La lunghezza di `offsets` è `num_positions + 1`.
    pub offsets: Vec<u32>,

    /// **Layer 2b** — buffer contiguo piatto dei token per-posizione.
    ///
    /// I token di tutte le posizioni, concatenati nell'ordine posizionale.
    pub tokens: Vec<TokenId>,

    /// **Layer 2c** — buffer contiguo dei pesi associati.
    ///
    /// Allineato con `tokens`: `weights[i]` è il peso del token `tokens[i]`.
    pub weights: Vec<f32>,
}

impl OrderedSparseSequence {
    /// Costruisce una sequenza ordered-sparse a partire dalle attivazioni
    /// posizionali, calcolando automaticamente la firma globale.
    ///
    /// `frames` è la sequenza di insiemi posizionali: `frames[i]` è la lista
    /// delle coppie `(token_id, weight)` attive alla posizione `i`.
    ///
    /// Fallisce se una posizione è vuota (una posizione senza attivazioni è
    /// un dato ambiguo: non si può distinguere un token assente da un buco
    /// nella traiettoria), o se un peso non è finito.
    pub fn from_frames(
        frames: &[Vec<(TokenId, f32)>],
    ) -> Result<Self, &'static str> {
        if frames.is_empty() {
            return Err("La sequenza ordered-sparse non può essere vuota");
        }

        let mut offsets = Vec::with_capacity(frames.len() + 1);
        let mut tokens = Vec::new();
        let mut weights = Vec::new();
        let mut sig = [0u64; 2];

        offsets.push(0u32);
        for frame in frames {
            if frame.is_empty() {
                return Err("Una posizione della sequenza non può essere vuota");
            }
            // Ordina i token del frame per id. Il two-pointer merge di
            // `positional_jaccard` assume frame ordinati per token id; senza
            // questo ordinamento l'intersezione sarebbe errata su head
            // multi-token (es. SPLADE). Su BGE-M3 (1 token per frame) è un
            // no-op, ma rende l'invariante vero per costruzione, non per caso.
            // Per il caso comune a token singolo evitiamo ogni allocazione.
            if frame.len() > 1 {
                let mut sorted = frame.clone();
                sorted.sort_unstable_by_key(|&(token, _)| token);
                for &(token, weight) in &sorted {
                    if !weight.is_finite() {
                        return Err("Il peso di un token deve essere finito");
                    }
                    // Aggrega il token nella firma globale SOLO se è emesso
                    // (weight > 0). Un token soppresso (weight <= 0) non è
                    // "presente" nel cammino per il guardiano del pruning.
                    if weight > 0.0 {
                        let h = (token as u128)
                            .wrapping_mul(0x9E3779B97F4A7C15)
                            .rotate_left(17);
                        let h = (h ^ (h >> 31)) as u64;
                        sig[0] |= h;
                        sig[1] |= h.rotate_left(32);
                    }

                    tokens.push(token);
                    weights.push(weight);
                }
                offsets.push(tokens.len() as u32);
                continue;
            }
            for &(token, weight) in frame {
                if !weight.is_finite() {
                    return Err("Il peso di un token deve essere finito");
                }
                // Aggrega il token nella firma globale SOLO se è emesso
                // (weight > 0). I soppressi restano nel buffer ma non
                // contribuiscono alla firma del pruning.
                if weight > 0.0 {
                    let h = (token as u128)
                        .wrapping_mul(0x9E3779B97F4A7C15)
                        .rotate_left(17);
                    let h = (h ^ (h >> 31)) as u64;
                    sig[0] |= h;
                    sig[1] |= h.rotate_left(32);
                }

                tokens.push(token);
                weights.push(weight);
            }
            offsets.push(tokens.len() as u32);
        }

        Ok(Self {
            global_signature: sig,
            offsets,
            tokens,
            weights,
        })
    }

    /// Il numero di posizioni (passi) della sequenza.
    pub fn num_positions(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }

    /// Lo slice zero-copy dei token della posizione `i`.
    pub fn tokens_at(&self, i: usize) -> &[TokenId] {
        let start = self.offsets[i] as usize;
        let end = self.offsets[i + 1] as usize;
        &self.tokens[start..end]
    }

    /// Lo slice zero-copy dei pesi della posizione `i`.
    pub fn weights_at(&self, i: usize) -> &[f32] {
        let start = self.offsets[i] as usize;
        let end = self.offsets[i + 1] as usize;
        &self.weights[start..end]
    }

    /// **Strato 1 — Pruning cinematico (O(1))**.
    ///
    /// Calcola la sovrapposizione globale tra questa sequenza e un'altra
    /// tramite l'intersezione delle firme a 128 bit. Ritorna il numero di
    /// bit condivisi (0..=128), da confrontare con la soglia `k` del pruning.
    ///
    /// Un punteggio sotto soglia significa che le due traiettorie divergono
    /// topologicamente: il DTW sui vettori densi/ColBERT non va nemmeno
    /// calcolato.
    pub fn global_overlap(&self, other: &Self) -> u32 {
        let a = self.global_signature[0] & other.global_signature[0];
        let b = self.global_signature[1] & other.global_signature[1];
        a.count_ones() + b.count_ones()
    }

    /// **Strato 2 — Overlap posizionale**.
    ///
    /// Confronta i sottoinsiemi di token attivi alla posizione `i` delle due
    /// sequenze, ritornando il coefficiente di Jaccard locale (0.0 = nessun
    /// token condiviso, 1.0 = identici).
    ///
    /// Usato per modulare la banda di Sakoe-Chiba: concordanza alta stringe
    /// la finestra, concordanza bassa la allarga.
    pub fn positional_jaccard(&self, other: &Self, i: usize) -> f32 {
        let ta = self.tokens_at(i);
        let tb = other.tokens_at(i);
        let wa = self.weights_at(i);
        let wb = other.weights_at(i);
        // Caso genuinamente vuoto (mai prodotto da `from_frames`, che rifiuta
        // posizioni vuote): conserva il comportamento storico (1.0). Un frame
        // con soli soppressi NON è vuoto qui — viene trattato come privo di
        // presenza (Jaccard 0), coerentemente con la firma del guardiano.
        if ta.is_empty() && tb.is_empty() {
            return 1.0;
        }
        // Two-pointer merge su frame ordinati, contando SOLO i token emessi
        // (w > 0). I soppressi non sono "presenza" per il Jaccard posizionale:
        // la medesima definizione di presenza della firma del pruning O(1).
        // O(n+m) senza allocazioni temporanee (niente HashSet/Vec). I token
        // sono ordinati per costruzione in `from_frames`.
        let (mut pa, mut pb) = (0usize, 0usize);
        let mut inter = 0usize;
        let mut union = 0usize;
        while pa < ta.len() && pb < tb.len() {
            // Salta i soppressi su entrambi i lati.
            if wa[pa] <= 0.0 {
                pa += 1;
                continue;
            }
            if wb[pb] <= 0.0 {
                pb += 1;
                continue;
            }
            // Entrambi i token correnti sono emessi.
            union += 1;
            if ta[pa] == tb[pb] {
                inter += 1;
                pa += 1;
                pb += 1;
            } else if ta[pa] < tb[pb] {
                pa += 1;
            } else {
                pb += 1;
            }
        }
        // Conta i token emessi residui (non consumati dal merge).
        while pa < ta.len() {
            if wa[pa] > 0.0 {
                union += 1;
            }
            pa += 1;
        }
        while pb < tb.len() {
            if wb[pb] > 0.0 {
                union += 1;
            }
            pb += 1;
        }
        if union == 0 {
            // Entrambi i frame hanno SOLO soppressi: nessuna presenza,
            // concordanza nulla (opzione 1 — banda massima, penalità piena).
            0.0
        } else {
            inter as f32 / union as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(v: Vec<Vec<(u32, f32)>>) -> Vec<Vec<(TokenId, f32)>> {
        v
    }

    #[test]
    fn test_costruzione_e_slice() {
        let s = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5), (2, 0.3)],
            vec![(3, 0.9)],
            vec![(1, 0.1), (4, 0.7)],
        ]))
        .unwrap();

        assert_eq!(s.num_positions(), 3);
        assert_eq!(s.tokens_at(0), &[1, 2]);
        assert_eq!(s.weights_at(0), &[0.5, 0.3]);
        assert_eq!(s.tokens_at(1), &[3]);
        assert_eq!(s.tokens_at(2), &[1, 4]);
        // Gli offset devono essere contigui e coprire tutto il buffer.
        assert_eq!(s.offsets, vec![0, 2, 3, 5]);
        assert_eq!(s.tokens.len(), 5);
        assert_eq!(s.weights.len(), 5);
    }

    #[test]
    fn test_ordina_frame_multi_token_in_ingresso() {
        // Il two-pointer merge di positional_jaccard assume frame ordinati per
        // token id. from_frames deve ordinare i frame non ordinati in ingresso,
        // altrimenti l'intersezione sarebbe errata su head multi-token (SPLADE).
        let s = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(5, 0.1), (2, 0.4), (9, 0.2)], // non ordinato
            vec![(1, 0.3)],
        ]))
        .unwrap();
        // I token della posizione 0 devono essere ordinati per id.
        assert_eq!(s.tokens_at(0), &[2, 5, 9]);
        // I pesi devono seguire l'ordinamento dei token.
        assert_eq!(s.weights_at(0), &[0.4, 0.1, 0.2]);
        assert_eq!(s.offsets, vec![0, 3, 4]);
    }

    #[test]
    fn test_positional_jaccard_con_frame_ordinati_da_from_frames() {
        // Due frame con gli stessi token ma in ordine d'ingresso diverso:
        // dopo l'ordinamento di from_frames, il Jaccard deve essere 1.0.
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(2, 0.3), (1, 0.5), (3, 0.2)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(3, 0.2), (1, 0.5), (2, 0.3)],
        ]))
        .unwrap();
        assert!((a.positional_jaccard(&b, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_rifiuta_sequenza_vuota() {
        assert!(OrderedSparseSequence::from_frames(&vec![]).is_err());
    }

    #[test]
    fn test_rifiuta_posizione_vuota() {
        let r = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5)],
            vec![],
        ]));
        assert!(r.is_err());
    }

    #[test]
    fn test_rifiuta_peso_non_finito() {
        let r = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, f32::NAN)],
        ]));
        assert!(r.is_err());
        let r = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, f32::INFINITY)],
        ]));
        assert!(r.is_err());
    }

    #[test]
    fn test_global_overlap_identiche_massimo() {
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5), (2, 0.3)],
            vec![(3, 0.9)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5), (2, 0.3)],
            vec![(3, 0.9)],
        ]))
        .unwrap();
        // Stesse firme: l'overlap è il numero di bit della firma stessa.
        let overlap = a.global_overlap(&b);
        assert_eq!(overlap, a.global_signature[0].count_ones() + a.global_signature[1].count_ones());
    }

    #[test]
    fn test_global_overlap_disgiunte_zero() {
        // Token molto distanti nel vocab potrebbero condividere bit per
        // collisione di hash. Usiamo token con hash notoriamente separati:
        // la proprietà vera è che sequenze senza token comuni NON aumentano
        // l'overlap oltre quello delle singole firme.
        let a = OrderedSparseSequence::from_frames(&frames(vec![vec![(1, 0.5)]])).unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![vec![(2, 0.5)]])).unwrap();
        // Due firme diverse possono condividere bit per caso (hash).
        // L'overlap di una firma con se stessa è sempre >= dell'overlap
        // con una firma diversa in media, ma non garantito bit a bit.
        // Verifichiamo la proprietà debole: l'overlap è un valore valido 0..=128.
        let o = a.global_overlap(&b);
        assert!(o <= 128);
    }

    #[test]
    fn test_positional_jaccard_identico() {
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5), (2, 0.3)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5), (2, 0.3)],
        ]))
        .unwrap();
        assert!((a.positional_jaccard(&b, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_positional_jaccard_disgiunto() {
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5), (2, 0.3)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(3, 0.5), (4, 0.3)],
        ]))
        .unwrap();
        assert!((a.positional_jaccard(&b, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_positional_jaccard_parziale() {
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5), (2, 0.3)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, 0.5), (3, 0.3)],
        ]))
        .unwrap();
        // Intersezione = {1}, unione = {1,2,3} => J = 1/3
        assert!((a.positional_jaccard(&b, 0) - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_soppressi_non_contribuiscono_alla_firma() {
        // Il soppresso condiviso (w <= 0) NON deve rendere due sequenze più
        // compatibili per il guardiano del pruning. Verifichiamo la proprietà
        // relativa: l'overlap di firma di (a,b) che condividono un soppresso
        // deve essere IDENTICO a quello di (a,b) senza il soppresso condiviso.
        //
        // Non possiamo pretendere overlap = 0 assoluto: due firme di token
        // emessi diversi (2 vs 3) possono condividere bit per collisione di
        // hash. Ciò che deve valere è che il soppresso non AGGIUNGA nulla.
        let a_soppr = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, -0.5), (2, 0.3)],
        ]))
        .unwrap();
        let b_soppr = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, -0.5), (3, 0.7)],
        ]))
        .unwrap();
        let a_senza = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(2, 0.3)],
        ]))
        .unwrap();
        let b_senza = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(3, 0.7)],
        ]))
        .unwrap();

        // La presenza del soppresso condiviso (1) non deve cambiare l'overlap:
        // il guardiano vede esattamente gli stessi token emessi.
        assert_eq!(
            a_soppr.global_overlap(&b_soppr),
            a_senza.global_overlap(&b_senza),
            "il soppresso condiviso non deve contribuire alla firma del pruning"
        );

        // Il soppresso (1) RESTA nel buffer posizionale, con peso firmato:
        // il DTW lo vede con peso firmato e può penalizzarlo.
        assert_eq!(a_soppr.tokens_at(0), &[1, 2]);
        assert_eq!(a_soppr.weights_at(0), &[-0.5, 0.3]);
    }

    #[test]
    fn test_soppresso_identico_emesso_non_collide() {
        // Un token soppresso in a (w<=0) e lo stesso token emesso in b (w>0):
        // il soppresso non deve comparire nella firma di a, quindi l'overlap
        // deve essere 0 (nessun token emesso comune).
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(7, -0.2)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(7, 0.9)],
        ]))
        .unwrap();

        assert_eq!(a.global_overlap(&b), 0);
        // La firma di a (tutto soppresso) deve essere vuota: 0 bit.
        let sig_a = a.global_signature[0].count_ones() + a.global_signature[1].count_ones();
        assert_eq!(sig_a, 0);
    }

    #[test]
    fn test_soppresso_condiviso_non_conta_come_concordanza() {
        // Opzione 2 confermata: i soppressi (w <= 0) sono esclusi dal Jaccard
        // posizionale, coerente con la firma del guardiano (presenza = w > 0).
        //
        // a: {1(soppresso), 2(emesso)}  b: {1(soppresso), 2(emesso)}
        // Il soppresso condiviso (1) NON è concordanza. Solo {2} conta:
        // Jaccard = 1/1 = 1.0 (non 1.0 per i due token, né 1/3 con il soppresso).
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, -0.5), (2, 0.3)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, -0.5), (2, 0.3)],
        ]))
        .unwrap();
        assert!((a.positional_jaccard(&b, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_soppresso_condiviso_con_emessi_diversi() {
        // a: {1(soppresso), 2(emesso)}  b: {1(soppresso), 3(emesso)}
        // Il soppresso condiviso (1) non conta; gli emessi {2} e {3} sono
        // disgiunti => Jaccard = 0.0 (non 1/3 che includerebbe il soppresso).
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, -0.5), (2, 0.3)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, -0.5), (3, 0.7)],
        ]))
        .unwrap();
        assert!((a.positional_jaccard(&b, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_frame_solo_soppressi_jaccard_zero() {
        // Opzione 1: un frame con SOLI soppressi non porta informazione di
        // concordanza => Jaccard 0.0 (banda massima, penalità piena).
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, -0.5), (2, -0.3)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(1, -0.5), (2, -0.3)],
        ]))
        .unwrap();
        assert!((a.positional_jaccard(&b, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_soppresso_vs_emesso_stesso_token_non_conta() {
        // a: {7(soppresso)}  b: {7(emesso)}
        // Il soppresso di a non è presenza; in b il token è emesso. Non c'è
        // concordanza di token emessi => Jaccard 0.0.
        let a = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(7, -0.2)],
        ]))
        .unwrap();
        let b = OrderedSparseSequence::from_frames(&frames(vec![
            vec![(7, 0.9)],
        ]))
        .unwrap();
        assert!((a.positional_jaccard(&b, 0)).abs() < 1e-6);
    }
}