# Schema a Blocchi — Architettura semantic-geo (versione corretta e allineata al codice)

**Data**: 23/09/26 23:32
**Autori**: Camillo (schema originale + correzioni), Iris (revisione contro codice reale)

## Nota di revisione

Lo schema originale conteneva tre imprecisioni rispetto al codice reale, corrette dopo verifica riga per riga sui sorgenti:

1. **parse.rs (Terza Via)** — formula invertita: l'azzeramento colpisce i token di disturbo, non quelli significativi.
2. **probe3.rs** — descrizione inventata (nessuna derivata seconda; è un benchmark LCG della frontiera di Pareto).
3. **combiner** — firma reale `combine(axes, weights)` con invariante di coerenza Pareto, non semplice somma lineare.

---

## Schema a blocchi

```text
[ Server CrispEmbed / Qdrant ]
       │
       ▼
1. INGEST & FFI  ──>  semantic-bridge / semantic-colbert
       │              • Ricezione payload dal server
       │              • MaxSim Multi-Vector: S(q,d) = ∑_i max_j (E_i · D_jᵀ)
       │
       ▼
2. PARSING & DTW ──>  semantic-walk
       │              • parse.rs: w_i = w_i_orig se st==0 ∧ id≥4, altrimenti 0.0
       │                (Igiene su token di disturbo senza rompere la biiezione 0..N-1)
       │              • dtw.rs: D(i,j) = d(x_i, y_j) + min(D_{i-1,j}, D_{i,j-1}, D_{i-1,j-1})
       │
       ▼
3. PARETO ENGINE ──>  semantic-quantum
       │              • pareto.rs: Frontiera di dominanza u ≻ v
       │              • probe3.rs: Benchmark LCG ed estrazione frontiera candidato
       │
       ▼
4. VALIDAZIONE   ──>  semantic-gate
       │              • budget.rs / sonda.rs: Controllo risorse
       │              • Gate(x) = 𝕀(Cost(x) ≤ B ∧ Verdict != Timeout)
       │
       ▼
5. SINTESI       ──>  semantic-combiner & semantic-graph
                      • combine(axes: &NormalizedAxes, weights: [f64; 3])
                        (Invariante di coerenza Pareto sugli assi normalizzati)
                      • Potatura su grafo: P_k(W) = Percentile_k({w_ij})
```

---

## Tabella dei Crate e Formalizzazione Reale

| Crate | Componente | Operazione Chiave | Formulata / Firma Reale |
| --- | --- | --- | --- |
| **`semantic-bridge`** / **`semantic-colbert`** | `proiezione.rs`, `ffi.rs` | Proiezione densa e MaxSim su embeddings ColBERT | $S(q, d) = \sum_{i \in q} \max_{j \in d} (E_i \cdot D_j^\top)$ |
| **`semantic-walk`** | `parse.rs` | Terza Via: mantiene la biiezione $0..N-1$ azzerando solo i token di disturbo | $w_i = \begin{cases} w_i^{\text{orig}} & \text{se } st=0 \land id \ge 4 \\ 0.0 & \text{altrimenti} \end{cases}$ |
| **`semantic-walk`** | `dtw.rs` | Allineamento temporale non lineare di sequenze | $D(i, j) = d(x_i, y_j) + \min(D_{i-1,j}, D_{i,j-1}, D_{i-1,j-1})$ |
| **`semantic-quantum`** | `pareto.rs` | Estrazione vettori non dominati nello spazio delle metriche | $u \succ v \iff (\forall m \, u_m \ge v_m) \land (\exists m \, u_m > v_m)$ |
| **`semantic-quantum`** | `probe3.rs` | Benchmark empirico e stress test della frontiera di Pareto | Generazione LCG $\rightarrow$ `estrai_frontiera_pareto_sui_candidati` |
| **`semantic-gate`** | `sonda.rs`, `budget.rs` | Interruzione per budget o timeout | $\text{Gate}(x) = \mathbb{I}(\text{Cost}(x) \le B \land \text{Verdict} \neq \text{Timeout})$ |
| **`semantic-graph`** | `zonizzazione.rs` | Potatura archi topologici su base percentile | $P_k(\mathbf{W}) = \text{Percentile}_k(\{w_{ij}\})$ |
| **`semantic-combiner`** | `lib.rs` | Aggregazione pesata su assi normalizzati con vincolo Pareto | `combine(axes: &NormalizedAxes, weights: [f64; 3])` |

---

## Stato

Baseline architetturale allineata al codice del repository. Prossimo passo: test di integrazione con il server (frames mode deployato su .18/.5) e completamento del Level 3 end-to-end.
