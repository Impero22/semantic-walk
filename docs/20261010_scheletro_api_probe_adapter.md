# Contratto pubblico `ProbeAdapter` — forma definitiva (10/10/26)

> **Stato:** DEFINITIVO — decisioni di Camillo recepite (10/10 01:31).
> **Base:** Via B consolidata su `4523eb8` (GraphAdapter minimale, ProbeAdapter
> come unico selettore di strategia).
> **Contesto:** docs/20261008_collasso_frame_adapter.md, "Aperto: scheletro API
> pubbliche ProbeAdapter (stende Camillo)".

---

## 1. Scopo

Definire la **forma definitiva del contratto pubblico** delle sonde semantiche:
l'API che il resto del sistema (solver, benchmark, futuro ingresso nel
framework) consumerà per selezionare una delle due dimensioni di osservazione
del grafo — geometria locale (`Curvature`) o struttura relazionale
(`Relational`).

Il modulo `probe_adapter.rs` esiste in forma funzionante e il contratto qui
fissato è quello implementato. Questo documento è il punto di verità del
contratto: cosa espone, cosa resta interno, quali garanzie fornisce.

## 2. Principi guida

1. **GraphAdapter minimale.** Il trait consumato dal solver espone solo
   `neighbors()`. Non conosce metadati di calibrazione, non sa quale
   dimensione della semantica sta misurando. (Lezione Via A, scartata 10/10.)
2. **La sonda è un selettore.** `ProbeAdapter` è l'unico punto di selezione
   della strategia. La complementarità è dichiarata dal tipo, non dispersa
   nel codice di consumo.
3. **Componibilità.** Si può costruire un `ProbeAdapter` per ogni strategia e
   confrontarne i cammini a runtime senza cambiare il trait consumato.
4. **Ortogonalità documentata.** Jaccard 0.0 e Spearman 0.0 tra i cammini
   delle due sonde (Dataset B) sono *evidenza di indipendenza*, non anomalia.

## 3. Contratto pubblico (definitivo)

```rust
/// Le sonde semantiche come strategie esplicite.
pub enum ProbeAdapter {
    /// Sonda di curvatura: geometria locale del frame (`FrameAdapter`).
    Curvature(FrameAdapter),
    /// Sonda relazionale: struttura del grafo (`ProximityAdapter`).
    Relational(ProximityAdapter),
}

impl ProbeAdapter {
    /// Costruisce una sonda di curvatura da grafo e traiettorie dense.
    pub fn curvature(grafo: Graph, traiettorie: HashMap<FactId, Vec<Vec<f64>>>) -> Self;

    /// Costruisce una sonda relazionale dal grafo.
    pub fn relational(grafo: Graph) -> Self;

    /// La natura della sonda selezionata (Curvature | Relational).
    pub fn kind(&self) -> ProbeKind;
}

// Composizione da adapter già istanziati a monte.
impl From<FrameAdapter> for ProbeAdapter;
impl From<ProximityAdapter> for ProbeAdapter;

// Il selettore è esso stesso un GraphAdapter: il solver lo consuma senza
// distinzione rispetto a un adapter singolo.
impl GraphAdapter for ProbeAdapter {
    fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)>;
}
```

### Decisioni di contratto (Camillo, 10/10 01:31)

1. **`ProbeKind` pubblico.** Re-esportato a livello di crate con
   `pub use solver::ProbeKind;`. Il benchmark e i consumatori devono poter
   etichettare e filtrare i cammini in base alla sonda utilizzata senza
   accedere alla struttura interna dei moduli.
2. **Naming `Curvature`/`Relational`.** Confermato: nel contratto pubblico si
   descrive la *dimensione semantica misurata*, non la struttura dati
   sottostante (`Frame`/`Proximity`).
3. **Costruttori + `From`.** Si mantengono i costruttori espliciti
   `curvature()`/`relational()`, e si aggiungono `From<FrameAdapter>` e
   `From<ProximityAdapter>` per massima compositività quando gli adapter sono
   già istanziati a monte.
4. **Nessuna serializzazione.** Non si implementa `Serialize`/`Deserialize`
   su `ProbeAdapter`: le traiettorie e la struttura del grafo gestiscono
   buffer di memoria significativi; la selezione della strategia resta a
   livello di codice nell'ambiente di esecuzione.

### Garanzie del contratto

- **Adiacenza preservata**: `neighbors()` restituisce sempre gli stessi vicini
  del grafo sottostante; cambia solo lo *stato cinematico* attribuito a
  ciascun vicino (dalla sonda scelta).
- **Nodi senza dati**: un nodo senza traiettoria (Curvature) o senza archi
  (Relational) produce `KinematicState::default()` (zero), mai errore o
  panico. Il ritiro è geometrico (giudizio), non un guasto.
- **Determinismo**: a parità di grafo e di dati, `neighbors()` è
  deterministico. L'ordine dei vicini è lasciato all'implementazione (non
  significativo per la correttezza del beam).

## 4. Implementazione

Integrato su base `b018516` (bozza) → `[commit finale]`:

- `lib.rs`: aggiunto `pub use solver::ProbeKind;`.
- `probe_adapter.rs`: aggiunte `impl From<FrameAdapter>` e
  `impl From<ProximityAdapter>`.
- Test: `kind_dichiara_la_sonda`, `entrambe_le_strategie_espongono_vicini`,
  `from_adapter_compone_le_strategie` (nuovo) — tutti verdi.

## 5. Criteri di accettazione (soddisfatti)

- [x] `GraphAdapter` resta minimale (solo `neighbors()`) — nessuna regressione.
- [x] `ProbeAdapter` è l'unico selettore di strategia pubblico.
- [x] Test: `kind_dichiara_la_sonda`, `entrambe_le_strategie_espongono_vicini`,
      `from_adapter_compone_le_strategie` — tutti verdi.
- [x] Workspace verde (semantic_walk 104 + altri crate).

## 6. Relazioni

- **Base**: commit `4523eb8` (Via B consolidata).
- **A monte**: docs/20261008_collasso_frame_adapter.md (Strada 1 —
  Complementarità).
- **A valle**: decisione guardiano Strato 1 (Jaccard globale) e allineamento
  Sezione 7.2.3 + τ_div nel paper, una volta stabilizzate le API.
