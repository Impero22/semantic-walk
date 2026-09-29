# Piano — Invariante di Stabilità (Opzione 2)

Data: 2026-09-29
Stato: PROPOSTO (in attesa di via libera su vetta-embedder)
Autore: Iris

## Obiettivo

Rendere il verdetto di emissione dei token (emesso/soppresso) robusto alle
perturbazioni numeriche attorno a zero, marcando i token di confine come
"verdetto incerto" invece di farli pendere a destra o a sinistra.

## Contesto

- Il rumore strutturale attorno a zero è ~0.002 (identificato in precedenza).
- Distribuzione pesi sul Dataset B (7005 pesi): il 4.08% dei pesi ha |w| < 0.001.
- Il minimo positivo è 0.000229; il quantile 0.1% dei positivi è 0.000275.
- ε = 0.001 copre la zona di confine senza toccare i token significativi
  (il 5% dei positivi è a 0.025, un ordine di grandezza sopra).

## Design

### 1. vetta-embedder (CrispEmbed) — `sparse_collect_walk`

Estendere lo stato di verdetto da 3 valori a 4:

- `0` = emesso   (w > ε)
- `1` = soppresso (w < -ε)
- `2` = padding   (attn_mask == 0)
- `3` = incerto   (|w| ≤ ε)   ← NUOVO

```cpp
int8_t st;
if (!tokens.attn_mask[t])
    st = 2;
else if (w > EPSILON)
    st = 0;
else if (w < -EPSILON)
    st = 1;
else
    st = 3; // zona di confine: verdetto incerto
```

`EPSILON` versionato (costante nominata, non magica). Valore proposto: 0.001,
da confermare con la distribuzione reale (già misurata).

**Compatibilità retroattiva**: i consumatori esistenti che leggono solo
status 0/1/2 continuano a funzionare — il 3 è un valore nuovo che non
collide con i precedenti. Il fold `/sparse` (max-per-id) continua a
considerare solo status 0.

### 2. semantic-walk — `from_frames` e `positional_jaccard`

Il token incerto (status 3) **non contribuisce alla firma del guardiano**
(come un soppresso: il suo segno è inaffidabile), ma **il peso grezzo resta
salvato** in `weights` per chi lo vuole usare.

Il `from_frames` riceve già il peso; la modifica è: nella costruzione della
firma, il criterio `weight > 0.0` diventa `weight > EPSILON` (stessa ε del
produttore, o una soglia equivalente documentata).

**Attenzione**: `from_frames` non riceve lo status, riceve solo `(token, weight)`.
Quindi la soglia ε deve essere applicata sul peso. Due opzioni:
- (a) il consumer applica la stessa ε sul peso → coerente col produttore
- (b) il produttore passa lo status e il consumer lo rispetta

La (a) è più semplice e non cambia il contratto di `from_frames`; la (b) è più
esplicita ma tocca il contratto con Camillo. **Scelta consigliata: (a)** — la ε
è la stessa, documentata in un punto condiviso, e il consumer non deve
conoscere lo status per comportarsi correttamente.

### 3. adapter.rs — documentazione contratto

Aggiornare la doc del contratto per includere lo status 3 ("incerto"),
spiegando che il consumer deve trattarlo come non-presente per la firma.

## Step di implementazione

1. **vetta-embedder**: aggiungere `EPSILON` versionato + stato 3 in
   `sparse_collect_walk`. Compilare e verificare con test esistente
   (nessuna regressione su status 0/1/2).
2. **semantic-walk**: in `from_frames`, sostituire il criterio `weight > 0.0`
   con `weight > EPSILON` (ε condivisa) nei due punti (multi-token e
   single-token). Stessa cosa in `positional_jaccard`.
3. **Test**: aggiungere test per token incerto (|w| < ε) che non contribuisce
   alla firma ma resta nel buffer.
4. **Validazione**: rilanciare il benchmark B e verificare che i numeri non
   peggiorino (AUC, medie per fascia).
5. **adapter.rs**: aggiornare doc contratto con status 3.
6. **Documentazione**: nota nel tracciamento del progetto.

## Rollback

- Il codice di produzione è copiabile dalla home (come indicato da Federico):
  in caso di errore, si ripristina la codebase e si rilanciano i test.
- Le modifiche sono additive (nuovo stato, nuova costante): nessuna
  riscrittura di percorsi esistenti.

## Rischi

- ε troppo piccolo → non risolve il problema (i token di confine restano
  instabili).
- ε troppo grande → sopprime token che contano (distorsione del segnale).
- Mitigazione: ε = 0.001 copre il rumore (~0.002) con margine, e il 4% dei
  token marcati è una zona di confine onesta senza toccare il segnale
  (quantile 5% dei positivi = 0.025).
