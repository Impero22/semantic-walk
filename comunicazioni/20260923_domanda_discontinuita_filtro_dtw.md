# Domanda a Camillo — Discontinuità strutturale tra filtro del walk e traiettoria densa

**Data**: 23/09/26 00:10 (aggiornata 23/09/26 02:06 con la radice: token speciali)
**Autore**: Iris

## Contesto

Stavo completando le asserzioni verificabili dello scheletro test a 3 livelli (commit `e893dc0`) quando ho verificato come il DTW consuma le sequenze ordered-sparse. Ho trovato una discontinuità nel contratto che abbiamo approvato insieme.

## La radice: i token speciali `<s>`/`</s>` (verificata sul codice reale)

Ho verificato la radice sul codice sorgente dell'embedder (`embedder/src/api_multivec.cpp`, `crispembed_encode_tokens`):

1. `tokenize_text` produce i token con `<s>`/`</s>` (CLS/BOS in posizione 0, SEP/EOS in coda).
2. `trim_padding` rimuove SOLO il padding (attn_mask=0) — **non tocca i token speciali**.
3. `run_encoder_raw` emette `raw_T = T` = tutti i token dopo il trim.
4. `last_token_ids.assign(tokens.ids.begin(), tokens.ids.begin() + raw_T)` — prende TUTTI i token id, senza filtrare gli speciali.

→ La biiezione 7==7 verificata sul campo includeva i token speciali. Il walk riceve `<s>`/`</s>` come passi veri.

## Conferma dal contratto ufficiale (docs/PER_TOKEN_ENDPOINTS.md)

Il documento del server lo dice esplicitamente:

1. **`/colbert/encode?tokens=1`** include i token speciali — "Cleaning is the consumer's job" (la pulizia è responsabilità del consumatore, non del server).
2. **`/ordered-sparse?format=frames`** tiene la posizione 0 = `<s>` come frame reale e conserva i soppressi (status 1) — "a suppressed token is a verdict, not an absence".
3. **Biiezione garantita**: `frames.length == n_tokens` per lo stesso testo. Il server droppa solo il padding (status 2), tiene tutto il resto.

Il server è quindi coerente col suo contratto: emette speciali e soppressi inclusi. L'anomalia è nel nostro crate, a valle.

## Il problema (nel crate)

La catena attuale è:

1. **`verifica_coerenza_posizionale`** asserisce `frames.length == colbert n_tokens` → per "gatto dorme": **7 == 7**. La biiezione alla **fonte** regge.
2. **`walk_to_sequence`** (parse.rs:182) applica il **filtro d'igiene** → scarta i passi con `st != 0` (soppressi) e `id < 4` (token speciali), restano **5** passi significativi.
3. **`colbert_to_trajectory`** (parse.rs:129) NON applica alcun filtro → **7** righe (tutti i token, speciali inclusi).
4. **`align_with_ordered_sparse`** (dtw.rs:175) richiede `sparse_a.num_positions() == seq_a.len()` → **5 ≠ 7** → fallisce con `"Disallineamento tra sequenze dense e guide ordered-sparse"`.

Non esiste alcun meccanismo che filtri la traiettoria densa per allinearla ai passi significativi del walk (verificato in ingest.rs/lib.rs).

**In sintesi**: la biiezione alla fonte regge (7==7), ma la nostra igiene interna la distrugge (5 vs 7). Il server è coerente; il crate rompe la corrispondenza che il contratto garantisce.

## Le due opzioni

**(a) Filtrare anche la traiettoria densa** sugli stessi indici del walk. Dato che walk e matrice ColBERT condividono la posizione assoluta `i` (garantita da `verifica_coerenza_posizionale`), serve una funzione che selezioni gli embedding agli indici dove il walk ha `st==0 && id>=4`. Il DTW allineerebbe così due sequenze filtrate coerentemente (5 vs 5). Il mio test Level 2 (`num_positions() == 5`) è coerente con questa opzione.

**(b) Non filtrare il walk a monte** (tenere i token speciali come posizioni), spostando il filtro d'igiene solo nel confronto `positional_jaccard`. Ma questo confligge con l'asserto del Level 2 (`num_positions() == 5`) e cambia il significato di "posizione" nel cammino.

## Domanda

Quale delle due opzioni preferisci, e perché? In particolare:
- Se (a): la funzione di filtro denso vive in `parse.rs` (accanto a `walk_to_sequence`), o in un modulo dedicato? E deve riusare la stessa logica di `walk_filtra_igiene` per garantire che i due filtri siano sempre coerenti?
- Se (b): come gestiamo i token speciali nel confronto `positional_jaccard` — li ignoriamo ma teniamo la posizione, o li pesiamo a zero?

## Nota

Il finding è già documentato nel tracciamento (`20260915_tracciamento_progetto.md`, sezione 23/09 00:05). Non ho toccato il contratto: attendo la tua decisione prima di agganciare i dati reali.
