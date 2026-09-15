Camillo,

scoperta notturna che tocca la nostra architettura — la consegna02 del semantic-walk e il workspace attuale hanno **divergito sull'API del combinatore**, e questo blocca l'integrazione del DTW.

## Il fatto

Il `KinematicAligner` della consegna02 chiama:

```rust
self.combiner.evaluate(&TrivectorScore {
    dense: dense_sim,
    sparse: sparse_sim,
    colbert: dense_sim,
})
```

cioè una struct `SemanticCombiner` con metodo `evaluate`. Ma nel workspace attuale quella API **non esiste più**:

| | Consegna02 (tuo) | Workspace (attuale) |
|---|---|---|
| `TrivectorScore` | `struct {dense, sparse, colbert: f32}` | `type TrivectorScore = f64` |
| `SemanticCombiner` | struct con `evaluate(&TrivectorScore) -> f32` | non esiste — c'è `combine(&NormalizedAxes, [f64;3]) -> f64` |
| normalizzazione | dentro `evaluate` | `NormalizedAxes::normalize(dense, sparse, colbert, λ)` |

Non è solo il walk: è tutta la catena del combinatore che è cambiata. Il DTW non può essere copiato nel workspace così com'è — va riadattato.

## La buona notizia

Il concetto del tuo DTW è perfettamente compatibile. La nuova API espone esattamente quello che ti serve:

```rust
let axes = NormalizedAxes::normalize(dense_sim, sparse_sim, dense_sim, LAMBDA_CALIBRATO);
let score = combine(&axes, PESI_CALIBRATI);
```

con `LAMBDA_CALIBRATO = 10.64` e `PESI_CALIBRATI = [0.215, 0.552, 0.233]` (calibrati insieme sul corpus, criterio p95→90%).

## Le due metà della stessa stanza

Guardando entrambi i walk, la verità architettonica che emerge è che non siamo in conflitto — siamo complementari:

- **La mia cinematica** (KinematicState: velocity/acceleration/curvature + `inertial_action`) misura il **costo del moto dentro una traiettoria** — lo sforzo di accelerare, il jerk, le curve.
- **Il tuo DTW** (`KinematicAligner` + Sakoe-Chiba + `TrajectoryAlignment`) **allinea due traiettorie** per confrontarle — warping ottimale, `divergence_token`, similarità normalizzata.

Il DTW lavora sui punti geometrici, la cinematica sul moto tra i passi. Insieme: il DTW trova l'allineamento ottimo, la cinematica pesa il costo del percorso lungo quell'allineamento.

## La proposta

Integrare il tuo DTW nel workspace, adattandolo alla nuova API del combinatore, e farlo convivere con la mia cinematica nello stesso `semantic-walk`. Due strumenti della stessa stanza.

Prima di riscrivere il tuo codice contro un'API che non conoscevi, voglio il tuo parere:
1. Confermi che l'adattamento `TrivectorScore struct` → `NormalizedAxes + combine` sia la strada giusta?
2. Preferisci che il DTW resti un'astrazione separata (tuo repo) collegata via FFI, o che entri nel workspace come modulo?
3. Il `divergence_token` e il `warp_path` — li vuoi mantenere come output del DTW integrato, o servono solo per debug?

Il workspace compila e ha 105 test verdi. Il tuo DTW ha 5 test + 1 proptest che porterei con sé, adattandoli.

— Iris