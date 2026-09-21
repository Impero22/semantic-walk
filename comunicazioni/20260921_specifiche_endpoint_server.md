# Specifiche tecniche — Endpoint per-token del server CrispEmbed

**Data**: 21/09/26
**Richiedente**: Iris (semantic-walk)
**Destinatario**: Federico (implementazione lato server)
**Stato**: proposta — il formato JSON esatto è negoziabile, il *contenuto semantico* è il contratto.

---

## 0. Contesto

Il server CrispEmbed oggi espone due endpoint che non bastano a costruire una
traiettoria per-token:

- `/api/embeddings` → embedding **dense di frase** (mean-pooling), un solo
  vettore per testo.
- `/sparse` → mappa **non ordinata** token→peso.

Per il cammino semantico (semantic-walk) servono i vettori **per-token**,
nell'**ordine esatto** in cui i token compaiono. Questo documento specifica i
due endpoint che il server deve esporre.

Il consumatore lato Iris (`semantic-walk/src/parse.rs`) è già scritto e
testato su dati sintetici. Quando questi endpoint saranno esposti, l'aggancio
sarà immediato: basta sostituire la sorgente dati. Il documento qui sotto
definisce il contratto che `parse.rs` si aspetta.

---

## 1. Endpoint A — Matrice ColBERT per-token

Restituisce, per un testo, la sequenza dei token nell'ordine esatto e la
matrice dei vettori densi per-token.

### 1.1 Richiesta

```
GET /colbert/trajectory?text=<url-encoded>
```

oppure, se si preferisce POST:

```
POST /colbert/trajectory
Content-Type: application/json

{ "text": "il testo da processare" }
```

### 1.2 Risposta (200 OK)

```json
{
  "sequence_id": "id-opzionale-del-fatto",
  "tokens": ["il", "testo", "da", "processare"],
  "embeddings": [
    [0.0123, -0.0456, "...", 0.0789],
    [0.1111, 0.2222, "...", -0.3333],
    "..."
  ],
  "dimension": 1024
}
```

### 1.3 Contratto di coerenza (VINCOLANTE)

1. `tokens.length == embeddings.length` — ogni token ha esattamente una riga.
2. Ogni vettore in `embeddings` ha lunghezza esattamente `dimension` (1024).
3. `embeddings[i]` è il vettore del token `tokens[i]` — **l'ordine conta**:
   è una traiettoria, non una borsa.
4. `dimension` è costante per tutte le righe.

Se una di queste è violata, il consumatore rifiuta il dato come corrotto
(`ParseError::TokenMatriceDisallineati` / `ParseError::DimensioneIncoerente`).

### 1.4 Note

- I vettori sono `f64` (JSON number). Il server può produrli come `f32`
  internamente; il JSON li serializza come numeri.
- Il server espone già il punteggio ColBERT MaxSim via C-ABI
  (`crispembed_colbert_score`). Questo endpoint espone la **matrice grezza**
  per-token, non il punteggio aggregato: serve al DTW del cammino.

---

## 2. Endpoint B — Walk ordered-sparse

Restituisce, per un testo, la sequenza ordinata delle attivazioni sparse:
per ogni posizione, la lista dei token attivi con i loro pesi.

### 2.1 Richiesta

```
GET /sparse/walk?text=<url-encoded>
```

oppure POST:

```
POST /sparse/walk
Content-Type: application/json

{ "text": "il testo da processare" }
```

### 2.2 Risposta (200 OK)

```json
{
  "sequence_id": "id-opzionale-del-fatto",
  "frames": [
    [ {"token": 42, "weight": 0.87}, {"token": 17, "weight": 0.31} ],
    [ {"token": 3,  "weight": 0.95} ],
    [ {"token": 42, "weight": 0.62}, {"token": 99, "weight": 0.44} ]
  ]
}
```

### 2.3 Contratto di coerenza (VINCOLANTE)

1. `frames.length` == numero di posizioni (token) della traiettoria.
2. `frames[i]` è l'insieme delle attivazioni sparse **alla posizione i**.
3. Ogni elemento di un frame è una coppia `(token, weight)`:
   - `token`: intero ≥ 0, identificativo del token nel vocabolario (`u32`).
   - `weight`: numero **finito** (mai NaN, mai infinito).
4. **Nessun frame vuoto**: ogni posizione deve avere almeno un token attivo.
   *(Vedi §4: questo è il punto di contratto aperto.)*

### 2.4 Note

- Il peso è `f32` (JSON number).
- L'ordine dei token *dentro* un singolo frame non è significativo per il
  consumatore (il `positional_jaccard` fonde i frame ordinati con two-pointer
  merge); l'ordine **tra** i frame è invece fondamentale: è la sequenza
  posizionale che guida il cammino.

---

## 3. Coerenza tra i due endpoint

Se gli stessi testi vengono processati da entrambi gli endpoint, il numero di
posizioni del walk (`frames.length`) **deve** coincidere con il numero di
token della matrice ColBERT (`tokens.length`): entrambi rappresentano la
stessa traiettoria, vista nel canale denso e in quello sparso.

Questa coerenza è verificata a valle nel DTW (`align_with_ordered_sparse`
ritorna `Err` se le sequenze dense e le guide sparse sono disallineate).

---

## 4. Punto di contratto APERTO — posizioni vuote

`OrderedSparseSequence::from_frames` (lato consumatore) **rifiuta** le
posizioni vuote: un token senza attivazioni sparse è considerato ambiguo
(non si può distinguere un token assente da un buco nella traiettoria).

Nel mondo reale, però, un token può non avere attivazioni sparse a una data
posizione. Le opzioni sono:

- **A. Saltare le posizioni vuote** — il walk ha meno frame dei token densi;
  la corrispondenza posizionale va ricostruita (più complesso, rompe la
  biiezione con la matrice densa).
- **B. Token speciale di "nessuna attivazione"** — es. un token riservato
  (token = 0, o un sentinel) con peso 0, per mantenere la biiezione
  posizionale.
- **C. Frame vuoto consentito** — ammorbidire il contratto lato consumatore
  per accettare `frames[i] == []` e trattarlo come "nessuna attivazione".

**Questa decisione è del contratto condiviso con Camillo**: non la decido da
sola. Finché non è presa, il comportamento lato consumatore è il rifiuto
(`ParseError::PosizioneVuota`).

---

## 5. Riassunto operativo per il server

| # | Endpoint | Metodo | Input | Output |
|---|----------|--------|-------|--------|
| A | `/colbert/trajectory` | GET/POST | `text` | `sequence_id`, `tokens[]`, `embeddings[][]`, `dimension` |
| B | `/sparse/walk` | GET/POST | `text` | `sequence_id`, `frames[]` di `{token, weight}` |

Vincoli non negoziabili:
- Ordine dei token preservato (traiettoria, non borsa).
- Coerenza dimensionale interna (tutte le righe = `dimension`).
- Coerenza posizionale tra i due endpoint (`frames.length == tokens.length`).
- Pesi finiti (mai NaN/inf).
- Posizioni vuote: in attesa di decisione condivisa (§4).

---

## 6. Stato lato consumatore

- `semantic-walk/src/parse.rs` scritto e testato (8 test verdi) sul contenuto
  semantico qui specificato.
- `semantic-walk/src/ingest.rs` definisce `CrispTrajectory` (canale denso).
- `semantic-walk/src/ordered_sparse.rs` definisce `OrderedSparseSequence`
  (canale sparso ordinato).
- Commit `a685f58` (parse), `4be4b4c` (ordered-sparse).

Quando gli endpoint saranno esposti, resta da scrivere solo il client HTTP
(che vivrà fuori dal crate puro `semantic-walk`), allineato al formato JSON
finale scelto.
