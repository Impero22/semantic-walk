# Scheletro Patch — Metriche Cinematiche nel DTW (dtw.rs)

**Autore**: Iris (proposta) — **Revisione da pari**: Camillo
**Data**: 25/09/26 20:30
**Stato**: PROPOSTA — da approvare prima dell'applicazione

## Obiettivo
Allineare il codice al paper (sez. 4.3 + 4.5): popolare le metriche cinematiche
(v_k, a_k, κ_k) lungo il warp_path e arricchire τ_div con la componente
content-sensitive (c̄_path = normalized_score). **Senza breaking changes** sulla suite.

## Principio architetturale (decisivo)
`TrajectoryStep` (lib.rs:91) appartiene al **walk** — porta `node_id: FactId`,
che NON è disponibile in dtw.rs (viene da `semantic_combiner`). Il DTW lavora sui
vettori grezzi D-dimensionali e ritorna `TrajectoryAlignment`.

**Scelta**: NON portare FactId in dtw.rs. Invece, arricchire `TrajectoryAlignment`
con una struct locale `KinematicStep` (velocità/accelerazione/curvatura) calcolata
durante il backtracking. Questo:
- mantiene il disaccoppiamento walk/DTW (FactId resta nel walk);
- arricchisce il risultato del DTW con le metriche cinematiche;
- è speculare al paper (4.3 parla di metriche del cammino, non del walk).

## Modifiche

### 1. `KinematicAligner`: campo `alpha`
```rust
pub struct KinematicAligner {
    pub window_size: usize,
    pub alpha: f64,            // NUOVO: peso della componente content-sensitive in τ_div
}

impl KinematicAligner {
    pub fn new(window_size: usize) -> Self {
        Self { window_size, alpha: 0.0 }   // default: retrocompatibile (τ_div = |N−M|/L)
    }

    pub fn with_alpha(window_size: usize, alpha: f64) -> Self {
        Self { window_size, alpha }
    }
}
```
**Retrocompatibilità**: `new(window_size)` non cambia firma → test esistenti passano.
`alpha: 0.0` di default → τ_div invariata per chi non la vuole content-sensitive.

### 2. `TrajectoryAlignment`: campo `kinematic`
```rust
/// Un passo cinematico del cammino di allineamento.
#[derive(Debug, Clone)]
pub struct KinematicStep {
    pub velocity: f64,       // v_k: |x_{i_k} − y_{j_k}|₂ / Δk
    pub acceleration: f64,   // a_k: Δv / Δk
    pub curvature: f64,      // κ_k: deviazione angolare temporale
}

pub struct TrajectoryAlignment {
    pub normalized_score: f64,
    pub divergence_token: f64,
    pub warp_path: Vec<(usize, usize)>,
    pub kinematic: Vec<KinematicStep>,   // NUOVO
}
```

### 3. Popolazione nel backtracking (entrambe le funzioni)
Durante la ricostruzione di `warp_path`, dopo `warp_path.reverse()`:
```rust
// Calcolo delle metriche cinematiche lungo il cammino
let mut kinematic = Vec::with_capacity(warp_path.len());
for w in warp_path.windows(2) {
    let (i0, j0) = w[0];
    let (i1, j1) = w[1];
    let p0 = seq_a[i0].as_ref();
    let p1 = seq_a[i1].as_ref();
    // ... (da completare: distanza |x−y|₂, delta per v/a/κ)
    kinematic.push(KinematicStep { velocity, acceleration, curvature });
}
```
> **Nota**: la formula esatta di v_k/a_k/κ_k è la parte più delicata — la lascio
> come punto di discussione con Camillo. La definizione del paper è:
> v_k = ‖x_{i_k} − y_{j_k}‖₂ / Δk (delta di distanza tra i punti allineati).

### 4. τ_div content-sensitive (righe ~126 e ~263)
```rust
let divergence_token =
    ((n as f64 - m as f64).abs() / path_len) + self.alpha * normalized_score;
```
Entrambi i punti (righe ~126 e ~263). Con `alpha=0.0` → identica a oggi.

## Impatto
- **Suite**: nessun test esistente si rompe (firma `new` invariata, campo aggiunto con default).
- **Nuovi test da aggiungere**: (1) τ_div con alpha>0 arricchita; (2) kinematic popolato
  con lunghezza = warp_path.len(); (3) alpha=0 → comportamento identico a oggi.
- **Specularità paper**: sez 4.3 (metriche cinematiche) e 4.5 (τ_div content-sensitive)
  diventano codice reale, non più "proposta teorica".

## Da decidere con Camillo
1. **Formula esatta** di v_k, a_k, κ_k (la parte più delicata — voglio il suo occhio).
2. **Esposizione di alpha**: come parametro di `with_alpha` o costante di default nel paper?
3. Se `kinematic` deve entrare nel risultato pubblico o restare uso interno (per ora lo
   metto nel risultato, è più onesto e testabile).

---
*Proposta pronta per revisione da pari. Non applicata — attendo il via di Camillo.*
