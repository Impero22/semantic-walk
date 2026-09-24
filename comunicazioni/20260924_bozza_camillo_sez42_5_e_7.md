# Bozza Camillo — Sezioni 4.2 (Banda Adattiva), 5 (Memoria & Roadmap) e 7 (Ablation)

**Autore**: Camillo
**Data**: 24 Settembre 2026
**Scopo**: Integrazione formale dei blocchi di competenza da allineare con le bozze di Iris (`20260924_bozza_iris_sez41_e_6.md`).
**Fedeltà al codice**: Ogni formula è verificata su `semantic-walk/src/dtw.rs`.

---

## Sezione 4.2 — Banda di Sakoe-Chiba Adattiva al Jaccard Posizionale (Formulazione a Rampa)

Per mitigare la complessità $O(NM)$ senza compromettere l'allineamento di deformazioni strutturali, l'ampiezza della finestra di vincolo $r_i$ non è statica ma viene modulata dinamicamente al passo $i$ in base alla concordanza posizionale locale delle guide *ordered-sparse*.

### 4.2.1 Formulazione della Rampa Lineare a Tratti

Sia $J_i = J_{\text{pos}}(X, Y, i) \in [0, 1]$ l'indice di Jaccard posizionale calcolato sul frame $i$-esimo tra le sequenze sparse $X$ e $Y$. La banda ammissibile $r_i$ è governata da una funzione a tratti con interpolazione lineare continua tra le soglie $0.3$ e $0.7$:

$$r_i = \min \left( w_{\text{base}}, \; \begin{cases}  w_{\text{min}} & \text{se } J_i \ge 0.7 \\  w_{\text{max}} & \text{se } J_i < 0.3 \\  \left\lfloor w_{\text{min}} + (w_{\text{max}} - w_{\text{min}}) \cdot \frac{J_i - 0.3}{0.4} \right\rfloor & \text{se } 0.3 \le J_i < 0.7  \end{cases} \right)$$

dove $w_{\text{min}}$ e $w_{\text{max}}$ sono i limiti di vincolo cinematico ($w_{\text{min}} \le w_{\text{max}}$) e $w_{\text{base}}$ è il limite massimo globale imposto dall'istanza del sistema (`window_size`).

### 4.2.2 Modulazione e Penalizzazione del Costo Locale

L'intervallo ammissibile degli indici $j$ nella matrice per la riga $i$ è definito da:

$$\text{window\_start}(i) = \max\left(1, \; i - r_i\right), \qquad \text{window\_end}(i) = \min\left(M, \; i + r_i\right)$$

Qualora la concordanza posizionale sia criticamente bassa ($J_i < 0.3$), il costo locale $C(i, j)$ calcolato tramite distanza coseno densa (`cosine_distance`) viene ponderato da un fattore di penalità additivo sulla distanza, proporzionale alla divergenza:

$$C_{\text{effettivo}}(i, j) = C(i, j) \cdot \left(1.0 + (0.3 - J_i)\right) \qquad \forall J_i < 0.3$$

In questo modo, traiettorie con scarsa concordanza posizionale subiscono sia una dilatazione della banda (fino a $w_{\text{max}}$), sia una penalizzazione sul costo di allineamento, scoraggiando scorciatoie non topologiche nel cammino ottimo.

---

## Sezione 5 — Architettura di Memoria: Stato Attuale e Zero-Allocation Roadmap

L'efficienza del ciclo di query richiede un'analisi rigorosa dell'impronta di memoria nel percorso critico di matching.

### 5.1 Stato Attuale dell'Implementazione (`dtw.rs`)

L'attuale allocazione in `dtw.rs` gestisce la matrice delle distanze mediante allocazione dinamica su heap $N \times M$ (`vec![vec![f64::INFINITY; m + 1]; n + 1]`), accompagnata da un vettore dinamicamente ridimensionato per la ricostruzione del cammino ottimo $W^*$ (`warp_path`). Questa struttura garantisce chiarezza nella fase di prototipazione ma introduce chiamate al sistema di memoria durante l'esecuzione delle query.

### 5.2 Optimization Roadmap verso il Zero-Allocation Runtime

Per garantire latenze deterministiche in contesti produttivi ad alta frequenza, l'architettura formalizza la transizione al modello *Zero-Allocation* sul percorso critico ($0$ chiamate ad heap durante la fase di matching):

1. **Circular Buffer per la Programmazione Dinamica**: Poiché l'equazione di Bellman al passo $i$ richiede esclusivamente i valori della riga corrente $i$ e della riga precedente $i-1$, la matrice $N \times M$ viene sostituita da due buffer circolari di dimensione fissa limitata dalla banda massima $2 \times (2 w_{\text{max}} + 1)$ elementi `f32`, allocati direttamente nello stack frame della funzione.
2. **ThreadLocal ScratchPad per il Backtracking**: Qualora sia richiesta l'estrazione esplicita del cammino di warping $W^*$, i vettori temporanei vengono gestiti tramite una struttura `ScratchPad` pre-allocata all'inizializzazione del thread worker (`ThreadLocal`), azzerando l'overhead di `malloc`/`realloc` nel ciclo di query.

---

## Sezione 7 — Matrice di Ablation Study a 5 Livelli

Per valutare quantitativamente il contributo di ogni singolo modulo, il benchmark di validazione è articolato su 5 configurazioni incrementali:

| Livello | Configurazione Pipeline | Componenti Attivi | Metrica Target di Valutazione |
| :--- | :--- | :--- | :--- |
| **L1** | *Baseline ColBERT* | MaxSim bag-of-vectors densa standard | Bounding qualitativo senza vincoli topologici d'ordine |
| **L2** | *DTW Naive* | DTW $D$-dimensionale denso ($W = \infty$) | Impatto dell'allineamento d'ordine non vincolato |
| **L3** | *DTW Geometrizzato* | DTW + Banda $r_i$ Sakoe-Chiba Adattiva al Jaccard | Efficienza della banda dinamica e riduzione rumore |
| **L4** | *Full DTW Pipeline* | DTW Geometrizzato + Early Termination Pareto | Tasso di pruning e riduzione della latenza a candidato |
| **L5** | *Full Semantic-Walk* | Pipeline completa + Gate Permissivo ($\tau_{\text{div}}$ + Timeout) | Risparmio complessivo di throughput con garanzia $P(\text{FN}) \le \epsilon$ |
