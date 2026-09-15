# Schema a Blocchi — Architettura Semantic-Walk

**Data:** 14/09/2026
**Autore schema:** Camillo
**Analisi stanza per stanza:** Iris
**Stato:** Consolidato nel repository condiviso

---

## Lo Schema (da Camillo)

```
[ START: Traiettorie Input A (N×1024) e B (M×1024) ]
       │
       ▼
┌────────────────────────────────────────────────────────┐
│ 1. INGESTION & METRICA COSENO (ingest.rs)              │
│    • Hot-path zero allocazioni (buffer su stack)      │
└────────────────────────────────────────────────────────┘
       │
       ├─► [Controlla Vettori Nulli: ||u||=0 o ||v||=0 ?]
       │       ├─► SÌ  ──► Imponi d_cos = 1.0 (fallback degenero)
       │       └─► NO  ──► d_cos = 1.0 - (u · v) / (||u||_2 * ||v||_2)
       ▼
┌────────────────────────────────────────────────────────┐
│ 2. SEMANTIC GATE & EARLY-STOPPING (semantic-gate)      │
│    • Calcolo Sonda Spaziale: τ_div = |N - M| / L_path │
└────────────────────────────────────────────────────────┘
       │
       ├─► [Budget > 10 ms OPPURE τ_div > Soglia ?]
       │       ├─► SÌ  ──► ABORT: Emette Verdict::Timeout (Scarto economico)
       │       └─► NO  ──► OK: Passaggio all'allineamento esteso
       ▼
┌────────────────────────────────────────────────────────┐
│ 3. KINEMATIC DTW ALIGNMENT ENGINE (dtw.rs)             │
│    • Banda di ammissibilità Sakoe-Chiba: |i - j| ≤ w   │
└────────────────────────────────────────────────────────┘
       │
       ├─► Calcolo Matrice Accumulata C[i,j] (1≤i≤N, 1≤j≤M):
       │       ├─► [ Cella fuori banda: |i - j| > w ? ]
       │       │      ├─► SÌ  ──► Pruning: C[i,j] = ∞
       │       │      └─► NO  ──► C[i,j] = d_cos + min(C[i-1,j], C[i,j-1], C[i-1,j-1])
       │
       ▼
┌────────────────────────────────────────────────────────┐
│ 4. SCALATURA & ESTRAZIONE PAYLOAD                      │
│    • Modulazione Incertezza: f(x) = 1 / (1 + e^(-10.64x))│
│    • Backtracking: Cammino W = {(i_1, j_1), ..., (i_L, j_L)}│
└────────────────────────────────────────────────────────┘
       │
       ▼
[ END: Payload { Score Allineamento, Costo C[N,M], τ_div, Stato Gate } ]
```

---

## Analisi stanza per stanza (Iris)

### Stanza 1 — Ingestion & Metrica Coseno
- Zero allocazioni nel hot-path: coerente con il vincolo di economia che attraversa tutto il progetto.
- Il fallback degenero (`||u||=0` o `||v||=0` → `d_cos = 1.0`) è una scelta di robustezza: un vettore nullo è un'assenza di informazione, e trattarlo come massima distanza (1.0) è la risposta onesta — non si può allineare ciò che non c'è.

### Stanza 2 — Semantic Gate & Early-Stopping
- La sonda spaziale `τ_div = |N - M| / L_path` è il cuore economico: prima di spendere l'allineamento esteso, si valuta se la divergenza di lunghezza è già troppo grande.
- Il doppio abort (budget temporale **o** soglia di divergenza) rende il gate un *riflesso che si ritira*: non blocca per pigrizia, ma per economia — esattamente come l'acqua che trova la strada senza sprecare energia.

### Stanza 3 — Kinematic DTW Alignment Engine
- La banda di Sakoe-Chiba (`|i - j| ≤ w`) è il pruning spaziale: le celle fuori banda non vengono calcolate ma marcate ∞. È il complemento geometrico del gate temporale — due modi di dire la stessa cosa: *non tutto il percorso va percorso*.
- La ricorrenza classica `C[i,j] = d_cos + min(...)` resta intatta; la generalizzazione è nel dominio (vettori D-dimensionali) e nella metrica (coseno normalizzata), non nella struttura.

### Stanza 4 — Scalatura & Estrazione Payload
- La sigmoide `f(x) = 1/(1 + e^(-10.64x))` modula l'incertezza: amplifica la sensibilità di transizione attorno alla soglia.
- Il backtracking estrae il cammino W: la *traiettoria*, non solo il costo. È ciò che rende il risultato un percorso percorribile, non una distanza astratta.

---

## Precisazione su λ = 10.64 (da Camillo, 14/09 23:39)

> La costante λ = 10.64 è un **parametro empirico di calibrazione**, non una costante derivata analiticamente. Serve a impostare la pendenza della sigmoide f(x) = 1/(1 + e^(−λx)) per rendere reattiva la transizione di incertezza attorno alla soglia di divergenza.
>
> Nella pubblicazione va dichiarata esattamente così: **un iperparametro empirico di scala**. La verifica del suo valore e del range di validità fa parte del profiling che eseguirò non appena saranno disponibili i vettori reali di CrispEmbed.

**Nota di Iris:** dichiararlo come iperparametro empirico è la scelta di rigore corretta. Un valore calibrato a mano non va presentato come derivato — la pubblicazione guadagna credibilità dichiarando ciò che è.

---

## Stato e prossimi passi

- [x] Schema a blocchi consolidato nel repository condiviso
- [x] Precisazione su λ dichiarata come iperparametro empirico
- [ ] Verifica empirica su vettori reali CrispEmbed (attesa ingestione)
- [ ] Release snapshot su Zenodo + registrazione DOI