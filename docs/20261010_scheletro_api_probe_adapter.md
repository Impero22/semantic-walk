# Scheletro API pubbliche `ProbeAdapter` — bozza di contratto (10/10/26)

> **Stato:** BOZZA — da revisionare da Camillo (richiesta 10/10 01:26).
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

Il modulo `probe_adapter.rs` esiste già in forma funzionante (Via B,
`4523eb8`). Questa bozza ne fissa il **contratto pubblico**: cosa deve
esporre, cosa resta interno, quali garanzie fornisce.

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

## 3. Contratto pubblico proposto

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

// Il selettore è esso stesso un GraphAdapter: il solver lo consuma senza
// distinzione rispetto a un adapter singolo.
impl GraphAdapter for ProbeAdapter {
    fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)>;
}
```

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

## 4. Domande aperte per la revisione

1. **`ProbeKind` pubblico o interno?** Oggi `ProbeKind` vive in `solver.rs`
   come tipo. Va esposto a livello di crate (`pub use`), o è un dettaglio che
   il consumatore non deve conoscere? *(Inclinazione: esporlo — il benchmark
   deve poter etichettare i cammini per sonda.)*
2. **Naming**: `Curvature`/`Relational` vs `Frame`/`Proximity`. Il primo
   descrive la *dimensione misurata* (semantica), il secondo l'*adapter*
   (implementazione). Per il contratto pubblico è più stabile il primo.
3. **Costruttori vs `From`**: `curvature()`/`relational()` sono sufficienti,
   o servono `From<FrameAdapter>` / `From<ProximityAdapter>` per composizione
   da adapter già costruiti?
4. **Serializzazione**: serve `Serialize`/`Deserialize` per passare la sonda
   oltre confine di processo (ingresso framework)? O la selezione resta a
   livello di codice?

## 5. Criteri di accettazione (per la forma definitiva)

- [ ] `GraphAdapter` resta minimale (solo `neighbors()`) — nessuna regressione.
- [ ] `ProbeAdapter` è l'unico selettore di strategia pubblico.
- [ ] Test: `kind_dichiara_la_sonda`, `entrambe_le_strategie_espongono_vicini`
      (già presenti) + un test che confronta i cammini delle due sonde su un
      grafo sintetico e ne verifica l'ortogonalità dichiarata.
- [ ] Workspace verde (semantic_walk 103 + altri crate).

## 6. Relazioni

- **Base**: commit `4523eb8` (Via B consolidata).
- **A monte**: docs/20261008_collasso_frame_adapter.md (Strada 1 —
  Complementarità).
- **A valle**: decisione guardiano Strato 1 (Jaccard globale) e allineamento
  Sezione 7.2.3 + τ_div nel paper, una volta stabilizzate le API.
