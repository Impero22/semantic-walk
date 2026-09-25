# Code Review — Workspace semantic-geo
**Data**: 24 settembre 2026  
**Reviewer**: Analisi automatica approfondita  
**Ambito**: Tutti i 7 crate Rust + script Python di zonizzazione  
**Toolchain**: cargo 1.98.1, rustc 1.98.1, clippy

---

# Rapporto di analisi tecnica — workspace `semantic-geo`

**Ambito**: analisi read-only di tutti i crate del workspace Rust `c:\Work\tmp\semantic-geo` (7 crate + root + script Python di zonizzazione), finalizzata a una revisione approfondita del codice.

**Toolchain verificato**: `cargo 1.98.1 (797e8a9bc 2026-08-05)`, `rustc 1.98.1 (48a229cea 2026-09-01)`.

**Esito sintetico**: `cargo clippy --workspace --all-targets` compila senza errori (solo warning, vedi Appendice A); `cargo test --workspace --no-run` compila tutti i binari di test senza errori (Appendice B). Nessun file è stato modificato.

Legenda severità: **Critico** (bug bloccante o unsafe non giustificato) · **Alto** (bug latente, panic in libreria, violazione di contratto) · **Medio** (problema di qualità/performance manutenibile) · **Basso** (igiene, dead code) · **Nit** (stile).

---

## 1. `semantic-combiner`

### 1.1 Scopo e API pubblica
Crate fondamenta del sistema: definisce la normalizzazione dei tre canali (dense/sparse/colbert), il confronto di Pareto e la fusione pesata. API pubblica in `c:\Work\tmp\semantic-geo\semantic-combiner\src\lib.rs`:
- `NormalizedAxes { dense, sparse, colbert }` (tutti `f64` in `[0,1]`) con `normalize(dense, sparse, colbert, sparse_lambda)` (righe 58-72).
- `ParetoOrder` enum (`ADominatesB`, `BDominatesA`, `Incomparable`) e `pareto_compare(&lhs, &rhs)` (righe 92-130).
- `combine(&axes, weights: [f64;3]) -> f64` (riga 138).
- Costanti calibrate `PESI_CALIBRATI = [0.215, 0.552, 0.233]`, `LAMBDA_CALIBRATO = 10.64`, factory `combiner_tricanale()` (righe 168-170).
- Alias `FactId = u64`, `TrivectorScore = f64` (righe 285-289).

### 1.2 Architettura e organizzazione
Modulo singolo, coeso, ben documentato. La filosofia "NaN come assenza, non come estremo" è esplicitata nei doc comment (righe 50-57, 75-81) e implementata coerentemente da `saturate01` (righe 82-90).

**Basso** — `src/lib.rs:285-289`: gli alias `FactId` e `TrivectorScore` sono dichiarati **dopo** il blocco `#[cfg(test)] mod tests_calibrati` (riga 271). Posizionamento inusuale: le dichiarazioni pubbliche dovrebbero precedere i test.

### 1.3 Correttezza
**Alto** — `src/lib.rs:138-147`: la validazione dei pesi in `combine` usa solo `debug_assert!`:
```rust
pub fn combine(axes: &NormalizedAxes, weights: [f64; 3]) -> f64 {
    debug_assert!(weights.iter().all(|w| w.is_finite() && *w >= 0.0), ...);
    let sum: f64 = weights.iter().sum();
    debug_assert!((sum - 1.0).abs() < 1e-9, ...);
```
In build release gli assert sono compilati via: pesi negativi o non normalizzati producono silently un punteggio errato senza alcun segnale. Dato che `combine` è il cuore della fusione trivettoriale consumato da gate/graph/quantum, un contratto violato in release passa inosservato. Raccomandazione: `Result` oppure validazione con fallback a NaN (coerente con la filosofia del ritiro geometrico).

**Alto** — `tests/pareto.proptest-regressions:8-9`: il file di regressioni proptest contiene seed storici con **pesi negativi**:
```
w = [0.6252128716503762, 0.8697240368646478, -0.49493690851502387]
w = [0.9003106035485906, 0.8195751881441748, -0.7198857916927655]
```
Questi seed violano il contratto attuale di `combine` (pesi non-negativi, somma 1). Se proptest li rigenerasse con la strategia corrente, i `debug_assert!` fallirebbero in debug. Evidenza di un bug storico (probabilmente la strategia `arb_weights` inizialmente non normalizzava per somma assoluta). I seed andrebbero rigenerati o il file andrebbe documentato come storico.

**Medio** — `src/lib.rs:58-72`: `normalize` non valida `sparse_lambda`. Con `sparse_lambda <= 0` la trasformazione `1 - exp(-λ·s)` produce valori inattesi (λ=0 → sparse sempre 0; λ<0 → cresce con s ma può superare 1 prima della saturazione). Nessun guard, nessun doc che vincoli λ > 0.

### 1.4 Qualità numerica
Buona. `saturate01` (righe 82-90) gestisce correttamente NaN/±inf con la semantica documentata. La saturazione usa `clamp(0.0, 1.0)` dopo i check di NaN/inf — ordine corretto (clamp su NaN restituirebbe NaN comunque, ma il check esplicito è più chiaro).

**Nit** — `src/lib.rs:110-130`: `pareto_compare` usa confronti `>=`/`>` diretti su `f64` senza epsilon. Per valori normalizzati in `[0,1]` derivanti da `saturate01` il rischio di NaN è basso (NaN viene preservato), ma se un asse è NaN entrambi i rami `lhs_ge_rhs` e `rhs_ge_lhs` sono `false` → `Incomparable`. Comportamento accettabile ma non documentato esplicitamente per il caso NaN.

### 1.5 Performance
`combine` è O(1), `pareto_compare` è O(1). Nessuna allocazione. Ottimo.

### 1.6 Gestione errori
Nessun `Result`: il crate usa la convenzione "NaN come ritiro". Coerente, ma vedi §1.3 sul rischio release di `debug_assert!`.

### 1.7 Copertura test
`tests/pareto.rs` definisce 5 proprietà proptest di qualità:
1. dominanza implica ordine combinato,
2. antisimmetria,
3. output in `[0,1]`,
4. monotonicità stretta,
5. indipendenza da peso nullo.
La strategia `arb_weights()` normalizza tre valori casuali per la loro somma (contratto rispettato). Test unitari in `src/lib.rs:230-283` coprono factory e casi limite.

**Manca**: test proptest che `combine` con pesi non validi (negativi/somma≠1) si comporti in modo definito; test su NaN che attraversa `pareto_compare`; test su `sparse_lambda` degenere.

### 1.8 Stile
Italiano coerente, doc comment filosofici ma precisi. Nessun TODO/FIXME. Nessun `unwrap`/`expect`/`panic!` nel codice di libreria.

### 1.9 Dipendenze
`Cargo.toml` usa `version.workspace = true` (buona pratica). Solo `proptest` come dev-dep. Nessuna dipendenza runtime: crate purissimo.

---

## 2. `semantic-gate`

### 2.1 Scopo e API pubblica
Gate di ammissione economica: decide se un candidato merita il matching completo. API in `c:\Work\tmp\semantic-geo\semantic-gate\src\lib.rs`:
- `GateConfig { pesi_dense, pesi_sparse, soglia, budget_ns }`,
- `Verdict { Passa, Blocca, Timeout }`,
- `Gate::decide(dense, sparse, deadline: Instant) -> Verdict`,
- `sonda_economica(dense, sparse, pesi_dense, pesi_sparse) -> f64` (in `src/sonda.rs`),
- `Budget { deadline, budget_ns }` (in `src/budget.rs`, esportato).

### 2.2 Architettura e organizzazione
Tre moduli ben separati (`lib.rs`, `sonda.rs`, `budget.rs`). La logica di decisione è permissiva: NaN → `Passa`, timeout → `Passa`. Scelta documentata e coerente con la filosofia del "ritiro geometrico" (il gate non blocca ciò che non sa valutare).

### 2.3 Correttezza
**Medio** — `src/budget.rs` (intero file, ~90 righe) + `src/lib.rs` (re-export): la struct `Budget` con `new`, `residuo_ns`, `esaurito` è **API pubblica morta**. `Gate::decide` accetta un `Instant` grezzo e non usa mai `Budget`. O il tipo va integrato nel flusso decisionale, o va rimosso/deprecato. Attualmente è superficie pubblica non testata e non consumata.

**Basso** — `src/lib.rs:99-104`: `Gate::default()` è implementato manualmente come metodo inerente (`pub fn default()`), non come trait `Default`. Clippy segnala `should_implement_trait` (Appendice A, riga 201-213). Un chiamante che si aspetta `Default::default()` resta sorpreso. Raccomandazione: `#[derive(Default)]` su `Gate` oppure `impl Default for Gate`.

**Medio** — `src/sonda.rs` (validazione pesi): `sonda_economica` valida che `pesi_dense + pesi_sparse ≈ 1 ± 1e-9` e restituisce NaN se invalidi. Tuttavia chiama `NormalizedAxes::normalize(dense, sparse, 0.0, LAMBDA_CALIBRATO)` fissando `colbert = 0.0` e poi `combine` con `[pesi_dense, pesi_sparse, 0.0]`. Il peso colbert=0 è corretto per una sonda economica, ma **non è documentato** che la sonda ignora deliberatamente il canale colbert. Un manutentore potrebbe pensare che sia un bug.

### 2.4 Qualità numerica
La validazione dei pesi in `sonda.rs` usa tolleranza `1e-9` sulla somma — appropriata per f64. NaN propagato correttamente. `punteggio_valido(score) = !score.is_nan()` è il discrimine per il verdetto.

### 2.5 Performance
`decide` chiama `Instant::now()` due volte (prima e dopo la sonda) — overhead minimo. La sonda è O(1). Nessun problema.

### 2.6 Gestione errori
Nessun `Result`: il verdetto è un enum. Coerente.

### 2.7 Copertura test
- `tests/coerenza.rs`: 3 proprietà proptest (1000 casi) — sopra soglia mai `Blocca`, soglia zero permissiva, dominanza su entrambe le sonde mai `Blocca`. Buone.
- `tests/pipeline.rs`: integrazione end-to-end gate → maxsim → combiner.

**Manca**: test su `Budget` (API morta non testata); test su pesi invalidi in `sonda_economica` (somma ≠ 1, negativi); test sul percorso `Timeout` con deadline già scaduto.

### 2.8 Stile
Italiano coerente. Doc comment chiari. Nessun TODO/FIXME.

### 2.9 Dipendenze
`Cargo.toml` usa workspace inheritance. Dipende da `semantic-combiner` (corretto). `semantic-colbert` è solo dev-dep (usato in `tests/pipeline.rs`). `proptest = "1.1"` — **incoerente** con gli altri crate che usano `"1"` (vedi §8.1).

---

## 3. `semantic-graph`

### 3.1 Scopo e API pubblica
Costruzione del grafo di prossimità tra fatti e metriche strutturali. API in `c:\Work\tmp\semantic-geo\semantic-graph\src\lib.rs`:
- `NodeId(u64)`, `Edge { from, to, score, dense, sparse, colbert }`,
- `GraphConfig { k, soglia, percentile_cutoff, pesi }`,
- `Graph { config, nodi, archi }` con `ha_arco`, `vicini`, `n_nodi`, `n_archi`,
- `costruisci(fatti: &[Fatto], config) -> Graph` (in `src/costruisci.rs`),
- metriche in `src/metriche.rs`: `grado`, `gradi`, `e_orfano`, `orfani`, `clustering`, `clusterings`, `archi_di`,
- `zonizzazione` in `src/zonizzazione.rs`.

### 3.2 Architettura e organizzazione
Quattro moduli. Separazione ragionevole tra costruzione (`costruisci.rs`), metriche (`metriche.rs`) e zonizzazione (`zonizzazione.rs`).

**Basso** — `src/lib.rs:224`: `pub mod zonizzazione;` è dichiarato **in fondo al file**, dopo il blocco `#[cfg(test)] mod tests` (righe 150-222). Posizionamento inusuale: le dichiarazioni di modulo dovrebbero stare in cima, vicino a `pub mod costruisci;` (riga 32) e `pub mod metriche;` (riga 33).

**Medio** — `src/lib.rs:82-99`: `GraphConfig::default()` usa `pesi: [0.6, 0.25, 0.15]`, **diversi** dai pesi calibrati del combiner `[0.215, 0.552, 0.233]`. Il commento a riga 96 dice "Da ricalibrare quando CrispEmbed fornirà sparse e colbert", ma nel frattempo il grafo usa una fusione non coerente con quella del combiner/gate. Questo è un rischio di **incoerenza di contratto tra crate**: lo stesso fatto può essere ordinato diversamente dal grafo rispetto al gate.

### 3.3 Correttezza
**Critico** — `src/costruisci.rs:132`:
```rust
let altro = fatti.iter().find(|f| f.id == altro_id).unwrap();
```
Doppio problema:
1. `unwrap()` in codice di libreria: se l'invariante "ogni `altro_id` nei candidati esiste in `fatti`" viene violata (es. `fatti` mutato tra la costruzione dei candidati e questa riga, o id duplicati), il programma va in panic. Nessun `Result`, nessun fallback.
2. Complessità: `find` è O(N) ed è eseguito dentro il loop sui candidati (O(N) per nodo) dentro il loop sui nodi (O(N)) → **O(N³)** nel caso peggiore. Per N=10.000 fatti sono 10¹² operazioni. Il lookup dovrebbe essere una `HashMap<NodeId, &Fatto>` costruita una volta (O(N)) per ridurre a O(N²).

**Alto** — `src/costruisci.rs:25-28` (metodo `Fatto::punteggio`):
```rust
NormalizedAxes::normalize(self.dense * altro.dense, self.sparse * altro.sparse, self.colbert * altro.colbert, LAMBDA_CALIBRATO)
```
La similarità di coppia è definita come **prodotto componente-wise delle metriche grezze**. Per dense/colbert in `[-1,1]` il prodotto è una proxy ragionevole (simile a un kernel lineare). Ma per `sparse` (non limitata superiormente, tipicamente `[0, +inf)`) il prodotto **esplode**: due fatti con sparse=10 ciascuno danno sparse_prodotto=100, che normalizzato con `1 - exp(-10.64·100)` satura a 1.0. La saturazione distrugge la discriminazione tra fatti "molto simili" e "similissimi" sul canale sparse. Concettualmente il prodotto non è una similarità: andrebbe usato il minimo, la media, o una similarità sparsa dedicata (es. Jaccard pesato). Il commento nel codice non giustifica questa scelta.

**Medio** — `src/lib.rs:140-145`: `ha_arco` usa `binary_search_by` assumendo che `archi` sia ordinato per `(from, to)`. L'invariante è garantita solo da `costruisci` (riga 145 di `costruisci.rs`: `archi_vec.sort_by(...)`), ma **non è documentata come invariante di `Graph`** né validata. Se un utente costruisce un `Graph` manualmente con `archi` non ordinati, `ha_arco` restituisce risultati errati senza alcun segnale. Raccomandazione: `debug_assert!(self.archi.is_sorted_by(...))` oppure costruzione tramite builder che garantisca l'ordinamento.

**Medio** — `src/zonizzazione.rs` (intero file): usa `f32` per `conf` e per la similarità, mentre **tutto il resto del workspace usa `f64`**. Incoerenza di tipo che può causare perdite di precisione indesiderate e rende difficile il riuso. Vedi §3.4.

### 3.4 Qualità numerica
**Medio** — `src/zonizzazione.rs:similarita_traiettorie`: la funzione alloca **due `HashMap` e un `HashSet` a ogni chiamata**. Nel sweep percentile (`examples/sweep_percentile.rs`) è chiamata O(N²) volte → allocazione quadratica sul percorso caldo. Per N=1000 traiettorie sono ~10⁶ allocazioni di hashmap. Raccomandazione: riusare i buffer tra chiamate (thread-local o passaggio di `&mut`), oppure pre-calcolare i pesi per cella.

**Medio** — `src/zonizzazione.rs:similarita_traiettorie`: ignora completamente il campo `pos` di `TraiettoriaFatto`. La similarità è un Jaccard pesato sulle celle **indipendente dall'ordine**. Questo contraddice il nome "traiettoria" (che implica ordine) e rende il campo `pos` dead data nel calcolo. Documentare che la similarità è order-insensitive, oppure incorporare l'ordine (es. DTW sulle celle, già disponibile in `semantic-walk`).

**Basso** — `src/zonizzazione.rs`: `carica_dizionario`/`carica_traiettorie` restituiscono `Result<_, Box<dyn Error>>`. Errore non tipizzato: il chiamante non può distinguere tra I/O, parse JSON e incoerenza dei dati. Per un crate di infrastruttura andrebbe definito un `ErroreZonizzazione` enum.

### 3.5 Performance
- `costruisci`: O(N²) per il punteggio candidati + O(N³) per il `find().unwrap()` (vedi §3.3). **Il collo di bottiglia dominante è il lookup lineare**, non il punteggio.
- `metriche.rs`: `grado`/`gradi`/`orfani`/`clustering` iterano su tutti gli archi → O(E) per chiamata, O(V·E) per `gradi`/`clusterings` su tutti i nodi. Per grafi densi (E ~ V²) diventa O(V³). Accettabile per analisi offline, problematico se usato online.
- `vicini` (in `metriche.rs`, privato): deduplica con `HashSet` allocato a ogni chiamata → O(E) allocazioni per `gradi` completo.

### 3.6 Gestione errori
`costruisci` non restituisce `Result`: usa `unwrap` (vedi §3.3). `zonizzazione` usa `Box<dyn Error>`. Incoerente.

### 3.7 Copertura test
- `tests/selfloop.rs`: verifica che non vengano creati self-loop. Usa `eprintln!` per debug (**Nit**: i test non dovrebbero stampare su stderr; usare asserzioni o `--nocapture`).
- `tests/test_zonizzazione_reale.rs`: carica i file reali da `../zonizzazione/...` (percorso relativo al crate). Clippy segnala `manual_range_contains` (Appendice A).
- `examples/sweep_percentile.rs`: sweep esteso con `UnionFind` per componenti connesse.

**Manca**: test su `ha_arco` con archi non ordinati (invariante); test su `costruisci` con id duplicati (caso che farebbe scattare l'`unwrap`); test su `Fatto::punteggio` con sparse grandi (saturazione); test proptest sulle metriche (clustering in `[0,1]`, simmetria `grado`).

**Medio** — `examples/sweep_percentile.rs` duplica la funzione `percentile` (privata in `costruisci.rs`). Duplicazione di codice: se la logica del percentile cambia, il sweep diverge silenziosamente. Renderla `pub(crate)` o esporla.

**Basso** — `examples/sweep_percentile.rs` usa il percorso relativo `zonizzazione/...` (cwd = workspace root), mentre `tests/test_zonizzazione_reale.rs` usa `../zonizzazione/...` (cwd = crate dir). **Convenzione di percorso incoerente**: uno dei due fallisce se eseguito dalla directory sbagliata. Raccomandazione: usare `env!("CARGO_MANIFEST_DIR")` per percorsi robusti.

### 3.8 Stile
Italiano coerente. Doc comment estesi. Nessun TODO/FIXME nel codice letto.

### 3.9 Dipendenze
**Alto** — `Cargo.toml`:
```toml
[package]
name = "semantic-graph"
version = "0.1.0"
edition = "2021"
```
Non usa `version.workspace = true` / `edition.workspace = true` (incoerente con combiner/gate/colbert/bridge). Manca `license.workspace = true`.

**Alto** — `Cargo.toml:10-11`:
```toml
semantic-gate = { path = "../semantic-gate" }
semantic-colbert = { path = "../semantic-colbert" }
```
Grep su tutto il crate (src/, tests/, examples/) per `semantic_gate|semantic_colbert` → **0 match**. Entrambe le dipendenze sono **inutilizzate**. Vanno rimosse: aumentano il tempo di compilazione e creano accoppiamento spurio.

`serde = "1.0"` e `serde_json = "1.0"` — pin più stretti di `semantic-walk` che usa `"1"` (vedi §8.1).

---

## 4. `semantic-walk`

### 4.1 Scopo e API pubblica
Allineamento DTW di traiettorie cinematiche + proiezione ordered-sparse. API in `c:\Work\tmp\semantic-geo\semantic-walk\src\lib.rs`:
- `KinematicState { velocity, acceleration, curvature }` (tutti `f32`) con `inertial_action(next, alpha, beta, gamma) = α·Δv² + β·Δa² + γ·|Δκ|`,
- `TrajectoryStep { node_id: FactId, state }` (riga 91),
- moduli: `dtw`, `ingest`, `ordered_sparse`, `adapter`, `parse`.

`dtw.rs`: `TrajectoryAlignment { normalized_score, divergence_token, warp_path }`, `KinematicAligner { window_size }` con `align(seq_a, seq_b)` e `align_with_ordered_sparse(seq_a, seq_b, sparse_a, sparse_b, min_overlap_threshold, w_min, w_max)`.

`ordered_sparse.rs`: `OrderedSparseSequence { global_signature: [u64;2], offsets, tokens, weights }` con `from_frames`, `global_overlap`, `positional_jaccard`, `tokens_at`, `num_positions`.

`adapter.rs`: `ServerResponse<T>`, `ColbertResultJson`, `WalkResultJson`, `WalkFramesResultJson`, `FrameEntry`, `AdapterError`, `colbert_json_to_raw`, `walk_json_to_raw`, `walk_frames_json_to_raw`.

`parse.rs`: `RawColbertTrajectory`, `RawWalk`, `ParseError`, `colbert_to_trajectory`, `walk_to_sequence`, `verifica_coerenza_posizionale`.

`ingest.rs`: `CrispTrajectory`, `align_crisp_trajectories`, `gate_crisp_alignment`.

### 4.2 Architettura e organizzazione
Cinque moduli ben separati per responsabilità. La filosofia "due strati" (guardiano O(1) + raffinatore DTW) è documentata in `ordered_sparse.rs:19-23`.

**Basso** — `src/lib.rs:181-189`: le dichiarazioni `pub mod dtw; pub mod ingest; pub mod ordered_sparse; pub mod adapter; pub mod parse;` sono poste **dopo** il blocco `#[cfg(test)] mod tests` (righe 100-180). Stessa anomalia di stile vista in combiner/graph.

**Basso** — `src/lib.rs:91`: `TrajectoryStep` è definito ma **mai usato** altrove nel crate (grep: 1 solo match, la definizione stessa). Dead code pubblico.

### 4.3 Correttezza
**Critico** — `src/dtw.rs:213`:
```rust
let j = sparse_a.positional_jaccard(sparse_b, i - 1);
```
Il loop esterno è `for i in 1..=n` (riga 205), dove `n = seq_a.len()`. `positional_jaccard(sparse_b, i-1)` accede a `sparse_b.offsets[i-1]` e `sparse_b.offsets[i]`. Ma il check di coerenza a riga 177 valida solo:
```rust
if sparse_a.num_positions() != n || sparse_b.num_positions() != m { return Err(...) }
```
cioè `sparse_a` ha `n` posizioni e `sparse_b` ha `m` posizioni, con `n` e `m` **indipendenti**. Se `n > m` (sequenza A più lunga di B), per `i-1 >= m` l'accesso `sparse_b.offsets[i]` va **out of bounds → panic**. Il caso è raggiungibile: `align_with_ordered_sparse` non valida `n == m`, e le due sequenze dense possono avere lunghezze diverse (è anzi il caso tipico del DTW). La funzione `positional_jaccard` in `ordered_sparse.rs` non fa bounds-check sull'indice (vedi §4.3 sotto).

Evidenza indiretta: gli example `test_dtw_inf.rs` e `test_dtw_panic.rs` esplorano proprio casi con `n != m` (n=10, m=2) usando `align` (non `align_with_ordered_sparse`), quindi non colpiscono il bug. Ma `align_with_ordered_sparse` con `n > m` e overlap sopra soglia → panic certo.

Raccomandazione: validare `n == m` all'ingresso (il Jaccard posizionale ha senso solo a posizioni allineate), oppure usare `min(i-1, m-1)` come clamp, oppure estendere `positional_jaccard` a gestire indici fuori range con fallback.

**Alto** — `src/ordered_sparse.rs:tokens_at(i)`: nessun bounds-check su `i`. Se `i >= num_positions()`, l'accesso a `offsets[i]`/`offsets[i+1]` va in panic. Essendo un metodo pubblico consumato da `dtw.rs`, è parte della superficie del bug Critico sopra.

**Alto** — `src/adapter.rs:277-285` vs `src/parse.rs:186-191`: **mismatch di contratto** tra adapter e parser.
- `walk_frames_json_to_raw` appiattisce i frame assegnando `pos.push(i as u32)` a **tutti** i token del frame `i` (riga 282). Il doc comment (righe 256-258) giustifica: "un futuro head multi-token appiattisce più token per posizione, conservando `pos = i` per ciascuno". Il test a riga 543-545 conferma: `raw.pos == vec![0, 0, 1]` per un frame con 2 token.
- `walk_to_sequence` in `parse.rs:186-191` richiede **stretta crescenza** di `pos`:
```rust
for i in 1..t {
    if raw.pos[i] <= raw.pos[i-1] { return Err(ParseError::WalkDisallineato); }
}
```
Quindi un frame multi-token (prodotta dall'adapter per "futuro SPLADE") viene **rifiutato** dal parser con `WalkDisallineato`. I due moduli hanno contratti incompatibili: l'adapter produce dati che il parser scarta. Oggi il caso non si manifesta perché BGE-M3 emette un token per frame, ma la generalità dichiarata dall'adapter è illusoria. Raccomandazione: o il parser accetta `pos` non-decrescente (crescenza larga), o l'adapter assegna `pos` progressivi per token.

**Medio** — `src/dtw.rs:202`: `let mut cost_matrix = vec![vec![f64::INFINITY; m + 1]; n + 1];` alloca `n+1` `Vec` separate (n+1 allocazioni heap). Per traiettorie lunghe (n=m=1000) sono 1001 allocazioni. Contraddice la filosofia "zero-alloc" dichiarata in `bridge` e il design a guardiano O(1) di `ordered_sparse`. Raccomandazione: singola `Vec<f64>` di dimensione `(n+1)*(m+1)` con indicizzazione manuale `i*(m+1)+j`.

**Medio** — `src/dtw.rs` (banda troppo stretta): se `window_size` è troppo piccolo rispetto a `|n - m|`, la cella `cost_matrix[n][m]` resta `INFINITY` perché mai raggiunta dalla banda di Sakoe-Chiba. Il risultato è `normalized_score = inf` restituito come `Ok(...)` — nessun errore, nessun segnale. L'example `test_dtw_inf.rs` (righe 1-14) **documenta esplicitamente** questo comportamento come noto ("Il revisore ha ragione: il risultato è 'riuscito' ma il costo è INFINITY"). È un bug accettato ma non risolto: andrebbe restituito `Err` oppure `None` quando la cella finale è irraggiungibile.

**Medio** — `src/ordered_sparse.rs:143-148` (e 165-170):
```rust
let h = (token as u128).wrapping_mul(0x9E3779B97F4A7C15).rotate_left(17);
let h = (h ^ (h >> 31)) as u64;
sig[0] |= h;
sig[1] |= h.rotate_left(32);
```
`sig[1]` è derivato da `sig[0]` tramite rotazione di 32 bit di **ogni contributo**. Le due metà sono quindi fortemente correlate: l'entropia effettiva della firma a 128 bit è ~64 bit, non 128. Per un pruning POPCNT questo indebolisce la discriminazione (più falsi positivi di overlap). Raccomandazione: usare due funzioni hash indipendenti (es. due moltiplicatori diversi) per popolare `sig[0]` e `sig[1]`.

**Basso** — `src/parse.rs:colbert_to_trajectory` e `src/adapter.rs:colbert_json_to_raw`: clonano gli embeddings. Per D=1024 e centinaia di token sono clonazioni costose sul percorso di ingest. Considerare zero-copy con lifetime o `Cow`.

### 4.4 Qualità numerica
`cosine_distance` in `dtw.rs` valida NaN/mismatched/empty e restituisce `Err` — buono. `inertial_action` usa `f32`: **Medio**, incoerente con il resto del workspace (f64). La conversione f32→f64 avviene implicitamente nel DTW; per Δv² e Δa² la precisione f32 può essere sufficiente, ma la scelta andrebbe documentata.

**Medio** — `src/dtw.rs:214-222`: interpolazione lineare della banda:
```rust
let t = (j - 0.3) / 0.4;
let w_i = (w_min as f32 + (w_max as f32 - w_min as f32) * t) as usize;
```
Uso di `f32` per interpolare tra `usize`, poi cast a `usize` (troncamento). Per `w_min=2, w_max=10, j=0.5` → `t=0.5`, `w_i = 2 + 8*0.5 = 6.0` → 6. OK, ma il troncamento (non arrotondamento) introduce un bias sistematico verso bande più strette. Raccomandazione: `.round() as usize`.

### 4.5 Performance
- `align`: O(n·m·D) per il costo locale (dot product su D dimensioni) entro la banda di Sakoe-Chiba → O(n·m·D) nel caso peggiore (banda larga), O(n·w·D) con banda stretta. La matrice di costo è O(n·m) in memoria.
- `align_with_ordered_sparse`: strato 1 O(1) (POPCNT su 2 u64), strato 2 come sopra con banda dinamica. Ottima idea, ma il bug Critico §4.3 ne compromette l'usabilità.
- `positional_jaccard`: merge a due puntatori su frame ordinati → O(|frame_i| + |frame_j|) per chiamata, chiamato n volte → O(n · tok_per_frame).

### 4.6 Gestione errori
`dtw.rs` usa `Result<_, &'static str>` — **Medio**: stringhe statiche invece di un enum tipizzato. `parse.rs` usa `ParseError` enum (buono). `adapter.rs` usa `AdapterError` enum (buono). Incoerenza tra moduli dello stesso crate.

### 4.7 Copertura test
- `tests/dtw_robustezza.rs`: 8 test inclusi NaN handling (regressione documentata come "Finding #2" di una review precedente).
- `tests/integrazione_end_to_end.rs`: end-to-end con D=1024, gate permissivo/restrittivo, DTW distingue simili da divergenti.
- `tests/scheletro_3_livelli.rs`: tre livelli (parse JSON, filtro igiene, fixture reale "il_gatto_dorme" con 7 token inclusi `<s>`/`</s>`). Clippy segnala `identity_op` (Appendice A).
- Test unitari in `ordered_sparse.rs`, `dtw.rs`, `parse.rs`, `adapter.rs`.

**Manca**: test su `align_with_ordered_sparse` con `n != m` (il caso Critico §4.3); test su `positional_jaccard` con indice fuori range; test proptest su `cosine_distance` (simmetria, [0,2] bounds); test su frame multi-token end-to-end (adapter→parse) che oggi fallirebbe.

### 4.8 Stile
Italiano coerente. Doc comment estesi e filosofici. Nessun TODO/FIXME.

### 4.9 Dipendenze
**Alto** — `Cargo.toml`: hardcode `version = "0.1.0"` e `edition = "2021"` (no workspace inheritance), manca `license`. `serde = { version = "1", ... }` — pin più lasco di graph (`"1.0"`). Incoerenza workspace.

---

## 5. `semantic-bridge`

### 5.1 Scopo e API pubblica
Ponte tra celle discrete (zonizzazione) e punti geometrici continui. API in `c:\Work\tmp\semantic-geo\semantic-bridge\src\lib.rs`:
- `WalkStep`, `Walk<'a> { celle, pos, conf, colbert, dim }` con `coerente()`,
- `Dizionario<'a> { centroidi, k, dim }` con `coerente()`,
- trait `TrasformazionePonte::proietta(walk, dizionario, out_points) -> Option<&[f64]>`,
- trait `OrchestratorePonte`,
- `ErrorePonte` enum,
- `ProiezionePunti { Centroide, MediaPesata }` (in `src/proiezione.rs`).

### 5.2 Architettura e organizzazione
Due moduli, design zero-alloc con buffer pre-allocato dal chiamante (`out_points: &'a mut [f64]`). Ottima separazione tra contratto (`lib.rs`) e implementazione (`proiezione.rs`).

### 5.3 Correttezza
**Medio** — `src/lib.rs:61` + `src/lib.rs:85-92` vs `src/proiezione.rs`: il campo `Walk.colbert` è **richiesto** da `coerente()` (riga 91: `self.colbert.len() == n * self.dim`) ma **mai letto** da `proiezione.rs` (grep `walk\.colbert|\.colbert` in proiezione.rs → 0 match). Le strategie `Centroide` e `MediaPesata` usano solo `celle`, `pos`, `conf`, `dim`. Il chiamante è obbligato a fornire una matrice ColBERT (costosa: n·dim f32, per n=100 e dim=1024 sono 100KB) che il ponte ignora. O il campo va usato (es. proiezione ibrida centroide+colbert), o va reso opzionale/rimosso dal contratto di coerenza.

**Basso** — `src/proiezione.rs:30-34`: `impl Default for ProiezionePunti` manuale. Clippy segnala `derivable_impls` (Appendice A): basta `#[derive(Default)]` + `#[default]` sulla variante `Centroide`.

### 5.4 Qualità numerica
`MediaPesata` usa finestra scorrevole a 3 elementi (2 ai bordi), pesata per `conf`, normalizzata. Controllo esplicito `walk.dim != dizionario.dim → None` (regressione documentata come "Finding #1" di una review precedente, con test dedicato per `dim = 1024`). Buona robustezza.

**Basso** — `src/proiezione.rs:281`: `centroidi[1 * d + j]` — clippy `identity_op` (Appendice A): `1 * d` è ridondante. Nit di test code.

**Basso** — `src/proiezione.rs:310`: `for j in 0..d { punti[j] ... }` — clippy `needless_range_loop` (Appendice A). Nit di test code.

### 5.5 Performance
Zero-alloc per design (buffer del chiamante). `Centroide` è O(n·dim) (copia diretta). `MediaPesata` è O(n·dim) con finestra 3. Ottimo.

### 5.6 Gestione errori
`Option<&[f64]>` per il ritiro (None = buffer disallineato o dati incoerenti). `ErrorePonte` enum distingue buffer issues da data inconsistency. Coerente.

### 5.7 Copertura test
Test unitari in `proiezione.rs` coprono: coerenza walk/dizionario, dim mismatch (regressione), Centroide vs MediaPesata, bordi. Manca proptest su proprietà (es. output sempre finito per input coerenti, MediaPesata riduce a Centroide quando conf è uniforme).

### 5.8 Stile
Italiano coerente. Doc comment chiari. Nessun TODO/FIXME.

### 5.9 Dipendenze
`Cargo.toml` usa workspace inheritance (buono). Dipende da `semantic-graph` (per `Dizionario`/zonizzazione). `proptest` dev-dep.

---

## 6. `semantic-colbert`

### 6.1 Scopo e API pubblica
MaxSim late-interaction ColBERT + FFI verso motore C++ CrispEmbed. API in `c:\Work\tmp\semantic-geo\semantic-colbert\src\lib.rs`:
- free function `maxsim(query_tokens: &[&[f64]], doc_tokens: &[&[f64]]) -> f64`,
- trait `ColbertScorer { compute_maxsim(...) -> f64 }`,
- `NativeMaxSim` (unit struct, delega a `maxsim`),
- in `src/ffi.rs`: `CrispEmbedColbertScoreFn`, `CrispEmbedScorer`, `CRISPEMBED_COLBERT_SCORE_SYMBOL`.

### 6.2 Architettura e organizzazione
Due moduli: logica pura (`lib.rs`) e FFI (`ffi.rs`). Separazione netta. Il trait `ColbertScorer` astrae la sorgente del punteggio (nativo vs C++) — buon design per l'iniezione.

### 6.3 Correttezza
**Alto** — `src/lib.rs:83-98` (loop interno di `maxsim`):
```rust
for q in query_tokens {
    let norm_q = norm_l2(q);
    ...
    for d in doc_tokens {
        let norm_d = norm_l2(d);   // ← ricalcolata per ogni coppia (q,d)
        ...
        let dot = dot_product(q, d);
        let cos_sim = (dot / (norm_q * norm_d)).clamp(-1.0, 1.0);
```
`norm_l2(d)` è ricalcolata **per ogni coppia (q,d)**. Complessità attuale: O(N·M·D) per i dot product + O(N·M·D) per le norme di `d` (ricalcolate N volte). Pre-calcolando le norme di `doc_tokens` una volta (O(M·D)) si risparmia un fattore N sulle norme. Per N=M=128 token e D=1024: ~16.7M operazioni di norma ridondanti. **Percorso caldo** (chiamato da gate, graph, quantum, walk). Raccomandazione: pre-computare `Vec<f64>` delle norme di doc_tokens prima del loop esterno.

**Medio** — `src/ffi.rs:156-164`:
```rust
let score = unsafe {
    func(query_flat.as_ptr(), query_tokens.len() as i32,
         doc_flat.as_ptr(), doc_tokens.len() as i32, dim as i32)
};
```
Cast `usize as i32` **senza check di overflow**. Il commento (righe 153-155) afferma "i valori realistici stanno ampiamente dentro i32", ma non c'è alcun `debug_assert!` o `try_from().unwrap_or(...)`. Con matrici patologiche (token > 2³¹) il cast troncato passerebbe dimensioni negative al C++ → undefined behavior. Per codice unsafe FFI la difesa dovrebbe essere esplicita.

**Medio** — `src/ffi.rs:102`: `unsafe { std::mem::transmute(ptr) }` senza annotazioni di tipo. Clippy segnala `missing_transmute_annotations` (Appendice A). Raccomandazione: `transmute::<*mut c_void, CrispEmbedColbertScoreFn>(ptr)` per rendere il contratto esplicito e aiutare il compilatore a rilevare mismatch ABI.

**Medio** — `src/ffi.rs` (uso di `AtomicPtr`): `CrispEmbedScorer` tiene il puntatore alla funzione in un `AtomicPtr<c_void>`. Il puntatore è impostato una volta alla costruzione e **mai mutato**. Gli atomics sono overhead ingiustificato: un `*const c_void` semplice (o meglio, un `Option<CrispEmbedColbertScoreFn>` tipizzato) basterebbe. L'uso di `AtomicPtr` suggerisce un design per mutazione futura che non c'è.

**Basso** — `src/ffi.rs:144-151`: conversione f64→f32 dei buffer. Perdita di precisione documentata e accettata (il C++ lavora in f32). OK, ma andrebbe segnalato che per embeddings normalizzati in [-1,1] la precisione f32 (~7 cifre) è abbondantemente sufficiente.

### 6.4 Qualità numerica
`maxsim` valida input (empty, mismatched dim, NaN) e restituisce NaN — coerente con la filosofia del ritiro. `clamp(-1.0, 1.0)` su cos_sim protegge da errori di arrotondamento. Divisione per `norm_q * norm_d` preceduta da check `== 0.0 || is_nan()`. Buono.

**Nit** — `src/lib.rs:107`: `somma_massimi / (query_tokens.len() as f64)` — se `query_tokens` è vuoto il check a riga 55-58 ha già restituito NaN, quindi la divisione per zero non è raggiungibile. Ma la difesa è implicita: un `debug_assert!(!query_tokens.is_empty())` renderebbe l'invariante esplicita.

### 6.5 Performance
Vedi §6.3 (Alto). Oltre alla norma ridondante, `dot_product` usa `a.iter().zip(b).map(|(x,y)| x*y).sum()` — il compilatore dovrebbe vettorizzare, ma per D=1024 vale la pena verificare con un benchmark che l'auto-vectorization avvenga (nessun `#[inline]` esplicito su `dot_product`, anche se c'è — riga 139).

### 6.6 Gestione errori
NaN come ritiro (free function). FFI: fallback a NaN se libreria non caricata. Nessun `Result`. Coerente, ma per la FFI un `Result` sarebbe più informato (distinguere "libreria assente" da "input invalido").

### 6.7 Copertura test
Test unitari in `lib.rs:150-240` e `ffi.rs:230-300` coprono: identità, ortogonalità, dimensioni mismatch, NaN, empty. Clippy segnala numerosi `useless_vec!` nei test (Appendice A, ~24 warning) — nit di stile.

**Manca**: benchmark che dimostri il costo della norma ridondante (§6.3); test su cast i32 overflow (§6.3); test proptest su simmetria di `maxsim` (maxsim(q,d) == maxsim(d,q) quando N==M).

### 6.8 Stile
Italiano coerente. Doc comment chiari. Nessun TODO/FIXME.

### 6.9 Dipendenze
`Cargo.toml` usa workspace inheritance. Nessuna dipendenza runtime (FFI usa solo `std`). Ottimo.

---

## 7. `semantic-quantum`

### 7.1 Scopo e API pubblica
Risoluzione quantistico-ispirata: collasso di rami multipli via integrale di cammino di Feynman, con pruning Pareto candidate-level post-collasso. API in `c:\Work\tmp\semantic-geo\semantic-quantum\src\lib.rs`:
- `BranchType` enum,
- `WalkBranch { branch_type, candidate_id: Option<FactId>, action: f32, amplitude: f32, s_inertial, s_geometric, s_colbert }`,
- `BranchBuilder { weights: (f32,f32,f32), colbert_scorer: Arc<dyn ColbertScorer + Send + Sync> }` con `build_branch`, `build_branch_from_tokens`, `compute_colbert_cost`,
- `QuantumResolver { divergence_threshold, horizon, kappa_break }` con `collapse(branches) -> Option<WalkBranch>`, `evaluate_colbert_branch`,
- in `src/pareto.rs`: `BranchCostVector`, `CandidateCostVector`, `estrai_frontiera_pareto`, `estrai_frontiera_pareto_adattivo`, `estrai_frontiera_pareto_sui_candidati`, `varianza_totale_normalizzata`, costanti `SOGLIA_ADATTIVA_DEFAULT = 256`, `VARIANZA_DIRECT_DEFAULT = 0.01`.

### 7.2 Architettura e organizzazione
Due moduli principali (`lib.rs`, `pareto.rs`) + 4 binari (`bench_pareto.rs`, `probe3.rs`, `probe_dominance.rs`, `probe_dominance2.rs`) + 1 example (`test_pareto_winner.rs`). Il design a 4 passi del collapse (filtro decoerenza, accumulo HashMap, argmax, estrazione rappresentante min-action) è documentato in `lib.rs`.

Il commento in `pareto.rs:180-200` spiega chiaramente **perché il pruning branch-level pre-collapse è VIETATO** (mutila la funzione d'onda Ψ(c) = Σ exp(-S_r/κ)) e perché il Pareto è legittimo solo post-collapse sui candidati aggregati. Documentazione di qualità.

### 7.3 Correttezza
**Alto** — `src/lib.rs:283-289`:
```rust
pub fn evaluate_colbert_branch(&self, query_tokens: &[&[f64]], doc_tokens: &[&[f64]]) -> f64 {
    maxsim(query_tokens, doc_tokens)
}
```
Chiama la **free function** `maxsim` di `semantic-colbert` direttamente, **bypassando** il `colbert_scorer` iniettato (`Arc<dyn ColbertScorer>`) che invece è usato da `BranchBuilder::compute_colbert_cost` (riga 192: `self.colbert_scorer.compute_maxsim(...)`). Incoerenza di contratto: due percorsi per lo stesso calcolo, uno iniettabile (testabile con scorer custom, es. `AlwaysNan` nei test a riga 630) e uno hardcoded. Se un utente inietta `CrispEmbedScorer` per usare la FFI C++, `evaluate_colbert_branch` ignora l'iniezione e usa il proxy nativo. Raccomandazione: `self.colbert_scorer.compute_maxsim(...)` anche qui, oppure rimuovere il metodo se ridondante.

**Medio** — `src/lib.rs:272` + `src/lib.rs:278-279`: il campo `horizon: usize` di `QuantumResolver` è dichiarato e inizializzato ma **mai letto** (grep: 3 match, tutti nella definizione/costruzione). Dead field. O va usato (es. per limitare la profondità dei rami) o va rimosso dalla struct pubblica.

**Medio** — `src/pareto.rs:170-177`:
```rust
if branches.len() <= soglia {
    estrai_frontiera_pareto(branches)
} else if varianza_totale_normalizzata(branches) >= varianza_direct {
    estrai_frontiera_pareto(branches)
} else {
    branches.to_vec()
}
```
I primi due rami sono **identici**. Clippy segnala `if_same_then_else` (Appendice A). La logica è corretta (entrambi i casi fanno FullPareto), ma la struttura a if-else è ridondante e confonde il lettore: sembra che ci sia una distinzione dove non c'è. Raccomandazione: `if branches.len() <= soglia || varianza >= varianza_direct { estrai_frontiera_pareto(branches) } else { branches.to_vec() }`.

**Medio** — `src/lib.rs:207-216`: `build_branch_from_tokens` ha **8 argomenti** (clippy `too_many_arguments`, Appendice A). Funzione con troppi parametri posizionali: facile scambiare l'ordine. Raccomandazione: struct di config (`BranchParams { branch_type, candidate_id, ... }`) o builder pattern.

**Basso** — `src/lib.rs:191-196`: `compute_colbert_cost` mappa NaN → `1.0` (penalità massima). Scelta documentata ("meglio un costo ignoto trattato come massimo che un costo falso"), coerente con la filosofia. Ma è l'**opposto** del ritiro geometrico del combiner (NaN → propagazione). Due filosofie di gestione dell'ignoto nello stesso sistema: il combiner ritira, il quantum penalizza. Va documentato a livello di workspace perché è una scelta architetturale, non un bug.

**Basso** — `src/bin/probe_dominance.rs:34-40`: `aggrega` usa `(b.action, b.amplitude, b.action*b.amplitude)` per i campi `(s_inertial, s_geometric, s_colbert)` di `CandidateCostVector`. Semantica confusa: `action*b.amplitude` come proxy di costo colbert non ha significato geometrico. Essendo un binario di probe sperimentale è accettabile, ma va etichettato come tale (commento "PROBE: proxy non geometrici").

### 7.4 Qualità numerica
`collapse` (righe 291+): 4 passi con guard NaN esplicite. `amplitude = exp(-action / kappa_break)`: se `action` è grande e `kappa_break` piccolo, `exp(-x)` underflow a 0.0 — gestito (il ramo contribuisce 0 all'accumulo). Se `action` è NaN, il filtro decoerenza lo scarta prima. Buono.

`varianza_totale_normalizzata`: calcola varianza dei costi normalizzati. Usata come discriminante per il gate adattivo FullPareto/DirectCollapse. Soglia default 0.01.

**Nit** — `src/pareto.rs:estrai_frontiera_pareto`: O(N²) con clonazioni dei `BranchCostVector` per ogni confronto. Per N=1024 rami sono ~10⁶ confronti con clonazioni. Il bench (`src/bin/bench_pareto.rs`) misura proprio questo overhead. Accettabile per N piccoli, ma il gate adattivo (riga 165) esiste apposta per evitare il costo su N grandi — peccato che i due rami identici (§7.3) rendano il gate ineffective quando `varianza >= varianza_direct` con N grande (fa comunque FullPareto O(N²)).

### 7.5 Performance
- `collapse`: O(B) con HashMap (B = numero rami). Ottimo.
- `estrai_frontiera_pareto`: O(B²) con clonazioni.
- `estrai_frontiera_pareto_sui_candidati`: O(C²) con C = candidati (tipicamente C << B).
- `bench_pareto.rs` misura collapse vs collapse+Pareto per N ∈ {16,64,128,256,1024} e dominanza ∈ {0.0,0.5,0.9}. Generatore deterministico LCG (righe 41-46) — buona pratica per benchmark riproducibili.

### 7.6 Gestione errori
`collapse` restituisce `Option<WalkBranch>` (None = tutti i rami decoerenti). Nessun `Result`. Coerente.

### 7.7 Copertura test
Test unitari in `lib.rs` (righe 370+) coprono: collapse con scorer NaN (`AlwaysNan`), build_branch, evaluate_colbert_branch. `example/test_pareto_winner.rs` è un **controesempio documentato** che dimostra come il pruning branch-level pre-collapse possa **invertire il vincitore** (A ha 1 ramo eccellente che domina tutti i rami di B; B ha 10 rami buoni la cui somma batte A; il pruning elimina B e fa vincere A). Questo example è la giustificazione empirica del divieto di pruning pre-collapse — eccellente documentazione eseguibile.

**Manca**: proptest su `collapse` (es. il vincitore ha sempre action minima tra i rappresentanti; amplitude sempre ≥ 0); test su `estrai_frontiera_pareto_adattivo` con varianza al contorno della soglia; test su `horizon` (dead field, §7.3).

### 7.8 Stile
Italiano coerente. Doc comment estesi con formule LaTeX-like (Ψ, κ, Σ). Clippy segnala `doc_overindented_list_items` in `bench_pareto.rs:15` (Appendice A) — nit. `useless_vec!` in test di `lib.rs:376-377`.

### 7.9 Dipendenze
**Alto** — `Cargo.toml`: hardcode `version = "0.1.0"` e `edition = "2021"` (no workspace inheritance), manca `license`. Dipende da `semantic-combiner`, `semantic-colbert`, `semantic-walk` — tutte usate. `proptest = "1"` dev-dep.

---

## 8. Root `Cargo.toml` + `zonizzazione/zonizza.py`

### 8.1 Root `Cargo.toml`
```toml
[workspace]
resolver = "2"
members = ["semantic-bridge", "semantic-colbert", "semantic-combiner",
           "semantic-gate", "semantic-graph", "semantic-quantum", "semantic-walk"]
[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
[workspace.dependencies]
```

**Alto** — Incoerenza di workspace inheritance. Tre crate su sette (`semantic-graph`, `semantic-walk`, `semantic-quantum`) **hardcodano** `version = "0.1.0"` e `edition = "2021"` invece di usare `version.workspace = true` / `edition.workspace = true`. Gli stessi tre **omettono** `license.workspace = true`. Gli altri quattro (bridge, colbert, combiner, gate) usano correttamente l'inheritance. Raccomandazione: uniformare tutti i crate a `version.workspace = true`, `edition.workspace = true`, `license.workspace = true`.

**Alto** — `[workspace.dependencies]` è **vuoto**. Ogni crate pina le proprie dipendenze individualmente, causando incoerenze di versione:
- `proptest`: `"1"` (combiner, bridge, quantum, walk) vs `"1.1"` (gate),
- `serde`: `"1.0"` (graph) vs `"1"` (walk),
- `serde_json`: `"1.0"` (graph, walk dev) — coerente.

Raccomandazione: spostare le dipendenze condivise in `[workspace.dependencies]` e usare `{ workspace = true }` nei crate. Questo garantisce una singola fonte di verità per le versioni.

**Medio** — Accoppiamento inter-crate non dichiarato a livello di workspace. Il grafo delle dipendenze è:
```
combiner ← gate ← walk ← quantum
   ↑         ↑      ↑
 colbert   graph  bridge ← graph
```
`semantic-graph` dipende da `gate` e `colbert` ma **non li usa** (§3.9). `semantic-walk` dipende da `gate` (usato in `ingest.rs`). `semantic-quantum` dipende da `combiner`, `colbert`, `walk`. `semantic-bridge` dipende da `graph`. Non c'è un crate "facade" che riesporta l'API pubblica del sistema: ogni consumatore deve conoscere il grafo delle dipendenze. Considerare un crate `semantic` umbrella.

### 8.2 `zonizzazione/zonizza.py`
Script Python (122 righe) che estrae le righe ColBERT da Qdrant, le clusterizza con `MiniBatchKMeans` (K=256), e produce due artefatti JSON: `dizionario_celle_v1.0.json` (centroidi normalizzati) e `traiettorie_v1.0.json` (celle/pos/conf per fatto). È l'origine dei dati consumati da `semantic-graph/src/zonizzazione.rs` e `semantic-bridge`.

**Medio** — `zonizza.py:67`:
```python
km = MiniBatchKMeans(n_clusters=K, batch_size=1024, n_init=5, random_state=0)
```
`random_state=0` garantisce riproducibilità. Buono. Ma `n_init=5` con MiniBatchKMeans: ogni run può dare centroidi leggermente diversi se il dataset cambia. Il dizionario è "congelato" e versionato (`versione: '1.0'`, riga 80), quindi la riproducibilità è gestita a livello di artefatto, non di algoritmo. OK, ma va documentato che il dizionario va rigenerato solo con pipeline versionata.

**Medio** — `zonizza.py:60-62`:
```python
norms = np.linalg.norm(M, axis=1, keepdims=True)
norms[norms == 0] = 1
Mn = M / norms
```
Protezione da divisione per zero (norma 0 → 1). Ma un vettore a norma 0 resta **vettore nullo** dopo la divisione (0/1 = 0), e viene assegnato a una cella con `conf = 0` (similarità coseno con qualsiasi centroide normalizzato = 0). Le traiettorie con passi a conf=0 inquinano il Jaccard pesato di `zonizzazione.rs` (peso 0 = passo invisibile). Non è un bug, ma è un edge case da documentare: fatti con embeddings degeneri producono traiettorie "fantasma".

**Basso** — `zonizza.py:18`: `API_KEY = os.environ.get('QDRANT_API_KEY', '')` — chiave da env, mai hardcodata. Buona pratica di sicurezza.

**Basso** — `zonizza.py:17`: `QDRANT = 'http://localhost:6332'` — hardcoded. Considerare env var anche per l'endpoint.

**Basso** — `zonizza.py:49`: `pos[i] if i < len(pos) else i` — fallback silenzioso se l'array `pos` del walk è più corto delle righe ColBERT. Questo caso (disallineamento tra walk e colbert sul server) produce posizioni sintetiche `i` senza alcun warning. Raccomandazione: loggare un warning quando `len(pos) != len(colbert)`.

**Nit** — `zonizza.py:97`: `from collections import defaultdict` importato **dentro** la funzione `main()`, non in cima al file. Stile non idiomatico.

**Nit** — `zonizza.py:82`: `'dim': 1024` hardcoded. Se il modello ColBERT cambia dimensione, il dizionario dichiara 1024 ma contiene centroidi di dimensione diversa. Derivare `dim` da `M.shape[1]`.

### 8.3 Coerenza tra artefatti Python e consumo Rust
Il formato JSON prodotto da `zonizza.py` (`dizionario_celle_v1.0.json` con campi `versione`, `K`, `dim`, `metric`, `centroidi`; `traiettorie_v1.0.json` con `celle`/`pos`/`conf` per fatto) è consumato da `semantic-graph/src/zonizzazione.rs` (`carica_dizionario`, `carica_traiettorie`). **Non c'è nessuno schema condiviso** (es. serde con `#[serde(rename)]` documentato, o un JSON Schema) tra il produttore Python e il consumatore Rust. Se `zonizza.py` rinomina un campo, il Rust fallisce a runtime con errore di parse non tipizzato (`Box<dyn Error>`, §3.4). Raccomandazione: definire il contratto in un unico luogo (es. un file `SCHEMA.md` o un test di round-trip con fixture generata da Python).

---

## 9. Sezione trasversale (workspace-level)

### 9.1 Incoerenze di contratto tra crate

**Alto** — Pesi di fusione divergenti. `semantic-combiner` definisce `PESI_CALIBRATI = [0.215, 0.552, 0.233]` (dense/sparse/colbert) come "i" pesi calibrati. `semantic-graph::GraphConfig::default()` usa `[0.6, 0.25, 0.15]` (lib.rs:98) con commento "Da ricalibrare quando CrispEmbed fornirà sparse e colbert". `semantic-quantum::BranchBuilder::new` usa `(0.4, 0.3, 0.3)` nei bench/probe (es. `bench_pareto.rs:48`). **Tre set di pesi diversi** per lo stesso sistema di fusione trivettoriale. Il gate usa `pesi_dense`/`pesi_sparse` configurabili (default non verificato ma derivato da `GateConfig::default()`). Non esiste una singola fonte di verità per i pesi. Raccomandazione: esporre i pesi calibrati dal combiner e farli consumare da graph/quantum/gate, oppure documentare esplicitamente che ogni crate usa pesi diversi per scopi diversi (ma allora il commento "da ricalibrare" di graph è fuorviante).

**Alto** — Filosofia di gestione dell'ignoto incoerente. `semantic-combiner` propaga NaN come "assenza di valore" (ritiro geometrico, `saturate01` preserva NaN). `semantic-quantum::compute_colbert_cost` mappa NaN → `1.0` (penalità massima, "meglio ignoto come massimo che falso"). Due filosofie opposte per lo stesso concetto. Non è un bug, ma è una **scelta architetturale non documentata a livello di workspace**: un manutentore che assume la coerenza del combiner resterà sorpreso dal quantum. Raccomandazione: un `ARCHITECTURE.md` che spieghi quando si ritira (combiner, gate, colbert) e quando si penalizza (quantum), e perché.

**Medio** — `semantic-walk` adapter vs parse (§4.3): contratto incompatibile su `pos` (multi-token frame). L'adapter dichiara generalità futura che il parser rifiuta oggi.

**Medio** — `semantic-quantum::evaluate_colbert_branch` bypassa lo scorer iniettato (§7.3), rompendo il contratto di iniezione di `BranchBuilder`.

### 9.2 Incoerenze di tipo numerico

**Medio** — `f32` vs `f64`. La maggior parte del workspace usa `f64` per metriche e punteggi. Eccezioni:
- `semantic-walk::KinematicState` (velocity/acceleration/curvature): `f32`,
- `semantic-walk::ordered_sparse::weights`: `f32`,
- `semantic-walk::WalkBranch` (action/amplitude/s_*): `f32`,
- `semantic-graph::zonizzazione` (conf, similarità): `f32`,
- `semantic-bridge::Walk.conf` / `Walk.colbert`: `f32`,
- `semantic-colbert::ffi` (buffer FFI): `f32` (imposto dal C++).

La scelta f32 è giustificata per la FFI (contratto C-ABI) e per il bridge (allineato alla zonizzazione). Ma `WalkBranch` in quantum usa f32 per action/amplitude mentre `combine` del combiner usa f64: i punteggi passano da f64 (combiner) a f32 (quantum) con perdita di precisione non documentata. Raccomandazione: documentare la scelta f32 nei crate che la usano, oppure uniformare a f64 dove il costo è trascurabile.

### 9.3 Incoerenze di gestione errori

**Medio** — Quattro stili di error handling nello stesso workspace:
1. `Result<_, EnumTipizzato>` (parse.rs `ParseError`, adapter.rs `AdapterError`, bridge `ErrorePonte`),
2. `Result<_, &'static str>` (dtw.rs),
3. `Result<_, Box<dyn Error>>` (zonizzazione.rs),
4. `Option<T>` / NaN-as-error (combiner, gate, colbert, quantum).

Nessuno stile dominante. Raccomandazione: definire una convenzione di workspace (es. `thiserror` per gli enum, `Option`/NaN per il ritiro geometrico) e applicarla uniformemente. `dtw.rs` dovrebbe usare un enum invece di `&'static str`.

### 9.4 Incoerenze di workspace inheritance e dipendenze

**Alto** — Tre crate su sette non usano workspace inheritance (§8.1). `[workspace.dependencies]` vuoto con versioni divergenti di proptest/serde (§8.1). Dipendenze inutilizzate in `semantic-graph` (§3.9).

### 9.5 Anomalie di stile ricorrenti

**Basso** — Dichiarazioni `pub mod` dopo blocchi `#[cfg(test)]`: `semantic-combiner/src/lib.rs` (alias a riga 285 dopo tests a riga 271), `semantic-graph/src/lib.rs` (`pub mod zonizzazione;` a riga 224 dopo tests), `semantic-walk/src/lib.rs` (mod a righe 181-189 dopo tests a righe 100-180). Pattern ricorrente: sembra che i moduli siano stati aggiunti in fondo al file senza riorganizzare. Raccomandazione: convenzione "moduli pubblici in cima, test in fondo".

**Nit** — `useless_vec!` diffuso nei test (24 warning in colbert, 2 in gate, 2 in quantum, 1 in walk — Appendice A). Pattern: `vec![vec![1.0, 0.0]]` dove basterebbe un array `[vec![1.0, 0.0]]`. Non è un bug, ma il volume (29 warning) suggerisce che i test sono stati scritti rapidamente senza passata di clippy.

### 9.6 Dead code ricorrente

**Basso** — API pubbliche dichiarate ma non consumate:
- `semantic-gate::Budget` (§2.3),
- `semantic-walk::TrajectoryStep` (§4.2),
- `semantic-quantum::QuantumResolver::horizon` (§7.3),
- `semantic-bridge::Walk::colbert` richiesto da `coerente()` ma non letto da `proiezione.rs` (§5.3).

Quattro casi di superficie pubblica morta. Raccomandazione: rimuovere o integrare. La dead code pubblica è peggio di quella privata perché impegna il contratto API senza fornire valore.

### 9.7 Complessità algoritmica — riepilogo

| Algoritmo | Complessità | Note |
|---|---|---|
| `combiner::combine` | O(1) | — |
| `combiner::pareto_compare` | O(1) | — |
| `colbert::maxsim` | O(N·M·D) | norma di `d` ridondante (§6.3, Alto) |
| `graph::costruisci` | O(N²) punteggio + **O(N³) lookup** | `find().unwrap()` (§3.3, Critico) |
| `graph::metriche::gradi` | O(V·E) | — |
| `graph::zonizzazione::similarita_traiettorie` | O(celle) + allocazioni | chiamata O(N²) volte nel sweep (§3.4, Medio) |
| `walk::dtw::align` | O(n·m·D) | banda Sakoe-Chiba riduce a O(n·w·D) |
| `walk::ordered_sparse::positional_jaccard` | O(tok_per_frame) | — |
| `quantum::collapse` | O(B) | HashMap accumulation |
| `quantum::estrai_frontiera_pareto` | O(B²) | con clonazioni |
| `quantum::estrai_frontiera_pareto_sui_candidati` | O(C²) | C << B tipicamente |
| `bridge::proiezione` | O(n·dim) | zero-alloc |

### 9.8 Sicurezza (unsafe)

Unico `unsafe` del workspace: `semantic-colbert/src/ffi.rs` (righe 102, 156-164). Tre problemi (§6.3): transmute senza annotazioni (Medio), cast `usize as i32` senza overflow check (Medio), `AtomicPtr` ingiustificato (Medio). Nessun altro crate usa unsafe. Per un sistema che fa FFI verso C++, la superficie unsafe è contenuta ma andrebbe hardenizzata con `debug_assert!` sui cast e transmute annotato.

### 9.9 Test — copertura complessiva

| Crate | Unit test | Integration test | proptest | Property chiave testate |
|---|---|---|---|---|
| combiner | sì | `pareto.rs` | sì (5 proprietà) | dominanza→ordine, antisimmetria, bounds, monotonicità, peso nullo |
| gate | sì | `coerenza.rs`, `pipeline.rs` | sì (3 proprietà) | sopra soglia mai Blocca, soglia zero permissiva, dominanza |
| graph | sì | `selfloop.rs`, `test_zonizzazione_reale.rs` | **no** | no self-loop |
| walk | sì | `dtw_robustezza.rs`, `integrazione_end_to_end.rs`, `scheletro_3_livelli.rs` | sì (in lib) | NaN handling, robustezza DTW |
| bridge | sì | — | **no** | coerenza walk/dizionario, dim mismatch |
| colbert | sì | — | **no** | identità, ortogonalità, NaN |
| quantum | sì | — | sì (in lib) | collapse con scorer NaN |

**Lacune principali**: graph e bridge non hanno proptest; nessun test cross-crate che verifichi la coerenza dei pesi (§9.1); nessun test sul caso Critico `align_with_ordered_sparse` con `n != m` (§4.3); nessun test sul mismatch adapter/parse multi-token (§4.3).

---

## Appendice A — Evidenza `cargo clippy --workspace --all-targets`

Comando eseguito da `c:\Work\tmp\semantic-geo`. Esito: **compilazione riuscita, 0 errori, warning presenti**. `Finished dev profile [unoptimized + debuginfo] target(s) in 1.46s`.

Riepilogo warning per crate (conteggi dal output completo salvato in `C:\Users\feder\.qoder\cache\projects\semantic-geo-734b808\agent-tools\ce906a82\56920c1d.txt`):

- **semantic-colbert**: 1 warning lib (`missing_transmute_annotations` a `src/ffi.rs:102`) + 24 warning test (23 `useless_vec` + 1 duplicato).
- **semantic-gate**: 1 warning lib (`should_implement_trait` su `Gate::default` a `src/lib.rs:100`) + 2 warning test (`useless_vec` in `tests/pipeline.rs:20-21`).
- **semantic-graph**: 1 warning lib (`unnecessary_sort_by` a `src/costruisci.rs:145`) + 1 warning test (`manual_range_contains` a `tests/test_zonizzazione_reale.rs:26`).
- **semantic-bridge**: 1 warning lib (`derivable_impls` su `impl Default for ProiezionePunti` a `src/proiezione.rs:30`) + 3 warning test (`identity_op` a `src/proiezione.rs:281`, `needless_range_loop` a `src/proiezione.rs:310`, 1 duplicato).
- **semantic-walk**: 1 warning lib (`too_many_arguments` su `align_with_ordered_sparse` a `src/dtw.rs:159`) + 2 warning test (`useless_vec` a `src/ordered_sparse.rs:356`, `identity_op` a `tests/scheletro_3_livelli.rs:143`).
- **semantic-quantum**: 2 warning lib (`if_same_then_else` a `src/pareto.rs:170`, `too_many_arguments` su `build_branch_from_tokens` a `src/lib.rs:207`) + 4 warning test (2 `useless_vec` a `src/lib.rs:376-377`, 2 duplicati) + 1 warning bin (`doc_overindented_list_items` a `src/bin/bench_pareto.rs:15`).
- **semantic-combiner**: 0 warning.

Totale: ~44 warning distinti (alcuni duplicati tra lib e test target). Nessuno è un errore. La maggioranza (29) è `useless_vec` nei test — nit di stile. I warning sostanziali sono: `missing_transmute_annotations` (colbert FFI), `should_implement_trait` (gate), `derivable_impls` (bridge), `if_same_then_else` (quantum pareto), `too_many_arguments` (walk dtw, quantum lib), `unnecessary_sort_by` (graph).

## Appendice B — Evidenza `cargo test --workspace --no-run`

Comando eseguito da `c:\Work\tmp\semantic-geo`. Esito: **compilazione riuscita, 0 errori**. `Finished test profile [unoptimized + debuginfo] target(s) in 14.11s`.

Binari di test generati (19 totali):
- unittests: `semantic_bridge`, `semantic_colbert`, `semantic_combiner`, `semantic_gate`, `semantic_graph`, `semantic_quantum`, `semantic_walk` (7 lib).
- integration tests: `pareto` (combiner), `coerenza` + `pipeline` (gate), `selfloop` + `test_zonizzazione_reale` (graph), `dtw_robustezza` + `integrazione_end_to_end` + `scheletro_3_livelli` (walk) (8 test bin).
- bin unittests: `bench_pareto`, `probe3`, `probe_dominance`, `probe_dominance2` (quantum, 4 bin).

Dipendenze compilate: `cfg-if`, `windows-link`, `fastrand`, `once_cell`, `bit-vec`, `wait-timeout`, `fnv`, `quick-error`, `unarray`, `regex-syntax`, `bitflags`, `memchr`, `serde_core`, `zerocopy`, `zmij`, `itoa`, `num-traits`, `getrandom` (2 versioni: 0.3.4 e 0.4.3), `windows-sys`, `bit-set`, `rand_core`, `rand_xorshift`, `rand`, `tempfile`, `rusty-fork`, `serde`, `serde_json`, `ppv-lite86`, `rand_chacha`, `proptest`.

**Nota**: `getrandom` compare in due versioni (0.3.4 e 0.4.3) — probabile conseguenza di dipendenze transitorie non allineate. Non è un errore ma indica che `Cargo.lock` potrebbe beneficiare di un `cargo update` selettivo o di un allineamento delle versioni richieste da proptest/rand.

---

## Riepilogo dei finding per severità

### Critico (1)
1. `semantic-walk/src/dtw.rs:213` — `sparse_a.positional_jaccard(sparse_b, i-1)` con `i` fino a `n`: se `n > m`, accesso out-of-bounds a `sparse_b.offsets[i]` → panic. Il check a riga 177 non valida `n == m`. (§4.3)

### Alto (8)
2. `semantic-combiner/src/lib.rs:138-147` — `combine` valida i pesi solo con `debug_assert!`: in release pesi negativi/non normalizzati producono punteggio errato silenziosamente. (§1.3)
3. `semantic-combiner/tests/pareto.proptest-regressions:8-9` — seed storici con pesi negativi che violano il contratto attuale. (§1.3)
4. `semantic-graph/src/costruisci.rs:132` — `fatti.iter().find(...).unwrap()` in libreria + lookup O(N) dentro O(N²) → O(N³). (§3.3)
5. `semantic-graph/src/costruisci.rs:25-28` — prodotto componente-wise come similarità: per sparse non limitata satura a 1.0, distrugge la discriminazione. (§3.3)
6. `semantic-colbert/src/lib.rs:83-98` — `norm_l2(d)` ricalcolata per ogni coppia (q,d) nel percorso caldo: O(N·M·D) ridondante. (§6.3)
7. `semantic-walk/src/adapter.rs:282` vs `src/parse.rs:186-191` — contratto incompatibile su `pos` (multi-token frame prodotto dall'adapter, rifiutato dal parser). (§4.3)
8. `semantic-quantum/src/lib.rs:283-289` — `evaluate_colbert_branch` bypassa lo scorer iniettato, rompendo il contratto di `BranchBuilder`. (§7.3)
9. Root `Cargo.toml` + 3 crate — workspace inheritance non usata da graph/walk/quantum; `[workspace.dependencies]` vuoto con versioni divergenti (proptest "1" vs "1.1", serde "1.0" vs "1"). (§8.1, §9.4)
10. `semantic-graph/Cargo.toml:10-11` — dipendenze `semantic-gate` e `semantic-colbert` dichiarate ma mai usate (0 match in src/tests/examples). (§3.9)

### Medio (15)
11. `semantic-combiner/src/lib.rs:58-72` — `normalize` non valida `sparse_lambda` (≤0 produce risultati inattesi). (§1.3)
12. `semantic-gate/src/budget.rs` + re-export — `Budget` è API pubblica morta, mai usata da `Gate::decide`. (§2.3)
13. `semantic-gate/src/sonda.rs` — sonda fissa `colbert=0.0` senza documentare che ignora deliberatamente il canale. (§2.3)
14. `semantic-graph/src/lib.rs:82-99` — `GraphConfig::default()` usa pesi `[0.6,0.25,0.15]` incoerenti con `PESI_CALIBRATI` del combiner. (§3.2, §9.1)
15. `semantic-graph/src/lib.rs:140-145` — `ha_arco` usa `binary_search` assumendo archi ordinati, invariante non documentata né validata. (§3.3)
16. `semantic-graph/src/zonizzazione.rs:similarita_traiettorie` — alloca 2 HashMap + 1 HashSet per chiamata, chiamata O(N²) volte nel sweep. (§3.4)
17. `semantic-graph/src/zonizzazione.rs` — usa `f32` mentre il resto del workspace usa `f64`; ignora il campo `pos` (similarità order-insensitive nonostante il nome "traiettoria"). (§3.3, §3.4, §9.2)
18. `semantic-graph/examples/sweep_percentile.rs` — duplica la funzione privata `percentile` da `costruisci.rs`; percorso relativo `zonizzazione/...` incoerente con `tests/test_zonizzazione_reale.rs` (`../zonizzazione/...`). (§3.7)
19. `semantic-walk/src/dtw.rs:202` — `Vec<Vec<f64>>` per la matrice di costo: n+1 allocazioni heap, contraddice la filosofia zero-alloc. (§4.3)
20. `semantic-walk/src/dtw.rs` — banda troppo stretta → `normalized_score = inf` restituito come `Ok` (documentato in `examples/test_dtw_inf.rs` come bug noto non risolto). (§4.3)
21. `semantic-walk/src/ordered_sparse.rs:143-148` — `sig[1] = sig[0].rotate_left(32)`: metà correlate, entropia effettiva ~64 bit invece di 128, indebolisce il pruning POPCNT. (§4.3)
22. `semantic-walk/src/ordered_sparse.rs:tokens_at(i)` — nessun bounds-check su `i`: panic se `i >= num_positions`. (§4.3)
23. `semantic-walk/src/dtw.rs:214-222` — interpolazione banda con troncamento (non arrotondamento) → bias sistematico verso bande più strette. (§4.4)
24. `semantic-colbert/src/ffi.rs:156-164` — cast `usize as i32` senza overflow check in codice unsafe FFI. (§6.3)
25. `semantic-colbert/src/ffi.rs:102` — `transmute` senza annotazioni di tipo. (§6.3)

### Basso (10)
26. `semantic-combiner/src/lib.rs:285-289` — alias `FactId`/`TrivectorScore` dichiarati dopo i test. (§1.2)
27. `semantic-gate/src/lib.rs:99-104` — `Gate::default()` come metodo inerente, non trait `Default` (clippy `should_implement_trait`). (§2.3)
28. `semantic-graph/src/lib.rs:224` — `pub mod zonizzazione;` in fondo al file dopo i test. (§3.2)
29. `semantic-graph/tests/selfloop.rs` — `eprintln!` per debug nei test. (§3.7)
30. `semantic-walk/src/lib.rs:181-189` — dichiarazioni `pub mod` dopo il blocco test. (§4.2)
31. `semantic-walk/src/lib.rs:91` — `TrajectoryStep` definito ma mai usato (dead code pubblico). (§4.2)
32. `semantic-bridge/src/lib.rs:61` + `proiezione.rs` — `Walk.colbert` richiesto da `coerente()` ma mai letto dalle proiezioni (dead field nel contratto). (§5.3)
33. `semantic-quantum/src/lib.rs:272` — campo `horizon` dichiarato e inizializzato ma mai letto (dead field). (§7.3)
34. `semantic-quantum/src/bin/probe_dominance.rs:34-40` — `aggrega` usa `(action, amplitude, action*amplitude)` come proxy di costi (si, sg, sc): semantica confusa, non etichettata come probe. (§7.3)
35. `zonizzazione/zonizza.py:49` — fallback silenzioso `pos[i] if i < len(pos) else i` senza warning su disallineamento walk/colbert. (§8.2)

### Nit (6)
36. `semantic-combiner/src/lib.rs:110-130` — `pareto_compare` senza epsilon su f64, caso NaN non documentato esplicitamente. (§1.4)
37. `semantic-colbert/src/lib.rs:107` — divisione per `query_tokens.len()` protetta solo implicitamente dal check precedente. (§6.4)
38. `semantic-bridge/src/proiezione.rs:281,310` — `identity_op` e `needless_range_loop` (clippy) in test code. (§5.4)
39. `semantic-quantum/src/bin/bench_pareto.rs:15` — `doc_overindented_list_items` (clippy). (§7.8)
40. Diffusione di `useless_vec!` nei test (29 warning clippy). (§9.5)
41. `zonizzazione/zonizza.py:97` — `from collections import defaultdict` dentro la funzione; `:82` `'dim': 1024` hardcoded. (§8.2)

---

## Raccomandazioni prioritarie per la revisione

1. **Risolvere il Critico §4.3** (`dtw.rs:213`): validare `n == m` all'ingresso di `align_with_ordered_sparse` oppure clampare l'indice. Aggiungere un test che oggi paniccherebbe.
2. **Eliminare l'Alto §3.3** (`costruisci.rs:132`): sostituire `find().unwrap()` con una `HashMap<NodeId, &Fatto>` pre-costruita (riduce O(N³)→O(N²) e rimuove il panic).
3. **Indurire l'Alto §1.3** (`combine`): sostituire `debug_assert!` con validazione che restituisce NaN o `Result`, coerente con la filosofia del ritiro. Rigenerare i seed proptest.
4. **Ottimizzare l'Alto §6.3** (`maxsim`): pre-calcolare le norme di `doc_tokens` una volta. Percorso caldo usato da 4 crate.
5. **Risolvere l'Alto §7.3** (`evaluate_colbert_branch`): usare `self.colbert_scorer.compute_maxsim(...)` per coerenza con `BranchBuilder`.
6. **Risolvere l'Alto §4.3** (adapter/parse mismatch): decidere se il parser accetta `pos` non-decrescente o se l'adapter assegna `pos` progressivi. Il caso multi-token è dichiarato "futuro" ma il contratto è già rotto.
7. **Uniformare il workspace (§8.1, §9.4)**: workspace inheritance per tutti i crate, `[workspace.dependencies]` popolato, rimuovere dipendenze inutilizzate da `semantic-graph`.
8. **Documentare le scelte architetturali (§9.1, §9.2, §9.3)**: un `ARCHITECTURE.md` che spieghi la divergenza dei pesi, la doppia filosofia dell'ignoto (ritiro vs penalizzazione), e la scelta f32/f64 per crate.
9. **Rimuovere o integrare la dead code pubblica (§9.6)**: `Budget`, `TrajectoryStep`, `horizon`, `Walk.colbert`.
10. **Estendere i test (§9.9)**: proptest per graph e bridge; test cross-crate sulla coerenza dei pesi; test sul caso Critico §4.3 e sul mismatch §4.3.
