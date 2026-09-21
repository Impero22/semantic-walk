# Specifiche tecniche — Endpoint per-token del server CrispEmbed (REV 2)

**Data**: 21/09/26 (rev 2.1 — allineata alla conferma del Coder)
**Richiedente**: Iris (semantic-walk)
**Destinatario**: Federico (implementazione lato server)
**Stato**: proposta — il formato JSON esatto è negoziabile, il *contenuto semantico* è il contratto.

> **Rev 2.1 (21/09, ~19:20)** — aggiornamento dopo la conferma del Coder:
> il contratto autoritativo (forme esatte, esempi verificati live su BGE-M3,
> recap degli endpoint) è in **`docs/PER_TOKEN_ENDPOINTS.md`** lato server.
> Questa rev 2.1 allinea la parte consumatore: il fix dei soppressi in
> `from_frames` è implementato e verde, il punto aperto §5 è deciso.

---

## 0. Rettifica rispetto alla rev 1 (21/09, ore ~17)

La rev 1 partiva da una premessa **falsa**: che il server "non basta a costruire
una traiettoria per-token". Dopo aver trovato il riferimento corretto
(`/home/iris/documenti/il-percorso-lessicale-walk.md`, scritto dal Coder il
7 settembre), la premessa va corretta:

* Il **`walk` esiste già** nel payload dei fatti della memoria, con formato
  `ids/w/pos/st` — quattro array della stessa lunghezza `T`, allineati per
  indice, una voce per occorrenza (token del testo), con `pos` strettamente
  crescente.
* Il **colbert** è già salvato in Qdrant come **multivector** (matrice T×1024,
  una riga per token nell'ordine del testo).
* L'endpoint B come immaginato nella rev 1 (`/sparse/walk` con `frames` per
  posizione) è **ridondante**: il walk esiste già nel payload.

Questa rev 2 separa ciò che è **già implementato** da ciò che è una **vera
aggiunta**, così l'implementazione parte dalle aggiunte reali.

---

## 1. Ciò che è GIÀ implementato (verificare, non rifare)

### 1.1 Il `walk` nel payload dei fatti

Formato reale (dal documento del Coder, 7/09):

| campo | significato |
|-------|-------------|
| `ids` | token id XLM-R (~250k vocabolario) toccato in quel passo |
| `w`   | peso di quell'occorrenza, **firmato** (`w > 0` conta; `w ≤ 0` è un *verdetto*, non rumore) |
| `pos` | posizione del token nell'input dell'encoder, **strettamente crescente** |
| `st`  | `0` emesso · `1` soppresso · `2` padding |

Quattro array della stessa lunghezza `T`, allineati per indice. Una voce per
**occorrenza**, in ordine di apparizione, senza dedup: la stessa parola contata
due volte esce due volte con pesi diversi.

**Filtro d'igiene obbligatorio** per qualsiasi calcolo:

```
tieni i passi con st == 0 e id >= 4
```

(il padding è `st == 2`; i token speciali `id 0/2/3` escono `st == 0` con peso
positivo ma vanno gittati via — in ogni cammino osservato la posizione 0 ha
`id 0` e peso ~0.18. Se ti fidi solo dello `st`, conti della punteggiatura.)

### 1.2 Il colbert in Qdrant

Il colbert è salvato in Qdrant come **multivector** (matrice T×1024, una riga
per token nell'ordine del testo). **Da verificare**: che il colbert conservi
l'ordine delle righe come il writer l'ha emesso (i multivector non dovrebbero
subire la canonizzazione della sparse, ma "non dovrebbero" non è "verificato"),
e che il conteggio delle righe torni coi `pos` del `walk`.

---

## 2. La VERA aggiunta — matrice ColBERT per-token come servizio

Il colbert è salvato in Qdrant, ma va esposto come **endpoint** per servire il
DTW del cammino (che oggi lavora su dati sintetici). Questo è il punto che
manca davvero.

### 2.1 Richiesta

```
GET /colbert/trajectory?text=<url-encoded>
```

oppure, se si preferisce POST:

```
POST /colbert/trajectory
Content-Type: application/json

{ "text": "il testo da processare" }
```

### 2.2 Risposta (200 OK)

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

### 2.3 Contratto di coerenza (VINCOLANTE)

1. `tokens.length == embeddings.length` — ogni token ha esattamente una riga.
2. Ogni vettore in `embeddings` ha lunghezza esattamente `dimension` (1024).
3. `embeddings[i]` è il vettore del token `tokens[i]` — **l'ordine conta**:
   è una traiettoria, non una borsa.
4. `dimension` è costante per tutte le righe.

Se una di queste è violata, il consumatore rifiuta il dato come corrotto
(`ParseError::TokenMatriceDisallineati` / `ParseError::DimensioneIncoerente`).

### 2.4 Note

- I vettori sono `f64` (JSON number). Il server può produrli come `f32`
  internamente; il JSON li serializza come numeri.
- Il server espone già il punteggio ColBERT MaxSim via C-ABI
  (`crispembed_colbert_score`). Questo endpoint espone la **matrice grezza**
  per-token, non il punteggio aggregato: serve al DTW del cammino.

---

## 3. Il walk come servizio — domanda aperta

Il walk esiste nel **payload dei fatti** (già scritto in memoria da
`fatti_sync` dal 7/09). Domanda per Federico: serve anche come **endpoint** per
testi arbitrari (non ancora fatti), oppure il consumatore legge il walk
direttamente dal payload dei fatti già in memoria?

Se serve come endpoint, il formato è il **walk reale** (`ids/w/pos/st`), non il
`frames` della rev 1. Il consumatore lato Iris si allinea a questo formato.

---

## 4. Coerenza tra i due canali

Il numero di posizioni del walk (`pos` massimo + 1, dopo il filtro d'igiene)
**deve** coincidere con il numero di righe della matrice ColBERT: entrambi
rappresentano la stessa traiettoria, vista nel canale denso e in quello sparso.

Questa coerenza è verificata a valle nel DTW (`align_with_ordered_sparse`
ritorna `Err` se le sequenze dense e le guide sparse sono disallineate).

---

## 5. Punto di contratto APERTO — soppressione e posizioni vuote

> **DECISO (rev 2.1)** — il Coder conferma che il server emette `frames[]` =
> tutte le posizioni non-padding (soppressi inclusi, peso firmato) per il DTW
> e la biiezione, con `status[]` parallelo (0 emesso · 1 soppresso · 2 padding).
> Il filtro è una scelta di lettura lato consumatore: **il DTW vede i soppressi
> con peso firmato e può penalizzarli** (restano nel buffer posizionale),
> mentre **la firma del pruning O(1) li esclude** (non sono token "presenti").
> Questa separazione è ora vera per costruzione lato consumatore: il fix in
> `from_frames` aggrega nella `global_signature` solo i token emessi
> (`weight > 0.0`), e due test verificano la proprietà relativa (il soppresso
> condiviso non AGGIUNGE nulla all'overlap di firma).

Il walk reale usa `st` per marcare i passi: `st == 0` (emesso), `st == 1`
(soppresso), `st == 2` (padding). Il filtro d'igiene tiene `st == 0` e `id >= 4`.

**Decisione condivisa (Coder + Iris):** i passi soppressi (`st == 1`) non si
ignorano del tutto — restano nel buffer posizionale con peso firmato (il DTW li
vede e può penalizzarli), ma non contribuiscono alla firma del pruning O(1).
Nota onesta del Coder per il futuro: lo `status` è per-frame; su BGE-M3
(1 token/frame) è esatto per token. Se si passa a una head multi-token (SPLADE),
andrà emesso lo status per-token dentro ogni frame.

---

## 6. Riassunto operativo per il server

| # | Cosa | Stato | Azione |
|---|------|-------|--------|
| 1 | `walk` nel payload dei fatti (`ids/w/pos/st`) | ✅ già implementato | verificare, non rifare |
| 2 | colbert multivector in Qdrant | ✅ già implementato | **verificare l'ordine delle righe** |
| 3 | endpoint `/colbert/trajectory` (matrice per-token) | ✅ implementato dal Coder | contratto in `docs/PER_TOKEN_ENDPOINTS.md` |
| 4 | walk come endpoint per testi arbitrari | ✅ implementato (`/ordered-sparse?format=frames`) | contratto in `docs/PER_TOKEN_ENDPOINTS.md` |
| 5 | `status[]` parallelo a `frames[]` | ✅ implementato | per la biiezione e la scelta di lettura |

Vincoli non negoziabili:
- Ordine dei token preservato (traiettoria, non borsa).
- Coerenza dimensionale interna (tutte le righe = `dimension`).
- Coerenza posizionale tra walk e colbert (`pos` ↔ righe).
- Pesi finiti (mai NaN/inf).
- Soppressi (`st == 1`): restano nel buffer posizionale (DTW li penalizza),
  esclusi dalla firma del pruning (§5 — DECISO).

---

## 7. Stato lato consumatore (allineato)

- `semantic-walk/src/parse.rs` — in allineamento al formato reale del walk
  (`ids/w/pos/st`), non più al formato `frames` immaginato nella rev 1.
- `semantic-walk/src/ingest.rs` — definisce `CrispTrajectory` (canale denso).
- `semantic-walk/src/ordered_sparse.rs` — definisce `OrderedSparseSequence`
  (canale sparso ordinato), allineato alla semantica `st`/`pos`. **Fix
  implementato e verde (21/09, ~19:20):** la `global_signature` aggrega solo i
  token emessi (`weight > 0.0`); i soppressi restano nel buffer posizionale con
  peso firmato. Due test verificano la proprietà relativa (il soppresso
  condiviso non contribuisce alla firma del pruning). 13 test su
  `ordered_sparse`, tutta la suite verde.

Resta da scrivere il **client HTTP** (che vivrà fuori dal crate puro
`semantic-walk`), allineato al formato JSON finale in `docs/PER_TOKEN_ENDPOINTS.md`.