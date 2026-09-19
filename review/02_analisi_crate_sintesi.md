# Analisi Crate di Sintesi — Graph, Quantum, Bridge

Analisi dettagliata dei tre crate che compongono il livello di sintesi: grafo di prossimità, resolver quantistico, e ponte geometrico.

---

## semantic-graph

### Scopo e API

Costruisce un grafo di prossimità pesato sui fatti della memoria usando strategia ibrida k-nearest + soglia. Espone metriche strutturali (degree, clustering coefficient, centralità, orfano).

**API pubblica:**
- `NodeId(usize)` — identificatore nodo
- `Edge { from, to, weight }` — arco pesato non-direzionato
- `GraphConfig { k, soglia }` — parametri costruzione
- `Graph { nodi, archi }` — grafo risultante
- `costruisci(fatti, config) -> Graph` — costruzione grafo
- `Fatto { id, dense, sparse, colbert }` — fatto con tre assi di similarità
- `metriche::degree(graph, node) -> usize`
- `metriche::clustering_coefficient(graph, node) -> f64`
- `metriche::orfani(graph) -> Vec<NodeId>`
- `zonizzazione` — modulo per caricamento dizionario/traiettorie JSON

### Correttezza

**Issue S1 — Test Pareto vacuo:**  
Il test di coerenza Pareto usa k=5 con soli 3 fatti e soglia=0.0. Con queste impostazioni, ogni fatto con probes positive passa — il test non esercita mai il pruning Pareto.

**Issue S2 — `Fatto::punteggio` bypassa normalizzazione:**

```rust
// costruisci.rs:22-27
let axes = NormalizedAxes {
    dense: self.dense * altro.dense,
    sparse: self.sparse * altro.sparse,
    colbert: self.colbert * altro.colbert,
};
```

Costruisce `NormalizedAxes` per struct literal senza passare da `normalize()`. Il prodotto di due valori in [0,1] è in [0,1], quindi matematicamente non rompe l'invariante — ma viola il contratto del combiner (che esiste per gestire NaN, inf, e valori fuori range).

**Issue S2 — Similarità query-dependent:**  
Il grafo misura "entrambi i fatti matchano la stessa query" (prodotto dei punteggi verso un query esterno), non "i fatti sono simili tra loro" (similarità intrinseca). Questo contraddice il pattern "build once, consult many" documentato in DESIGN.md.

**Issue S1 — Pareto tie-break (costruisci.rs:62-67):**

```rust
candidati.sort_by(|a, b| {
    b.0.partial_cmp(&a.0)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then(a.1.cmp(&b.1))  // ← tie-break per NodeId ascendente
});
candidati.truncate(k);
```

Quando due candidati hanno punteggio identico, il tie-break favorisce il NodeId più basso. Ma il pruning Pareto (riga successiva) confronta solo il candidato corrente con quelli già selezionati. Se il dominato ha ID minore del dominatore, il dominato viene selezionato per primo e il dominatore viene successivamente scartato.

**Issue S2 — NaN soglia → grafo vuoto:**  
Se `config.soglia` è NaN, il confronto `score >= soglia` è sempre false → nessun arco creato → grafo vuoto. Nessuna segnalazione. Polarità opposta al gate (che con NaN è permissivo).

**Issue S2 — Dedup silenzioso:**  
Se due archi con stessi `(from, to)` vengono generati (da lati opposti della coppia), il codice deduplica tenendo il primo — senza merge policy (max? average? last-wins?).

### Gestione Errori

Il modulo `zonizzazione` usa `Result<T, Box<dyn std::error::Error>>` con `?` per I/O. Le funzioni del grafo sono totfunzionali (nessun Result). NaN non è gestito esplicitamente — si propaga silenziosamente.

### Edge Cases

| Input | Comportamento | Corretto? |
|-------|--------------|-----------|
| Lista fatti vuota | Graph { nodi: 0, archi: 0 } | ✓ |
| k > n (più vicini che fatti) | Tronca a n-1 (tutti gli altri) | ✓ |
| k = 0 | Nessun vicino → grafo vuoto | ✓ (ma non testato) |
| soglia = NaN | Grafo vuoto silenzioso | ⚠ |
| probes = NaN | partial_cmp → Equal, punteggio NaN → sotto soglia | ⚠ |
| IDs duplicati | Dedup silenzioso (first wins) | ⚠ |
| Singolo fatto | Grafo con 1 nodo, 0 archi | ✓ |

### Performance

- **O(n² log n)** per la costruzione: per ogni nodo, sort completo di tutti gli altri (dovrebbe essere `select_nth_unstable` per top-k: O(n²))
- **O(N × E)** per le metriche: `clustering_coefficient` itera tutti gli archi per ogni nodo (manca indice di adiacenza)
- **Zero-alloc claim falso**: `vicini()` alloca un Vec a ogni chiamata; `candidati` è allocato per ogni nodo

**Fix suggerito:** pre-computare adjacency list (HashMap<NodeId, Vec<(NodeId, f64)>>), usare `select_nth_unstable_by` per top-k.

### Test Coverage

- 15 unit test inline + 2 integration test
- `selfloop.rs`: verifica che nessun self-loop sia creato
- `test_zonizzazione_reale.rs`: carica JSON reali (ma **false-green** se file assenti — ritorna senza assert)
- **Nessun proptest** nonostante DESIGN.md lo prometta

**Gap:** nessun test per k>n, k=0, NaN soglia, NaN probes, single-node, duplicati conflittuali, clustering coefficient, `archi_di()`.

### Documentazione

- DESIGN.md presente ma **stantio**: menziona "centrality" e "betweenness" non implementate; menziona un tipo `Node` che non esiste; le checkbox di stato sono tutte superate
- Rustdoc presente ma minimale

### Raccomandazioni Specifiche

1. **P0:** Fix Pareto tie-break (usare dominanza effettiva nel confronto, non ID ordering)
2. Usare `NormalizedAxes::normalize()` in `Fatto::punteggio`
3. Fix false-green in `test_zonizzazione_reale.rs` (assert su file presenti o skip esplicito)
4. Aggiungere validazione NaN per soglia (ritornare Result o documentare comportamento)
5. Sostituire sort completo con `select_nth_unstable` (O(n² log n) → O(n²))
6. Aggiungere proptest per coerenza Pareto con configurazioni non-vacue
7. Aggiornare DESIGN.md (rimuovere funzionalità non implementate)
8. Aggiungere adjacency index per metriche O(1)
9. Validare dimensione vettori in `zonizzazione` (centroid count, dim match)
10. Documentare la natura query-dependent del grafo (o cambiarla)

---

## semantic-quantum

### Scopo e API

Resolver ispirato alla meccanica quantistica: i percorsi nel grafo di walk diventano "rami" con ampiezza complessa; il collasso seleziona il candidato con interferenza costruttiva massima.

**API pubblica:**
- `QuantumResolver { divergence_threshold, weights, horizon, kappa_break }` — resolver
- `BranchBuilder { colbert_scorer, weights }` — costruttore rami (builder pattern)
- `WalkBranch { branch_type, candidate_id, action, amplitude }` — ramo quantistico
- `BranchCostVector { inertial, geometric, colbert }` — vettore di costo
- `collapse(branches) -> Option<WalkBranch>` — collasso della funzione d'onda
- `estrai_frontiera_pareto(branches) -> Vec<&WalkBranch>` — pruning Pareto

### Correttezza

**Issue S1 — Range ColBERT violato (lib.rs:147-154, 197):**

Il contratto di `maxsim` è [-1, 1] (documentato in semantic-colbert). Ma:

```rust
// compute_colbert_cost (linea 152):
(1.0 - maxsim as f32).clamp(0.0, 1.0)
// Se maxsim = -0.5 → (1.0 - (-0.5)).clamp(0,1) = 1.5.clamp(0,1) = 1.0
// Distruzione della discriminazione per cosine negative

// build_branch (linea 197):
let s_colbert = (1.0 - colbert_similarity).max(0.0);
// Se colbert_similarity = -0.5 → (1.0 - (-0.5)).max(0.0) = 1.5
// NON clampato sopra! Due politiche opposte nello stesso crate.
```

**Issue S1 — NaN in `build_branch` (linea 197):**

```rust
let s_colbert = (1.0 - colbert_similarity).max(0.0);
```

Se `colbert_similarity` è NaN: `(1.0 - NaN) = NaN`, `NaN.max(0.0) = 0.0` (stesso bug di dtw.rs). Il ramo ottiene **costo minimo** — il più attraente. Esattamente l'opposto di `compute_colbert_cost` che assegna penalità massima (1.0).

**Issue S1 — NaN nel collapse (lib.rs:255-272):**

1. Filtro decoerenza: `b.action > self.divergence_threshold` — con `action = NaN`, il confronto è **false** → il ramo passa il filtro
2. Accumulo: `*accum.entry(cid).or_insert(0.0) += NaN` → l'intero accumulatore per quel candidato diventa NaN
3. Selezione: `a.1.partial_cmp(b.1).unwrap_or(Equal)` — NaN vs anything = Equal → il vincitore dipende dall'ordine di iterazione HashMap (non-deterministico tra run)

**Risultato:** un singolo ramo NaN può rendere il vincitore random.

**Issue S1 — NaN in `dominates`:**  
Tutti i confronti con NaN sono false → `NaN_dominates(X)` = false E `X_dominates(NaN)` = false → il ramo NaN è **immortale** (mai eliminato dal pruning Pareto).

**Issue S1 — `bench_pareto` degenerato:**
- Branch 0 ancorato a (0.05, 0.05, 0.05) — domina sempre tutti gli altri
- LCG custom ritorna valori solo in [0, 0.5) — range dimezzato
- Colonne "Sorted" nel benchmark misurano l'adaptive gate, non un algoritmo sorted
- Le costanti di calibrazione (λ, gate threshold) sono derivate da questi dati rotti

### Gestione Errori

- `collapse` ritorna `Option<WalkBranch>` — None se nessun ramo valido
- Nessun Result, nessun panic
- NaN non è gestito come caso speciale in nessun punto

### Edge Cases

| Input | Comportamento | Corretto? |
|-------|--------------|-----------|
| branches vuoto | None | ✓ |
| Tutti candidate_id = None | None | ✓ |
| Tutti action > threshold | None | ✓ |
| NaN action | Passa filtro, avvelena accum | ✗ |
| NaN amplitude | Avvelena accum | ✗ |
| Singolo branch | Vince sempre | ✓ |
| Tie tra due candidati | Non-deterministico (HashMap) | ⚠ |

### Performance

- `collapse`: O(B) dove B = numero rami (singola passata + HashMap)
- `estrai_frontiera_pareto`: O(B²) nel caso peggiore (confronto a coppie)
- `bench_pareto`: O(B² log B) per sort — ma i dati sono degenerati

**Issue S4:** accumulatore `f32` per somma ampiezze — perde precisione con molti rami (oltre ~1000 rami con ampiezze molto diverse, la somma diventa inaccurata).

### Test Coverage

- 27 test inline (tutti pass)
- Proptest genuini (frontiera Pareto, dominanza)
- Test per builder pattern, collapse, adaptive gate
- **Buona** — ma nessun test con NaN inputs

**Issue S2 — Campi dead:**  
`horizon` e `kappa_break` in `QuantumResolver` non sono mai letti dopo la costruzione. Il resolver li memorizza ma il collapse non li usa — l'ampiezza arriva già calcolata dal branch.

**Issue S2 — `evaluate_colbert_branch`:**  
Bypassa la dependency injection (chiama la free function `maxsim` invece dello scorer iniettato). Inoltre, non ha nessun chiamante nel workspace.

### Documentazione

Il crate meglio documentato dei tre. Doc-comment estesi con formule LaTeX. DESIGN.md coerente con l'implementazione. Le motivazioni fisiche sono spiegate.

### Raccomandazioni Specifiche

1. **P0:** Unificare politica NaN (scartare rami NaN nel collapse, penalità massima nel build)
2. **P0:** Fix range ColBERT — accettare [-1,1] e mappare a costo [0,2] oppure clampare in ingresso
3. **P0:** Rigenerare bench_pareto con distributore corretto
4. Rimuovere o utilizzare `horizon` e `kappa_break`
5. Rimuovere `evaluate_colbert_branch` (dead code con DI bypass)
6. Usare `BTreeMap` o sort deterministico per il tie-break nel collapse
7. Promuovere accumulatore a `f64`
8. Aggiungere proptest con NaN inputs
9. Documentare che `semantic-walk` è usato solo nei test (non è dipendenza produttiva reale)
10. Correggere i nomi dei pesi: (0.215/0.552/0.233) sono tipizzati inertial/geometric/colbert ma etichettati dense/sparse nei commenti

---

## semantic-bridge

### Scopo e API

Trasforma il cammino discreto di celle (dalla zonizzazione) in punti geometrici continui per il consumo da parte del DTW. Il "ponte" tra il mondo discreto delle celle e il continuo degli embedding.

**API pubblica:**
- `Walk<'a> { celle, pos, st, w, conf, dim }` — cammino di celle
- `Dizionario<'a> { centroidi, dim, k }` — dizionario celle
- `WalkStep { cella, pos, st, w, conf }` — singolo passo (mai usato esternamente)
- `TrasformazionePonte` trait — interfaccia di proiezione
- `ProiezionePunti { Centroide, MediaPesata }` — strategie di proiezione
- `OrchestratorePonte` trait — orchestrazione (inimplementabile)
- `ErrorePonte { BufferInadeguato, DatiInconsistenti }` — enum errori (mai costruita)
- `Centroide` / `MediaPesata` — implementazioni proiezione

### Correttezza

**Issue S1 — Dimensione mai validata (proiezione.rs):**

```rust
// Il codice usa dizionario.dim per la proiezione
let d = dizionario.dim;
// Ma walk.dim non è MAI confrontato con dizionario.dim
```

Se `walk.dim = 768` e `dizionario.dim = 1024`, la proiezione produce punti 1024-dimensionali da celle che rappresentano spazi 768-dimensionali. Il risultato è `Some(...)` — nessuna segnalazione di errore.

**Issue S1 — NaN confidence → output Some:**  
`MediaPesata` usa `conf` come peso. Se un conf è NaN:
- `peso_tot += NaN` → peso_tot = NaN
- Guard: `peso_tot <= 0.0` → NaN <= 0.0 è **false** → il guard non scatta
- Divisione: `out[j] / NaN` = NaN
- Risultato: punti NaN ritornati come `Some()`

Il contratto dice "None = ritiro geometrico" ma NaN dovrebbe essere un ritiro, non un output valido.

**Issue S2 — `OrchestratorePonte` inimplementabile:**  
Il trait ha un metodo con lifetime self-referential (`&'a self` che ritorna `&'a [f64]` legato al buffer del chiamante). Come documentato in `ORCHESTRATORE_ANALISI.md:38`: "il trait non è implementabile in Rust safe." È pubblicato ma inutilizzabile.

**Issue S2 — `ErrorePonte` mai costruita:**  
L'enum è definita con due varianti (`BufferInadeguato`, `DatiInconsistenti`) ma non è mai usata — la proiezione ritorna `Option` (None), non `Result`. La tassonomia errori documentata è irrealizzata.

**Issue S2 — `walk.colbert` validato ma mai letto:**  
`Walk::coerente()` verifica che il vettore colbert abbia la lunghezza giusta, ma `proiezione()` non lo usa mai. Questo forza il chiamante a materializzare ~550KB di dati ColBERT per niente.

**Issue S2 — `MediaPesata` rompe invariante 1:1:**  
Documentato come "proiezione passo-per-passo" ma in realtà smootha con i vicini (finestra ±1). Il passo i-esimo nel output NON corrisponde solo alla cella i-esima — è una media delle celle i-1, i, i+1.

**Issue S2 — Confidence negativa:**  
Il guard verifica solo `peso_tot <= 0.0`. Con confidence negative ma somma positiva (es: conf = [-1, 3, -1], somma = 1), la proiezione produce punti "extrapolati" geometricamente invalidi.

### Gestione Errori

- `Option<&[f64]>` — None per ritiro geometrico (buffer insufficiente, walk incoerente, cella fuori range)
- Nessun `Result` usato (nonostante `ErrorePonte` esista)
- Nessun panic possibile

### Edge Cases

| Input | Comportamento | Corretto? |
|-------|--------------|-----------|
| walk vuoto (celle.len()=0) | `coerente()` = false → None | ✓ |
| buffer troppo piccolo | None | ✓ |
| cella ≥ dizionario.k | None | ✓ |
| walk.dim ≠ dizionario.dim | Some() con dati meaningless | ✗ |
| conf = NaN | Some() con output NaN | ✗ |
| conf negativo (somma > 0) | Some() con extrapolazione | ⚠ |
| Singolo passo | Proiezione Centroide: ok; MediaPesata: finestra=[0,0] | ✓ |

### Performance

- **Zero-alloc genuino**: scrive direttamente nel buffer del chiamante, nessuna allocazione intermedia
- **Element-by-element widening f32→f64**: per d=1024, ogni punto richiede 1024 conversioni scalar (nessuna vettorizzazione automatica garantita)
- **MediaPesata**: rilegge i centroidi dei vicini fino a 3× (per ogni punto nella finestra)

**Fix suggerito:** per `Centroide`, usare `copy_from_slice` con cast batch (o `bytemuck` se il tipo lo permette). Per `MediaPesata`, cachare il punto precedente e riusarlo.

### Test Coverage

- Unit test per `coerente()`, proiezione base, buffer insufficiente
- Integration test per pipeline walk→proiezione→DTW
- **Nessun proptest** (dichiarato in dev-deps ma mai usato)

**Gap:** nessun test per dim mismatch, NaN/negative conf, single-step walk, MediaPesata interior points, buffer reuse.

### Documentazione

Il miglior rustdoc del workspace (zero warning `cargo doc`). I moduli sono ben spiegati con esempi. `ORCHESTRATORE_ANALISI.md` è un documento onesto che spiega perché l'orchestratore non funziona. DESIGN.md presente ma contraddice l'implementazione in più punti.

### Raccomandazioni Specifiche

1. **P0:** Validare `walk.dim == dizionario.dim` prima della proiezione
2. **P0:** Gestire NaN confidence → ritornare None (ritiro geometrico)
3. Rimuovere o riscrivere `OrchestratorePonte` (pubblicato ma inimplementabile)
4. Rimuovere `ErrorePonte` o migrare a `Result` (coerenza con la documentazione)
5. Validare conf >= 0.0 per ogni passo (o documentare comportamento con negativi)
6. Rimuovere `walk.colbert` dalla validazione se non è usato nella proiezione
7. Aggiungere proptest (già in dev-deps)
8. Aggiornare DESIGN.md per riflettere l'implementazione reale
9. Considerare `copy_from_slice` + cast per la proiezione Centroide
10. Documentare che MediaPesata rompe l'invariante 1:1 step↔cell

---

[Torna al sommario](./00_sommario_esecutivo.md)
