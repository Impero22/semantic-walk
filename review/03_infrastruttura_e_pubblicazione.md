# Infrastruttura e Pubblicazione

Analisi dell'infrastruttura di build, configuration, licensing, e readiness per la pubblicazione su Zenodo/GitHub.

---

## Configurazione Workspace

### Root Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "semantic-bridge",
    "semantic-colbert",
    "semantic-combiner",
    "semantic-gate",
    "semantic-graph",
    "semantic-quantum",
    "semantic-walk",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# ← VUOTO
```

**Osservazioni:**
- 7 membri dichiarati, tutti presenti sul filesystem ✓
- `resolver = "2"` corretto per edition 2021 ✓
- `[workspace.dependencies]` dichiarato ma **vuoto** — ogni crate ridichiara le proprie versioni
- Nessuna centralizzazione delle dipendenze condivise (serde, proptest)

### Cargo.lock

- Committato nel repository ✓
- Risolve offline (nessuna dipendenza da registri esterni) ✓
- Tutte le dipendenze da crates.io (nessun git, nessun path esterno)
- **Versioni duplicate** (solo compile-time/dev, nessun impatto runtime):
  - `getrandom` 0.3 + 0.4
  - `r-efi` 5 + 6
  - `syn` 2 + 3
- `zmij 1.0.23` (ryu replacement in serde_json) — leggermente esotico, da monitorare

### MSRV

Nessun MSRV dichiarato. Il minimo de facto è ≥1.82 per via della sintassi `unsafe extern` nel modulo FFI di semantic-colbert. Raccomandazione: aggiungere `rust-version = "1.82"` in `[workspace.package]`.

### Strumenti di sicurezza assenti

- Nessun `cargo-deny` (policy licensing/advisories)
- Nessun `cargo-audit` (vulnerabilità note)
- Nessuna CI pipeline (GitHub Actions, GitLab CI)
- Nessun `cargo-geiger` (unsafe audit)

---

## Coerenza Manifest per Crate

| Crate | version.workspace | edition.workspace | license | repository | rust-version | features |
|-------|:-:|:-:|:-:|:-:|:-:|:-:|
| semantic-bridge | ✓ | ✓ | ✓ (ereditato) | ✗ | ✗ | ✗ |
| semantic-colbert | ✓ | ✓ | ✓ (ereditato) | ✗ | ✗ | ✗ |
| semantic-combiner | ✓ | ✓ | ✓ (ereditato) | ✗ | ✗ | ✗ |
| semantic-gate | ✓ | ✓ | ✓ (ereditato) | ✗ | ✗ | ✗ |
| semantic-graph | **✗ hardcode** | **✗ hardcode** | **✗ omesso** | ✗ | ✗ | ✗ |
| semantic-quantum | **✗ hardcode** | **✗ hardcode** | **✗ omesso** | ✗ | ✗ | ✗ |
| semantic-walk | **✗ hardcode** | **✗ hardcode** | **✗ omesso** | ✗ | ✗ | ✗ |

**Problema:** 3 crate su 7 non ereditano il license dal workspace. `cargo metadata` riporta `license: null` per questi crate — bloccante per pubblicazione su crates.io.

**Nessun crate dichiara:** `repository`, `homepage`, `documentation`, `readme`, `keywords`, `categories`, `authors`, `rust-version`, `[features]`.

---

## Dipendenze Phantom / Inutilizzate

| Crate sorgente | Dipendenza dichiarata | Uso reale |
|----------------|----------------------|-----------|
| semantic-bridge | semantic-graph | **INUTILIZZATA** — nessun import, nessun riferimento |
| semantic-graph | semantic-colbert | **INUTILIZZATA** — nessun import nel codice produttivo |
| semantic-graph | semantic-gate | **INUTILIZZATA** — nessun import nel codice produttivo |
| semantic-quantum | semantic-walk | Usata solo in `#[cfg(test)]` e doc-link |

**Impatto:** tempi di compilazione gonfiati, DAG delle dipendenze più complesso del necessario, confusione per il lettore.

**Fix:** rimuovere le dipendenze inutilizzate dai Cargo.toml; spostare semantic-walk a `[dev-dependencies]` in quantum.

---

## Feature Flag — Opportunità Mancate

### semantic-colbert/ffi

Il modulo FFI contiene codice `unsafe` e dipende da una libreria esterna (`crisp-embed`). Dovrebbe essere gated:

```toml
[features]
default = []
ffi = []  # abilita il modulo FFI con crisp-embed
```

### semantic-graph/zonizzazione

Il modulo zonizzazione tira in `serde` + `serde_json` — uniche dipendenze runtime esterne del workspace. Dovrebbe essere opzionale:

```toml
[features]
default = []
zonizzazione = ["dep:serde", "dep:serde_json"]
```

---

## .gitignore

### Problemi attuali

- **Duplicato:** la riga `target/` appare due volte (artefatto di merge)
- **File ridondante:** `semantic-colbert/.gitignore` contiene solo `/target` (già coperto dal root)

### Patterns mancanti

```gitignore
# Python
__pycache__/
*.pyc
*.pyo
.venv/
venv/

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db
desktop.ini

# Rust
**/*.rs.bk
*.pdb

# Profiling
perf.data*
flamegraph.svg

# Environment
.env
.env.*
```

---

## Audit Licensing

### Stato attuale (incoerente)

| Fonte | Licenza dichiarata |
|-------|-------------------|
| `Cargo.toml` workspace | MIT OR Apache-2.0 |
| `.zenodo.json` | Apache-2.0 |
| `ZENODO.md` | MIT / Apache-2.0 |
| `LICENSE-MIT` | MIT (copyright: Camillo Almadori + Iris) |
| `LICENSE-APACHE` | Apache-2.0 (copyright: **[yyyy] [name]** — NON COMPILATO) |
| Decisione documentata (Fase 4) | Drop MIT → Apache-2.0 only |
| `cargo metadata` (3 crate) | `license: null` |

### Problemi specifici

1. **LICENSE-APACHE riga 189:** il campo copyright è ancora il boilerplate `[yyyy] [name of copyright owner]`
2. **LICENSE-MIT:** nomina "Iris" come co-holder — ma Iris non ha soggettività giuridica (documentato in `comunicazioni/20260916_piano_pubblicazione_step.md`)
3. **Decisione non applicata:** la Fase 4 del piano di pubblicazione dice "solo Apache-2.0" ma il workspace dichiara ancora dual licensing
4. **Nessun file NOTICE** (richiesto da Apache-2.0 per redistribuzione)
5. **Nessun header SPDX** nei sorgenti (opzionale ma best practice)
6. **3 crate senza license** in metadata — non pubblicabili su crates.io

### Azione raccomandata

Decidere una volta per tutte (Apache-2.0 only come da Fase 4), poi:
1. Compilare copyright in LICENSE-APACHE
2. Rimuovere LICENSE-MIT (o mantenere se dual license è la scelta finale)
3. Aggiornare `.zenodo.json`, `ZENODO.md`, `Cargo.toml` workspace
4. Aggiungere `license = "Apache-2.0"` ai 3 crate che lo omettono
5. Creare file NOTICE
6. (Opzionale) Aggiungere SPDX headers

---

## .zenodo.json — Problemi

```json
{
  "title": "semantic-walk: v0.1.1 — First Immutable Snapshot...",
  "upload_type": "software",
  "description": "First immutable research snapshot...",
  "creators": [...],
  "access_right": "open",
  "license": "Apache-2.0"
}
```

| Campo | Stato | Problema |
|-------|-------|----------|
| `title` | ⚠ | In conflitto con il title in ZENODO.md |
| `version` | ✗ | **Assente** (dice v0.1.1 nel title, workspace è 0.1.0) |
| `publication_date` | ✗ | **Assente** |
| `keywords` | ✗ | **Assente** (necessario per discoverability) |
| `related_identifiers` | ✗ | **Assente** (necessario per link paper↔code DOI) |
| `creators[].orcid` | ✗ | **Assente** |
| `description` | ⚠ | Troppo breve per norme Zenodo (minimo ~100 parole raccomandate) |
| `license` | ⚠ | Apache-2.0 only vs workspace dual |

---

## README.md — Problemi

Il README è scritto come monologo di Iris (forte filosoficamente, debole come documentazione developer-facing):

1. **Dice "3 crates"** — sono 7
2. **Numeri errati:** 149 fatti / 24324 righe (reali: 153 / 24926)
3. **Nessuna istruzione di build** — come si compila?
4. **Nessun MSRV dichiarato**
5. **Nessun esempio d'uso** — come si usa il workspace?
6. **Nessuna sezione license**
7. **Path environment-specific** (`/home/iris/...`) — non portabile
8. **Non aggiornato dal 10 settembre** — le comunicazioni arrivano al 17
9. **`comunicazioni/` non linkato** — invisibile al lettore
10. **Nessun badge** (build status, license, crates.io version)

**Raccomandazione:** mantenere la voce di Iris come introduzione filosofica, ma aggiungere sezioni tecniche standard (Build, Usage, Architecture, Contributing, License).

---

## GUIDA_ACCESSO_CAMILLO.md — Sicurezza

Il file espone in un repository che sarà pubblico:

| Dato | Rischio |
|------|---------|
| Hostname: `uncino.eu` | DNS enumeration |
| Porta SSH: `2222` | Target scanning |
| Username: `camillo` | Credential stuffing |
| Path filesystem: `/home/camillo/...` | Directory traversal |
| OS: CachyOS (Arch-based) | Exploit specifici |
| GPU device paths | Hardware fingerprinting |
| Sudo rights documentati | Privilege escalation path |

**Impatto:** abbassa materialmente il costo di reconnaissance per un attaccante.

**Mancanze:**
- Nessuna procedura di key rotation / offboarding
- Nessun accenno a 2FA
- Nessuna menzione di fail2ban o rate limiting
- Si ferma a "hai una shell" — nessuna istruzione per toolchain Rust o build

**Raccomandazione:** rimuovere dal repository pubblico, spostare in un vault privato (1Password, Bitwarden) o in un file `.env` non committato.

---

## Pipeline Python (zonizzazione/)

### CRITICAL — API Key esposta

```python
# zonizza.py:18
API_KEY = <redatta: usare variabile d'ambiente QDRANT_API_KEY>
```

**Azioni richieste:**
1. Ruotare immediatamente la chiave su Qdrant
2. Rimuovere dal file (sostituire con `os.environ['QDRANT_API_KEY']`)
3. Pulire la git history (`git filter-branch` o BFG Repo-Cleaner)
4. Verificare che la chiave non sia stata usata da terzi (log Qdrant)

### Problemi di ingegneria

| Problema | Impatto |
|----------|---------|
| Nessun `requirements.txt` o `pyproject.toml` | Dipendenze implicite (numpy, scikit-learn) — non riproducibile |
| Nessun argparse | Configurazione solo via editing del sorgente |
| Nessuna environment variable | Secret hardcodati |
| Nessun logging | Debugging impossibile senza print |
| Nessun type hint | Code review difficoltosa |
| Nessun exception handling su network calls | Crash silenzioso se Qdrant non risponde |
| `MiniBatchKMeans` non bit-reproducibile | Cambiando versione sklearn, i centroidi cambiano anche con `random_state=0` |
| Nessun guard per centroidi a norma zero | Empty cluster → divisione per zero → NaN centroidi |
| `data_generazione` timestamp nell'output | Byte diversi a ogni run — non riproducibile |
| Nessuna validazione dimensione righe | Righe con dim diversa → crash numpy o risultati errati |

---

## Artefatti Dati

### dizionario_celle_v1.0.json (5.67 MB)

- Schema parzialmente deserializzato dal Rust (solo `centroidi`, `dim`, `k`)
- Campi scartati: `n_fatti`, `n_righe`, `inertia_norm`, `data_generazione`
- Impossibile validare provenienza/consistenza con traiettorie
- 256 centroidi × 1024 dimensioni × ~22 byte/float = ~5.5 MB (coerente)

### traiettorie_v1.0.json (704 KB)

- 153 fatti con traiettorie di celle
- Nessuna assertion di consistenza lunghezze (celle vs pos vs conf devono avere stessa lunghezza per ogni fatto)
- Il loader Rust non valida — accetta anche dati incoerenti

### Raccomandazione archiviazione

6.4 MB di JSON nel repository sono accettabili per Git ma problematici per Zenodo (che preferisce archive stabili). Opzioni:
- Git LFS per i JSON
- DOI separato per i dati (Zenodo dataset)
- Compressione (xz → ~1.5 MB)

---

## Infrastruttura di Test

### Stato attuale

| Strumento | Presente? | Note |
|-----------|:---------:|------|
| CI pipeline | ✗ | Nessun `.github/workflows/`, `.gitlab-ci.yml` |
| Coverage tooling | ✗ | Nessun cargo-llvm-cov, tarpaulin |
| Fuzzing | ✗ | Nessun cargo-fuzz |
| Benchmark (Criterion) | ✗ | Solo `bench_pareto` hand-rolled (e rotto) |
| Test report machine-readable | ✗ | Nonostante claim di "128 test, 0 falliti" |
| Integration test cross-crate | ⚠ | Solo semantic-walk e semantic-gate hanno test di integrazione |

### False-green

`semantic-graph/tests/test_zonizzazione_reale.rs`:
```rust
// Se i file JSON non sono presenti, il test ritorna senza assert
// → passa silenziosamente anche senza dati
let dizionario = match std::fs::File::open(&path_diz) {
    Ok(f) => f,
    Err(_) => return, // ← FALSE GREEN
};
```

**Fix:** usare `#[ignore]` con motivo esplicito, oppure `panic!("File richiesto assente: {}", path)`.

---

## Gap di Integrazione Cross-Crate

### Crate orfani

`semantic-bridge` e `semantic-quantum` sono **orfani** — nessun altro crate nel workspace dipende da loro. Sono foglie del DAG senza consumatori.

### zonizzazione → bridge: adapter mancante

- `TraiettoriaFatto` (dal JSON) non ha campo `colbert`
- `Walk::coerente()` richiede che il vettore colbert sia presente e della lunghezza giusta
- **Nessun Walk può essere costruito dai dati committati** senza un adapter di flattening che non esiste

### bridge → DTW: formato incompatibile

- `proiezione()` ritorna `&[f64]` flat (n×d elementi contigui)
- `KinematicAligner::align()` prende `&[T: AsRef<[f64]>]` (slice di slice)
- Manca un adapter che reshape il flat buffer in slice-per-punto

### graph/quantum → pipeline reale

- `gate_crisp_alignment` usa il gate direttamente, non il quantum resolver
- Il quantum resolver non ha nessun chiamante nel codice produttivo
- I quattro pesi diversi (combiner, gate, graph, quantum) non hanno single source of truth

### Drift di tipo numerico

| Crate | Tipo principale |
|-------|----------------|
| semantic-combiner | f64 |
| semantic-gate | f64 |
| semantic-graph | f64 |
| semantic-walk | f64 |
| semantic-colbert | f64 (input), f64 (output) |
| semantic-quantum | **f32** (BranchCostVector, amplitude) |
| semantic-bridge | **f32** (centroidi, conf) → f64 (output proiezione) |

Il widening f32→f64 avviene nella proiezione; il narrowing f64→f32 avviene nel quantum builder. Nessuna documentazione della precisione persa/guadagnata.

---

## Checklist Publication Readiness

| # | Item | Stato | Note |
|---|------|:-----:|------|
| 1 | API key rimossa dal codice | ✗ | `zonizza.py:18` |
| 2 | API key ruotata su Qdrant | ✗ | — |
| 3 | Git history pulita dal secret | ✗ | — |
| 4 | LICENSE-APACHE copyright compilato | ✗ | Riga 189 boilerplate |
| 5 | Licenza coerente su tutti i file | ✗ | 5 fonti diverse |
| 6 | File NOTICE presente | ✗ | Richiesto da Apache-2.0 |
| 7 | `.zenodo.json` completo | ✗ | Mancano 5 campi |
| 8 | `.zenodo.json` versione coerente | ✗ | v0.1.1 vs 0.1.0 |
| 9 | README.md aggiornato | ✗ | "3 crates", numeri errati |
| 10 | README.md con build instructions | ✗ | — |
| 11 | MSRV dichiarato | ✗ | — |
| 12 | GUIDA_ACCESSO rimossa dal pubblico | ✗ | Dati sensibili |
| 13 | bench_pareto funzionante | ✗ | Generatore degenerato |
| 14 | Test false-green fixati | ✗ | `test_zonizzazione_reale.rs` |
| 15 | NaN bugs fixati (S1) | ✗ | 5 finding separati |
| 16 | Dipendenze phantom rimosse | ✗ | 3 inutilizzate |
| 17 | License field in tutti i crate | ✗ | 3 omessi |
| 18 | CI pipeline presente | ✗ | — |
| 19 | `requirements.txt` per Python | ✗ | — |
| 20 | ORCID per creators | ✗ | — |
| 21 | Build offline verificata | ✓ | — |
| 22 | Cargo.lock committato | ✓ | — |
| 23 | Test passano (subset) | ✓ | 56/56 |
| 24 | Documentazione interna (comunicazioni) | ✓ | 8 documenti |
| 25 | ZENODO.md con formule corrette | ✓ | — |

**Score: 5/25 pronti — 20 item da completare prima della pubblicazione.**

---

[Torna al sommario](./00_sommario_esecutivo.md)
