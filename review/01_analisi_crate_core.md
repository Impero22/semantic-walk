# Analisi Crate Core — Pipeline di Base

Analisi dettagliata dei quattro crate che formano la pipeline core: combiner, gate, colbert, walk.

---

## semantic-combiner

### Scopo e API

Combinatore puro-matematico che fonde tre assi di similarità (dense, sparse, colbert) in un punteggio normalizzato e Pareto-coerente in [0,1].

**API pubblica:**
- `NormalizedAxes { dense, sparse, colbert }` — tuple normalizzata
- `normalize(dense, sparse, colbert) -> NormalizedAxes` — saturazione + NaN preservation
- `combine(axes, weights) -> f64` — media pesata
- `pareto_compare(a, b) -> ParetoOrder` — dominanza Pareto
- `ParetoOrder { Dominates, Dominated, Incomparable, Equal }`
- `PESI_CALIBRATI: [f64; 3]` — pesi misurati empiricamente
- `LAMBDA_CALIBRATO: f64` — λ=10.64, iper-parametro di sharpening
- `saturate01(x: f64) -> f64` — clamp [0,1] con NaN pass-through

### Correttezza

La matematica è corretta:
- `saturate01` gestisce propriamente `f64::INFINITY`, `f64::NEG_INFINITY` e `f64::NAN` (preservato, non clampato)
- `pareto_compare` implementa dominanza stretta con fall-through a `Incomparable`
- `combine` è una media pesata con validazione weights (somma = 1.0)
- Il contratto NaN è documentato e coerente: NaN in → NaN out

**Issue S3 — Campi pub su `NormalizedAxes`:**  
I campi `dense`, `sparse`, `colbert` sono `pub` senza newtype o builder. L'invariante [0,1] non è enforcement dal type system. Un chiamante può costruire `NormalizedAxes { dense: 5.0, .. }` bypassando `normalize()`. Questo accade effettivamente in `semantic-graph/costruisci.rs:22-27` (Finding #8).

**Fix suggerito:** rendere i campi `pub(crate)` ed esporre solo getter, oppure aggiungere `debug_assert!(self.dense >= 0.0 && self.dense <= 1.0)` in un metodo `validate()`.

**Issue — Validazione solo debug:**  
`combine()` usa `debug_assert!` per verificare che i pesi sommino a 1.0. In release build, pesi invalidi sono accettati silenziosamente producendo punteggi fuori range.

**Fix suggerito:** aggiungere un check runtime opzionale (feature flag `strict-validation`) oppure documentare il comportamento undefined in release.

### Gestione Errori

Nessun `Result` o `Option` — il crate è totfunzionale. Ogni funzione produce sempre un output (possibilmente NaN). Questa è una scelta di design corretta per un kernel matematico puro.

### Edge Cases

| Input | Comportamento | Corretto? |
|-------|--------------|-----------|
| Tutti assi a 0.0 | `combine` ritorna 0.0 | ✓ |
| Tutti assi a 1.0 | `combine` ritorna 1.0 | ✓ |
| NaN in un asse | `saturate01` preserva NaN, `combine` propaga NaN | ✓ |
| Pesi = [0, 0, 0] | debug_assert fallisce; release: ritorna 0.0 | ⚠ documentare |
| Pesi negativi | debug_assert fallisce; release: comportamento non definito | ⚠ |

**Gap:** nessun test esplicito per NaN in `combine` o `pareto_compare`. La proprietà è verificata indirettamente dai proptest ma manca un unit test mirato.

### Performance

Zero-allocation: tutte le operazioni sono aritmetica su stack (f64 moltiplicazioni e confronti). Nessun heap, nessun Vec, nessun branch prediction penalty significativo. Throughput stimato: ~2ns per `combine()`.

### Test Coverage

- 5 property tests con proptest (1000 cases ciascuno)
- File di regressione committato (`pareto.proptest-regressions`)
- Invarianti testati: Pareto coherence, normalizzazione [0,1], simmetria, idempotenza
- **Buona copertura** — il crate più testato in proporzione alla complessità

### Documentazione

Rustdoc presente su tutte le funzioni pubbliche. Le formule matematiche sono nei doc-comment. Manca un esempio d'uso nel crate-level doc.

### Raccomandazioni Specifiche

1. Rendere `NormalizedAxes` opaco (campi privati + getter) per enforcement dell'invariante
2. Aggiungere unit test per NaN in `pareto_compare` (NaN vs valore, NaN vs NaN)
3. Promuovere `debug_assert` dei pesi a check runtime con `Result` oppure documentare UB in release
4. Aggiungere `#![deny(missing_docs)]` per coerenza

---

## semantic-gate

### Scopo e API

Gate pre-inferenza veloce: decide se il costoso matching ColBERT vale il budget per un dato candidato, usando solo le metriche economiche (dense + sparse).

**API pubblica:**
- `Gate { config }` — istanza del gate
- `GateConfig { soglia_dense, soglia_sparse, budget_ns, pesi_sonda }` — configurazione
- `Verdict { Passa, Blocca, Timeout }` — esito decisione
- `Budget::new(duration_ns) -> Budget` — timer con deadline
- `sonda_economica(dense, sparse, pesi) -> f64` — punteggio economico
- `punteggio_valido(score: f64) -> bool` — check finitezza

### Correttezza

- **Doppio check deadline** corretto: prima della sonda e dopo, per gestire il caso in cui la sonda stessa superi il budget
- **Permissività by design**: NaN score → `Passa`, budget scaduto → `Timeout` (che il chiamante tratta come passaggio), pesi invalidi → `Passa`
- **Nessun panic possibile**: tutte le funzioni sono total, nessun unwrap/expect nel path produttivo
- `sonda_economica` è una media pesata corretta con `is_finite()` guard

**Issue S3 — `budget_ns` non usato:**  
`GateConfig.budget_ns` è memorizzato ma `Gate::decide()` non lo legge — il budget viene passato separatamente come parametro. API confondente: l'utente si aspetta che la config controlli il timeout.

**Fix suggerito:** o usare `self.config.budget_ns` come default quando nessun Budget esterno è fornito, oppure rimuovere il campo dalla config e documentare che il budget è sempre caller-supplied.

**Issue — `Gate::default()` manuale:**  
Implementato come metodo inherent, non come `impl Default for Gate`. Non idiomatico ma funzionalmente corretto. Impedisce l'uso di `Gate` in contesti generici che richiedono `Default`.

### Gestione Errori

Nessun `Result` — il gate è totfunzionale. Ogni percorso produce un `Verdict`. L'assenza di panici è verificabile per ispezione: nessun `unwrap()`, `expect()`, o indexing non guardato.

### Edge Cases

| Input | Comportamento | Corretto? |
|-------|--------------|-----------|
| dense=NaN, sparse=0.5 | `punteggio_valido` fallisce → `Passa` | ✓ permissivo |
| budget=0ns | Timeout immediato | ✓ |
| pesi=[0,0] | somma=0 → `Passa` (documentato) | ✓ |
| Entrambi soglie=1.0 | Nessun candidato passa mai (tranne NaN→Passa) | ✓ |

**Gap:** nessun timing stress test. Il Budget non è mai testato in integrazione con Gate nei test esistenti — solo standalone.

### Performance

Stimato <100ns per decisione (due confronti + una media pesata). Zero-allocation. Il double-deadline-check aggiunge ~5ns (clock_gettime su Linux, QueryPerformanceCounter su Windows).

### Test Coverage

- Unit test inline (funzioni sonda, verdict logic)
- proptest 1000 cases (permissività: nessun input produce panic)
- Integration test (`tests/pipeline.rs`, `tests/coerenza.rs`)
- **Buona copertura** — manca solo il timing integration

### Documentazione

DESIGN.md presente e accurato. Rustdoc su API pubblica. La filosofia permissiva è ben argomentata nei commenti inline.

### Raccomandazioni Specifiche

1. Implementare `Default` trait invece di metodo inherent
2. Risolvere ambiguità `budget_ns` (usarlo o rimuoverlo)
3. Aggiungere integration test con Budget reale e thread::sleep per verificare Timeout
4. Considerare `#[non_exhaustive]` su `Verdict` per evoluzione futura

---

## semantic-colbert

### Scopo e API

Implementazione del reranking ColBERT MaxSim (late-interaction) con scorer trait pluggable.

**API pubblica:**
- `maxsim(query: &[&[f64]], doc: &[&[f64]]) -> f64` — MaxSim nativo
- `ColbertScorer` trait — astrazione per scorer (native, FFI, mock)
- `NativeMaxSim` — implementazione zero-dep del trait
- `ffi::CrispEmbedScorer` — FFI verso libreria esterna (condizionale)

### Correttezza

L'implementazione MaxSim è corretta:
- Per ogni query token, trova il max cosine similarity contro tutti i doc tokens
- Somma i max e normalizza per il numero di query tokens
- Ritorna NaN per input vuoti, dimensione mismatch, o norma zero

**Issue — Performance: norms ricomputate:**  
Le norms dei document tokens sono ricalcolate dentro il loop su query tokens. Per N query tokens e M doc tokens: O(N×M) chiamate sqrt invece di O(M). Con N=128, M=256: ~32.768 sqrt ridondanti.

```rust
// Attuale (semplificato):
for q in query {
    for d in doc {
        let norm_d = d.iter().map(|x| x*x).sum::<f64>().sqrt(); // ← ridondante
    }
}
```

**Fix suggerito:** pre-computare `doc_norms: Vec<f64>` una volta, poi indicizzare nel loop interno.

### Unsafe Code (FFI)

Il modulo `ffi.rs` contiene codice unsafe per l'interfaccia con `crisp-embed`:

- **AtomicUsize con Ordering::Relaxed**: accettabile — pattern write-once (il puntatore è inizializzato una volta e mai modificato)
- **`std::mem::transmute` per function pointer**: pattern standard FFI, sound se la libreria esterna rispetta l'ABI dichiarata
- **`len as i32` cast**: rischio teorico di overflow se len > i32::MAX (~2.1 miliardi di token). Improbabile ma non impossibile con batch molto grandi.

**Fix suggerito:** aggiungere `debug_assert!(len <= i32::MAX as usize)` prima del cast.

### Edge Cases

| Input | Comportamento | Corretto? |
|-------|--------------|-----------|
| query vuoto | Ritorna NaN | ✓ |
| doc vuoto | Ritorna NaN | ✓ |
| Dimensioni mismatch | Ritorna NaN | ✓ |
| Norma zero (vettore nullo) | Salta quel token (NaN → skip nel max) | ⚠ |
| Tutti doc norms = 0 | Ritorna NaN | ✓ |

**Gap:** nessun test per NaN *dentro* un vettore (non vettore vuoto, ma vettore con elementi NaN). Nessun large-input stress test. Nessun test al boundary i32.

### Performance

- Implementazione nativa: O(N×M×D) dove D=dimensione embedding
- Cache-unfriendly: accede a doc tokens in ordine sparso (max per query token)
- Norms ridondanti: vedi sopra
- Per uso reale (N~128, M~256, D~1024): ~33M FLOPs per call — non banale

### Test Coverage

- Unit test per `maxsim` (casi base, simmetria, NaN)
- Mock FFI test senza hardware
- Trait impl test per `NativeMaxSim`
- **Discreta** — mancano stress test e boundary cases

### Documentazione

Rustdoc presente. Il contratto [-1,1] è documentato nel trait ma non nel free function `maxsim`. Questo causa il Finding #3 (quantum assume [0,1]).

### Raccomandazioni Specifiche

1. Pre-computare doc norms (fix performance O(N×M) → O(N×M + M))
2. Documentare il range [-1,1] anche su `maxsim()` free function
3. Aggiungere `debug_assert!` per i32 overflow nel FFI
4. Gate FFI module behind `#[cfg(feature = "ffi")]` (feature flag non-default)
5. Aggiungere test con NaN *within* vectors (non solo empty)
6. Considerare SIMD per il dot product interno (D=1024 è candidato naturale)

---

## semantic-walk

### Scopo e API

Modellazione cinematica di traiettorie + allineamento DTW con banda di Sakoe-Chiba.

**API pubblica:**
- `KinematicState { position, velocity, acceleration }` — stato fisico
- `TrajectoryStep { state, timestamp }` — passo di traiettoria
- `KinematicAligner { window_size }` — allineatore DTW con banda
- `TrajectoryAlignment { distance, path }` — risultato allineamento
- `CrispTrajectory` — traiettoria da embedding crisp
- `align_crisp_trajectories(a, b) -> Result<f64, &'static str>`
- `gate_crisp_alignment(a, b, budget) -> Verdict`
- `inertial_action(state, alpha, beta, gamma) -> f64`

### Correttezza

L'implementazione DTW con Sakoe-Chiba band è corretta nella struttura:
- Cost matrix (n+1)×(m+1) con INFINITY ai bordi
- Banda limitata: `|i-j| <= window_size`
- Backtracking per estrarre il warp path
- `cosine_distance` con guard per norme zero

**BUG S1 — `cosine_distance` con NaN (dtw.rs:41):**

```rust
let sim = (dot / (norm_u_sq.sqrt() * norm_v_sq.sqrt())).clamp(-1.0, 1.0);
Ok((1.0 - sim).max(0.0))
```

Se `u` contiene NaN:
1. `dot` = NaN (NaN × anything = NaN)
2. `norm_u_sq` = NaN
3. Check `norm_u_sq == 0.0` → **false** (NaN ≠ 0.0)
4. `sim` = `NaN.clamp(-1.0, 1.0)` = **NaN** (Rust: clamp su NaN ritorna NaN)
5. `1.0 - NaN` = NaN
6. `NaN.max(0.0)` = **0.0** ← `f64::max` in Rust: "If one argument is NaN, returns the other"

**Risultato:** vettori corrotti appaiono "identici" (distance = 0.0).

**Impatto:** qualsiasi embedding con NaN (da FFI corrotto, memoria uninitialized, o divisione per zero upstream) produce un falso positivo di similarità massima nel DTW. Il bug è silenzioso — nessun log, nessun errore, nessun ritiro.

**Fix suggerito:**
```rust
if sim.is_nan() {
    return Ok(f64::NAN); // oppure return Ok(1.0) per massimo penalty
}
Ok((1.0 - sim).max(0.0))
```

**Issue S2 — Windowed DTW con finestra troppo piccola:**  
Se `window_size < |n - m|`, la cella `cost_matrix[n][m]` resta `INFINITY` perché non raggiungibile entro la banda. Il risultato è distance = INFINITY senza spiegazione.

Nessuna validazione, nessuna documentazione del constraint.

**Fix suggerito:** `debug_assert!(window_size >= n.abs_diff(m))` o meglio un check runtime che ritorna `Err("window_size insufficiente per le lunghezze date")`.

### Gestione Errori

- `Result<f64, &'static str>` per precondizioni (sequenze vuote, dimensione mismatch)
- Errori statici descrittivi in italiano
- Nessun panic nel path produttivo
- **Coerente** con la convenzione del workspace

**Issue:** `inertial_action` non valida che alpha, beta, gamma siano non-negativi. Pesi negativi produrrebbero azione negativa → esponenziale > 1 → amplificazione invece di attenuazione.

### Edge Cases

| Input | Comportamento | Corretto? |
|-------|--------------|-----------|
| seq_a vuota | Err("...non possono essere vuote") | ✓ |
| Dimensione mismatch | Err("...dimensioni non corrispondono") | ✓ |
| window_size = 0 | Solo diagonale i==j allineata | ⚠ non testato |
| window_size > max(n,m) | Equivalente a DTW full | ✓ |
| NaN in vettori | distance = 0.0 (BUG) | ✗ |
| Sequenze identiche | distance = 0.0 | ✓ |
| n=1, m=100, window=1 | INFINITY (non raggiungibile) | ⚠ non documentato |

### Performance

- **Cost matrix:** `Vec<Vec<f64>>` — (N+1) allocazioni separate, cache-unfriendly
- **Norms:** ricalcolate per ogni cella DTW invece di essere pre-computate
- **Warp path:** non pre-allocato (push in Vec vuota)
- **Complessità:** O(n × m × D) con window: O(n × w × D) dove w = window_size

**Fix suggerito:** flat `Vec<f64>` di dimensione `(n+1) * (m+1)` con indexing manuale. Pre-computare norms. Pre-allocare warp path con `Vec::with_capacity(n + m)`.

### Test Coverage

- `dtw_robustezza.rs`: edge cases DTW
- `integrazione_end_to_end.rs`: 7 test end-to-end (pipeline completa)
- proptest per `inertial_action`
- **Buona** per il path felice; **insufficiente** per NaN e boundary window

**Gap:**
- Nessun test con NaN nei vettori
- Nessun test con window_size = 0
- `TrajectoryStep` mai testato direttamente
- Nessun test di performance/regressione

### Documentazione

Doc-comment presenti. La fisica del modello cinematico è spiegata. Manca documentazione del constraint `window_size >= |n-m|`.

### Style

- `pub mod` declarations dopo il blocco `#[cfg(test)]` — cosmetico ma confonde la navigazione
- Naming coerente italiano/inglese (come nel resto del workspace)

### Raccomandazioni Specifiche

1. **P0:** Fix NaN in `cosine_distance` (Finding #2)
2. Validare `window_size >= n.abs_diff(m)` con errore esplicito
3. Pre-computare norms fuori dal loop DTW
4. Sostituire `Vec<Vec<f64>>` con flat allocation
5. Aggiungere test NaN-in-vector
6. Validare alpha/beta/gamma non-negativi in `inertial_action`
7. Pre-allocare warp path

---

[Torna al sommario](./00_sommario_esecutivo.md)
