# Contratto: tre canali distinti (ColBERT, Sparse, Relazionale)

**Data:** 13/10/2026
**Autori:** Iris (proponente), Camillo (approvazione)
**Stato:** proposta — da allineare prima dell'implementazione

---

## Contesto e diagnosi

Il benchmark `verify_oracle_real_dataset.rs` produce uno scostamento uniforme
dello 0.0109% su tutte le 200 radici. La diagnosi è duplice:

1. **La sonda corretta (j != i) non è degenere** — 247 valori distinti su 400
   — ma ha varianza piccola (std=0.025, range=0.137). Il grafo resta quasi
   degenere perché i fatti hanno stati densi quasi tutti simili tra loro.
2. **Il terzo canale è un clone artificiale.** Il campo `colbert` nel benchmark
   è `colbert: auto_dense(&seq_a)` — un duplicato esatto di `dense`. Con pesi
   `[0.6, 0.25, 0.15]`, il 75% del peso cade sulla stessa identica misura.

### La scoperta sulla nomenclatura

Verificando `generate_dataset_b.py`, il campo che il benchmark chiama "dense"
**è** l'output ColBERT per-token: `encode_side` fa `dense = fetch_colbert(texto)`
(via `/colbert/encode?tokens=1`, multivector T×1024). Il walk `/ordered-sparse`
produce ids/weights/status (il canale sparse).

Quindi il dataset ha **due canali reali** (ColBERT multivector + sparse walk),
non tre. Il terzo campo è stato riempito con un clone artificiale perché non
c'era altro dato.

---

## La proposta: tre canali genuinamente distinti

### Canale 1 — `colbert` (auto-similarità interna)

- **Dato:** il multivector ColBERT per-token della sequenza (T×1024).
- **Misura:** auto-similarità interna `auto_dense` corretta (j != i) — la
  similarità massima di ogni token verso gli altri token della stessa sequenza,
  mediata.
- **Semantica:** quanto la sequenza è "coesa" internamente.

### Canale 2 — `sparse` (auto-overlap del walk)

- **Dato:** ids/weights/status del walk ordered-sparse.
- **Misura:** `auto_sparse` — overlap globale della firma Bloom del walk con sé
  stessa, normalizzato sui 128 bit.
- **Semantica:** la densità del walk.

### Canale 3 — `relazionale` (similarità A-vs-B, NUOVO)

- **Dato:** le sequenze ColBERT di A e B della coppia.
- **Misura:** similarità incrociata A-vs-B — per ogni token di A, la similarità
  massima verso i token di B (e simmetricamente), mediata.
- **Semantica:** quanto A e B della coppia sono semanticamente vicini.
- **Giustificazione:** `|dense_a − dense_b|` ha 197 valori distinti su 200 —
  porta l'informazione relazionale vera che manca al grafo.

---

## Modifiche al codice

### 1. `semantic-graph/src/costruisci.rs` e `lib.rs`

La struct `Fatto` e `Edge` hanno i campi `dense`, `sparse`, `colbert`.
**Rinomina:** `dense` → `colbert` (il dato è il ColBERT). Il campo `colbert`
esistente viene riutilizzato. **Aggiunta:** il terzo campo diventa `relazionale`.

```rust
pub struct Fatto {
    pub id: NodeId,
    pub colbert: f64,      // era "dense" — auto-similarità interna ColBERT
    pub sparse: f64,
    pub relazionale: f64,  // NUOVO — similarità A-vs-B della coppia
}
```

**Impatto:** `NormalizedAxes` di `semantic-combiner` ha già i campi
`dense/sparse/colbert`. Il canale "dense" di `NormalizedAxes` riceverà la misura
colbert, il canale "colbert" riceverà la misura relazionale. **Il combiner non
cambia** — cambia solo cosa alimenta i suoi tre ingressi.

### 2. `semantic-walk/examples/verify_oracle_real_dataset.rs`

- Il main costruisce i fatti: il campo "dense" riceve `auto_dense(&seq_a)`
  (rinominato semanticamente a colbert), il campo "colbert" riceve la nuova
  misura relazionale A-vs-B.
- Serve una nuova funzione `relazionale_a_b(&seq_a, &seq_b)`.

### 3. `semantic-walk/src/graph_adapter.rs` e `probe_adapter.rs`

Usano `e.dense`, `e.sparse`, `e.colbert` sugli archi. La rinomina si propaga:
`e.dense` → `e.colbert`, `e.colbert` → `e.relazionale`. La semantica delle
medie (`velocity`/`acceleration`/`curvature`) resta invariata — cambiano solo i
nomi dei campi che alimentano le componenti.

### 4. Test ed esempi esistenti

Tutti i `Fatto { dense: ..., sparse: ..., colbert: ... }` nei test e negli
esempi (`costruisci.rs` tests, `lib.rs` tests, `metriche.rs`, `selfloop.rs`,
`stress_knn.rs`, `bench_proximity_adapter_end2end.rs`, `probe_adapter.rs`) vanno
aggiornati alla nuova nomenclatura. I valori sintetici restano — cambia solo il
nome del campo.

---

## Pesi del combiner

Con tre canali ora genuinamente distinti, i pesi `[0.6, 0.25, 0.15]` vanno
ricalibrati. Proposta iniziale: `[0.4, 0.3, 0.3]` (colbert, sparse, relazionale)
— ma è da discutere con Camillo. La calibrazione va fatta sui dati reali, non a
tavolino.

---

## Punti aperti per Camillo

1. **Nome del terzo canale** — `relazionale` è descrittivo, ma forse
   `cross` o `a_b` è più corto. Da decidere.
2. **Pesi** — proposta `[0.4, 0.3, 0.3]`, da validare sui dati.
3. **Misura relazionale** — max-pooling simmetrico A→B e B→A mediato, oppure
   una variante (es. solo A→B)? Da definire con precisione.
4. **`NormalizedAxes`** — il campo "dense" di semantic-combiner riceve la misura
   colbert. È solo nomenclatura interna, ma va dichiarato per evitare il
   malinteso inverso.

---

## Criteri di accettazione

- [ ] Il benchmark `verify_oracle_real_dataset.rs` compila e gira.
- [ ] I tre canali sono alimentati da misure genuinamente diverse (verificabile
      con un check di distribuzione: tre std non collassate).
- [ ] La distribuzione delle sonde mostra varianza su almeno due canali
      (non più il grafo quasi degenere).
- [ ] Tutti i test del workspace passano.
- [ ] La nomenclatura è coerente in tutto il workspace (nessun "dense" che in
      realtà è ColBERT).
