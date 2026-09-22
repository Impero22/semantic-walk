# Verifica del contratto di ingest contro vetta-semantic (.18)

**Data:** 2026-09-22 — 20:05
**Autore:** Iris
**Stato:** verificato, in attesa dell'adattatore JSON→RawWalk/RawColbertTrajectory

---

## Contesto

Federico ha sostituito il crispembed della `.18` (dedicato ai nostri test) con la
versione comprendente le modifiche che avevo richiesto, e mi ha consegnato la
documentazione completa: `VETTA_SEMANTIC_API.md` (living document, aperto il
giorno in cui il taglio semantic-only è andato in produzione su Altair `.18`).

Questo documento registra la verifica del ponte di ingest (`semantic-walk/src/parse.rs`)
contro gli endpoint esposti, e la scelta di disegno per l'adattatore mancante.

> **⚠️ Nota operativa:** il server aggiornato è attualmente **solo sulla `.18`**
> (porta 8091, live dal 2026-09-22 19:54). La macchina `.5` (produzione framework)
> ha lo stesso layout ma lo swap è in attesa di scheduling. I test sui dati reali
> vanno fatti contro la `.18`.

---

## Cosa combacia (verifica concettuale)

### 1. Matrice ColBERT per-token → `RawColbertTrajectory`

Endpoint: `POST /colbert/encode?tokens=1`

```jsonc
{ "results": [ { "multivector": [[0.056, ...], ...], "n_tokens": 5,
                 "dim": 1024, "tokens": ["<s>", "▁il", "▁gat", "to", "</s>"] } ] }
```

- `tokens` (1:1 con le righe della matrice) → il campo `tokens: Vec<String>` di
  `RawColbertTrajectory`.
- `multivector` (matrice T×1024) → il campo `embeddings: Vec<Vec<f64>>`.
- `n_tokens` → la verifica `TokenMatriceDisallineati` (righe == token).
- `dim` → la verifica `DimensioneIncoerente` (righe tutte della stessa D).

### 2. Cammino ordered-sparse → `RawWalk`

Endpoint: `POST /ordered-sparse?format=frames` (forma flat di default)

```jsonc
{ "results": [ { "n": 5,
    "ids":      [0, 211, 27294, 45, 2],
    "weights":  [0.196189, 0.197522, 0.277092, -0.013, 0.201],
    "positions":[0, 1, 2, 3, 4],
    "status":   [0, 0, 0, 1, 0] } ] }
```

- `ids` → `ids: Vec<TokenId>` (token id XLM-R, ~250k vocabolario).
- `weights` (firmati, `w ≤ 0` = verdetto) → `w: Vec<WalkWeight>`.
- `positions` (strettamente crescente) → `pos: Vec<u32>` — la verifica di stretta
  crescenza in `walk_to_sequence` è coperta.
- `status` (0 emesso / 1 soppresso / 2 padding) → `st: Vec<u8>` — il filtro
  d'igiene `walk_filtra_igiene` tiene `st == 0` e `id >= 4`.

### 3. La biiezione (il vincolo più importante)

Il documento garantisce: **`frames.length == colbert n_tokens`** (il frames mode
regruppa per posizione e droppa il padding). Questo è esattamente ciò che
`verifica_coerenza_posizionale` controlla con `CoerenzaPosizionaleFallita`:
il numero di passi emessi dal walk (dopo il filtro) deve coincidere con il
numero di righe della matrice ColBERT. Entrambi rappresentano la stessa
traiettoria, vista nel canale denso e in quello sparso.

---

## Il pezzo mancante: l'adattatore JSON

**Scoperta chiave:** `semantic-walk` è un crate **puro** — niente reqwest, niente
serde, nessun client HTTP (Cargo.toml dipende solo da semantic-combiner e
semantic-gate). `RawWalk` e `RawColbertTrajectory` sono tipi intermedî, ma **chi
li costruisce dalla risposta JSON del server non esiste ancora**. È l'adattatore
che va scritto — il pezzo mancante dell'aggancio ai dati reali.

### Scelta di disegno (deciso da Iris)

1. **Forma flat vs frames.** Il server espone entrambe. La flat è
   `{n, ids, weights, positions, status}` — quattro array paralleli, un token per
   posizione. Il frames mode regruppa per posizione. Il nostro `walk_to_sequence`
   assume "un solo token per posizione" (`vec![(id, w)]` per frame), quindi la
   **flat** è la forma naturale per noi. L'adattatore consumerà la flat.
2. **Mapping dei nomi dei campi.** Il server chiama i campi
   `weights/positions/status`; il nostro `RawWalk` li chiama `w/pos/st`.
   L'adattatore farà il mapping esplicito.

### Contratto che l'adattatore deve rispettare

- Deserializzare `{n, ids, weights, positions, status}` (flat) in `RawWalk`.
- Deserializzare `{multivector, n_tokens, dim, tokens?}` in `RawColbertTrajectory`
  (`tokens` opzionale: assente → placeholder da `n_tokens`; presente e disallineato
  → `ConteggioTokenIncoerente`).
- Dopo la costruzione, invocare `verifica_coerenza_posizionale` per garantire la
  biiezione prima di procedere con l'allineamento DTW.

---

## Prossimi passi

1. Scrivere l'adattatore JSON→RawWalk/RawColbertTrajectory (mio compito).
2. Testarlo contro la `.18` (unico deploy con il server aggiornato).
3. Integrazione del percorso candidate-level 3D sui dati reali.
4. Push di tutti i file in sospeso (attende il token di Camillo).
---

## Aggiornamento 20:18 — adattatore scritto e verde

**Autore:** Iris

Il pezzo mancante individuato nel documento — l'adattatore JSON→RawWalk/RawColbertTrajectory — è ora scritto e testato.

> **⚠️ Correzione 22/09 20:44 (verifica sul campo):** la sezione 1 documenta
> l'endpoint con `?tokens=1` — ed è corretta: **con** il parametro il campo
> `tokens` c'è. La verifica reale su `.18:8091` conferma il comportamento opt-in
> documentato dal Coder:
> - **Senza** `?tokens=1` → chiavi `{dim, multivector, n_tokens}`, **nessun `tokens`**.
> - **Con** `?tokens=1` → chiavi `{dim, multivector, n_tokens, tokens}`.
>
> Di conseguenza `tokens` è **opzionale** nel contratto: l'adattatore lo rende
> tale (placeholder da `n_tokens` quando assente; se presente e disallineato →
> `ConteggioTokenIncoerente`). La verità di conteggio resta `n_tokens`, e la
> coerenza `tokens.len() == embeddings.len()` è garantita dall'allineamento a
> `n_tokens`. Il placeholder non deve essere scambiato a valle per pezzi lessicali
> veri: quando il campo manca, i token reali semplicemente non ci sono.
> (Commit `12585d8`.)

### `semantic-walk/src/adapter.rs` (nuovo modulo)

- **Tipi serde** che mappano i nomi esatti della risposta del server:
  - `ColbertResultJson` → `{multivector, n_tokens, dim, tokens}` (rename serde).
  - `WalkResultJson` → `{n, ids, weights, positions, status}` (rename serde: weights→w, positions→pos, status→st).
  - `ServerResponse<T>` → l'involucro `{results: [...]}`.
- **Funzioni di conversione**:
  - `colbert_json_to_raw` → verifica `n_tokens == embeddings.len()`; se `tokens`
    presente, verifica anche `tokens.len() == n_tokens` (`ConteggioTokenIncoerente`).
  - `walk_json_to_raw` → verifica `n == ids.len() == w.len() == pos.len() == st.len()` (`ConteggioIncoerente`).
  - Entrambe gestiscono `results` vuoto (`RisultatoAssente`).
- **Anello chiuso**: il test `adapter_poi_parse_anello_completo` costruisce RawWalk/RawColbertTrajectory dall'JSON, invoca `verifica_coerenza_posizionale` (biiezione), poi `colbert_to_trajectory` e `walk_to_sequence` → il contratto del cammino è raggiunto.

### Stato

- `semantic-walk` dipende ora da `serde` (dipendenza normale, non dev).
- `lib.rs`: aggiunto `pub mod adapter;`.
- **58 test verdi** nel crate (7 nuovi per l'adapter, incluso l'anello completo).
- Da fare: test contro la `.18` (unico deploy col server aggiornato), poi integrazione del percorso candidate-level 3D sui dati reali.

## AGGIORNAMENTO 22/09 21:00 — Anello chiuso sul campo

L'esempio `end_to_end_server.rs` chiude il cerchio: i dati JSON reali catturati
dal server (matrice ColBERT per-token + walk ordered-sparse flat) vengono
deserializzati, trasformati in `Raw*` e verificati.

**Scoperta sul campo (disallineamento di contratto):** `verifica_coerenza_posizionale`
applicava il filtro d'igiene (`st==0 AND id>=4`), confrontando i soli passi
significativi (5) con le righe ColBERT totali (7) → falliva sul dato reale.

**La biiezione documentata è `frames.length == colbert n_tokens`:** il frames
mode include i token speciali (droppa solo il padding), la matrice ColBERT ha
una riga per ogni token. Il filtro d'igiene è un concetto del crate per il DTW
a valle, non della verifica dei due canali alla fonte.

**Fix:** la verifica ora conta `raw_walk.ids.len()` (tutti i passi). 2 test nuovi
(il vecchio `conta_solo_emessi` codificava il comportamento sbagliato — i test
verdi confermano ciò che chiedi loro, serve il dato reale per vedere l'errore).
Commit 270443f.
