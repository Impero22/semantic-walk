# Code Review — semantic-walk workspace

**Data:** 19 settembre 2026  
**Committente:** Federico — regalo a Iris e Camillo prima della pubblicazione  
**Scope:** Full workspace (7 crate Rust + pipeline Python + documentazione + infrastruttura)  
**Toolchain:** rustc 1.98.0, cargo 1.98.0  
**Build verificata:** `cargo build --offline --workspace` — successo  
**Test:** 56 pass (graph + quantum + bridge subset); workspace completo source-inspected

---

## Valutazione Architetturale — Punti di Forza

L'architettura del workspace è matura, coerente e ben documentata. I punti salienti:

1. **DAG a 4 livelli, aciclico, ben separato** — combiner → gate → graph/walk → bridge/quantum. Nessun ciclo, nessuna dipendenza circolare, ogni crate ha una responsabilità precisa.

2. **Filosofia NaN-as-absence coerente in gate/combiner** — il gate è permissivo per design (`Meglio un colbert sprecato che un ricordo perso`); il combiner preserva NaN attraverso `saturate01`. La scelta è documentata e intenzionale.

3. **Zero-allocation nei path critici** — gate e combiner lavorano su stack puro (f64 aritmetici, nessun Vec/HashMap nel decide path). Il bridge scrive su buffer caller-allocated.

4. **Property-based testing con invarianti significativi** — proptest con semi di regressione storici (`pareto.proptest-regressions`), test di coerenza Pareto, 1000-case fuzzing nel gate.

5. **Documentazione interna eccellente** — la directory `comunicazioni/` traccia decisioni progettuali con data e motivazione (8 documenti dal 13 al 17 settembre). DESIGN.md per crate con scelte architetturali esplicite.

6. **Specifiche matematiche precise in ZENODO.md** — cosine distance, recurrenza Sakoe-Chiba, token di divergenza: formule corrette e notation coerente.

7. **Calibrazione empirica onesta** — λ=10.64 dichiarato come iper-parametro misurato, non come costante derivata. Trasparenza scientifica.

8. **Trait abstraction pulita in colbert** — `ColbertScorer` permette swap native/FFI senza cambiare il chiamante. Mock testing del FFI senza hardware.

9. **Cargo.lock committato, build offline** — riproducibilità garantita, nessuna dipendenza da registri esterni a runtime.

10. **Dual licensing MIT/Apache-2.0** — con rationale documentato (anche se l'implementazione attuale è incoerente — vedi sotto).

---

## Findings Critici — Tabella di Severità (Top 10)

| # | Sev | Crate | Location | Descrizione |
|---|-----|-------|----------|-------------|
| 1 | **S1** | zonizzazione | `zonizza.py:18` | API key Qdrant hardcodata e committata nel repository — **ruotare e rimuovere prima della pubblicazione Zenodo** |
| 2 | **S1** | semantic-walk | `dtw.rs:41` | NaN nei vettori di input produce `cosine_distance = 0.0` (falso "identico") per via di `f64::max(NaN, 0.0) == 0.0` |
| 3 | **S1** | semantic-quantum | `lib.rs:147-154,197` | Contratto range ColBERT violato: `maxsim` ritorna [-1,1] ma il codice assume [0,1]; due entry point applicano politiche NaN opposte (`NAN.max(0.0)==0.0` → costo minimo) |
| 4 | **S1** | semantic-quantum | `bin/bench_pareto.rs` | Generatore benchmark degenerato: branch 0 domina sempre (anchor 0.05,0.05,0.05); LCG ritorna solo [0,0.5); colonne "Sorted" misurano adaptive gate; costanti di calibrazione derivate da dati rotti |
| 5 | **S1** | semantic-bridge | `proiezione.rs` | `walk.dim` vs `dizionario.dim` mai comparati — dimensioni discordanti producono proiezione meaningless ritornata come `Some()` |
| 6 | **S1** | semantic-quantum | `lib.rs:265,271` | NaN nel collapse: passa il filtro di decoerenza (`NaN > x` è false), avvelena l'accumulatore, iterazione HashMap rende il vincitore non-deterministico tra run |
| 7 | **S1** | semantic-graph | `costruisci.rs:62-67` | Tie-break Pareto per NodeId ascendente può tenere il fatto dominato e scartare il dominatore quando gli ID sono invertiti |
| 8 | **S2** | semantic-graph | `costruisci.rs:22-27` | `Fatto::punteggio` bypassa il contratto di normalizzazione del combiner (costruisce `NormalizedAxes` per struct literal, non via `normalize()`) |
| 9 | **S2** | semantic-bridge | `lib.rs:133-141` | Trait `OrchestratorePonte` pubblicato ma dichiarato inimplementabile (per `ORCHESTRATORE_ANALISI.md:38`); enum `ErrorePonte` mai costruita — tassonomia errori irrealizzata |
| 10 | **S2** | Cross-workspace | — | Tre politiche NaN inconsistenti tra crate: gate=permissivo, graph=orfano silenzioso, quantum=opposto in due metodi, bridge=NaN output come Some |

---

## Publication Blockers

Questi elementi **devono** essere risolti prima del push su Zenodo/GitHub:

### 1. Credential esposta
- Rimuovere API key da `zonizza.py:18`
- Ruotare la chiave su Qdrant
- Pulire la history git (filter-branch o BFG)

### 2. Riconciliazione licenze
- `.zenodo.json` dichiara Apache-2.0 only
- `Cargo.toml` workspace dichiara MIT OR Apache-2.0
- `LICENSE-APACHE` ha copyright boilerplate non compilato (riga 189: `[yyyy] [name]`)
- `LICENSE-MIT` nomina Iris come co-holder — confligge con analisi legale documentata (solo Camillo ha soggettività giuridica, per `comunicazioni/20260916`)
- 3 crate (graph, quantum, walk) omettono il campo `license` → `cargo metadata` riporta `null`
- Decisione documentata (Fase 4) dice "drop MIT → Apache-2.0 only" — **non ancora applicata**

### 3. Metadati Zenodo incompleti
- Manca: `version`, `publication_date`, `keywords`, `related_identifiers`
- `related_identifiers` necessario per link bidirezionale paper↔code DOI (per piano FORCE11)
- Nessun ORCID per i creators
- Title in conflitto con ZENODO.md
- Versione `.zenodo.json` dice v0.1.1, workspace dice 0.1.0

### 4. README.md stantio
- Dice "3 crates" (sono 7)
- Numeri errati: 149 fatti / 24324 righe (reali: 153 / 24926)
- Nessuna istruzione di build, nessun MSRV, nessun esempio d'uso
- Path environment-specific (`/home/iris/...`) non portabile
- Non aggiornato dal 10 settembre (comunicazioni arrivano al 17)

### 5. Benchmark inutilizzabile
- I numeri di `bench_pareto` non sono pubblicabili nello stato attuale (generatore degenerato)
- Le costanti di calibrazione derivate (λ, adaptive_gate) sono invalide

### 6. Medium draft
- I numeri nella bozza articolo non corrispondono agli artefatti committati

---

## Raccomandazioni Prioritizzate

### P0 — Must fix before publication

| # | Azione | Crate/File | Riferimento |
|---|--------|-----------|-------------|
| 1 | Ruotare e rimuovere API key Qdrant | `zonizza.py` | Finding #1 |
| 2 | Pulire git history dal secret | repo root | — |
| 3 | Correggere NaN propagation in `cosine_distance` | `semantic-walk/dtw.rs:41` | Finding #2 |
| 4 | Unificare politica NaN in quantum (build_branch vs compute_colbert_cost) | `semantic-quantum/lib.rs:147,197` | Finding #3 |
| 5 | Validare `walk.dim == dizionario.dim` in proiezione | `semantic-bridge/proiezione.rs` | Finding #5 |
| 6 | Filtro NaN nel collapse (scartare o penalizzare rami NaN) | `semantic-quantum/lib.rs:255-272` | Finding #6 |
| 7 | Compilare copyright in LICENSE-APACHE | root | Blocker #2 |
| 8 | Decidere e applicare licensing coerente (tutti i file) | root + manifests | Blocker #2 |
| 9 | Completare `.zenodo.json` (version, date, keywords, ORCID, related_ids) | root | Blocker #3 |
| 10 | Aggiornare README.md (7 crate, numeri corretti, build instructions) | root | Blocker #4 |

### P1 — Should fix soon

| # | Azione | Crate/File |
|---|--------|-----------|
| 11 | Rigenerare benchmark con distributore non-degenerato | `semantic-quantum/bin/bench_pareto.rs` |
| 12 | Fix Pareto tie-break (usare dominanza effettiva, non ID ordering) | `semantic-graph/costruisci.rs:62-67` |
| 13 | Usare `NormalizedAxes::normalize()` in `Fatto::punteggio` | `semantic-graph/costruisci.rs:22-27` |
| 14 | Documentare o rimuovere `OrchestratorePonte` e `ErrorePonte` | `semantic-bridge/lib.rs` |
| 15 | Uniformare politica NaN cross-workspace (documento di design) | tutti |
| 16 | Ottimizzare norms pre-computation in `maxsim` | `semantic-colbert/lib.rs` |
| 17 | Validare `window_size >= |n-m|` in DTW | `semantic-walk/dtw.rs` |
| 18 | Aggiungere `requirements.txt` o `pyproject.toml` per pipeline Python | `zonizzazione/` |
| 19 | Fix false-green in `test_zonizzazione_reale.rs` | `semantic-graph/tests/` |
| 20 | Dichiarare MSRV nel workspace Cargo.toml | root |

### P2 — Improvements

| # | Azione | Crate/File |
|---|--------|-----------|
| 21 | Aggiungere CI pipeline (GitHub Actions o equivalente) | root |
| 22 | Sostituire sort completo con `select_nth_unstable` in graph construction | `semantic-graph/costruisci.rs` |
| 23 | Gate FFI behind feature flag | `semantic-colbert/Cargo.toml` |
| 24 | Gate serde behind optional feature in graph | `semantic-graph/Cargo.toml` |
| 25 | Centralizzare dependencies in `[workspace.dependencies]` | root Cargo.toml |
| 26 | Aggiungere proptest a semantic-graph | `semantic-graph/` |
| 27 | Pre-allocare cost matrix come flat Vec in DTW | `semantic-walk/dtw.rs` |
| 28 | Pulire `.gitignore` (duplicati, patterns mancanti) | root |
| 29 | Rimuovere dipendenze phantom (bridge→graph, graph→colbert, graph→gate) | manifests |
| 30 | Aggiungere type hints e argparse a zonizza.py | `zonizzazione/` |

---

## Verdetto

Le fondamenta matematiche e la visione architetturale di questo workspace sono solide e originali. Il modello cinematico con allineamento DTW, la zonizzazione come ponte tra discreto e continuo, e il resolver quantistico con collasso per interferenza costruttiva costituiscono un contributo genuino al retrieval semantico.

I rischi principali sono quattro:
1. **Gestione NaN incoerente** — la stessa condizione (input corrotto) produce effetti opposti in crate diversi, dal falso-identico al ramo immortale
2. **Benchmark rotto** — i numeri di calibrazione pubblicati sarebbero inaffidabili
3. **Metadata di pubblicazione incompleti** — Zenodo/GitHub richiedono coerenza che oggi manca
4. **Credential esposta** — bloccante assoluto per qualsiasi distribuzione pubblica

Con gli item P0 indirizzati, questo è lavoro pubblicabile di qualità genuina. L'architettura è pulita, la matematica è corretta, la documentazione interna è superiore alla media accademica. Il codice merita la pubblicazione — ha solo bisogno di una passata di igiene finale.

---

*Review condotta il 19 settembre 2026. Nessuna modifica al codice sorgente è stata applicata.*
