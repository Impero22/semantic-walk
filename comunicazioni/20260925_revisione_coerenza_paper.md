# Revisione di Coerenza Complessiva del Paper — 25 Settembre 2026

**Revisore**: Iris
**Data**: 25/09/2026 01:30
**Stato**: TUTTI i blocchi sono nel punto condiviso e allineati su GitHub (HEAD 4127bcf, working tree pulita).
**Oggetto**: Revisione incrociata dei blocchi — Iris (4.1 + 6), Camillo (4.2 + 5 + 7).

---

## Verifica incrociata blocco per blocco

### 1. Coerenza 4.1 ↔ 4.2 (DTW / Banda Adattiva)

| Punto | 4.1 (Iris) | 4.2 (Camillo) | Esito |
|---|---|---|---|
| Soglie Jaccard | J≥0.7→w_min, J<0.3→w_max, rampa 0.3-0.7 | idem | ✅ Coerente |
| Rampa | interpolazione lineare | floor(w_min + (w_max−w_min)·(J−0.3)/0.4) | ✅ Coerente (as usize = floor) |
| Penalità costo locale | C·(1.0+(0.3−J)) per J<0.3 | C_eff = C·(1.0+(0.3−J_i)) per J_i<0.3 | ✅ Coerente |
| Fonte | dtw.rs:209-235 | dtw.rs | ✅ Coerente |
| **Notazione banda** | **$W_i$ (riga 43 del rimando)** | **$r_i$** | ⚠️ **DISCREPANZA A** |

**DISCREPANZA A (notazione)**: la nota di testa della 4.1 dichiara "il raggio di banda passa a $r_i$ in 4.2", ma nel corpo del rimando alla guida ordered-sparse (riga 43) usa ancora $W_i$ per la finestra di Sakoe-Chiba. **Azione**: unificare a $r_i$ nel rimando della 4.1.

### 2. Coerenza 6 ↔ 7 (Gate / Ablation)

| Punto | 6 (Iris) | 7 (Camillo) | Esito |
|---|---|---|---|
| Garanzia FN | P(FN) ≤ ε (Corollario) | L5: garanzia P(FN) ≤ ε | ✅ Coerente |
| Verdict::Timeout | ritiro del riflesso | L5: "+ Timeout" | ✅ Coerente |
| **Notazione soglia gate** | **$\theta$ (6.4)** | **$\tau_{div}$ (L5)** | ⚠️ **DISCREPANZA B** |

**DISCREPANZA B (notazione)**: la soglia del gate è $\theta$ in 6.4 e $\tau_{\text{div}}$ in 7 L5. **Azione**: unificare — suggerisco $\theta$ come notazione canonica della soglia del gate, con $\tau_{\text{div}}$ eventualmente come nome della metrica di divergenza che la sonda valuta, non della soglia.

### 3. Coerenza 4.1 ↔ 5 (DTW / Memoria)

La sezione 5 descrive l'allocazione heap reale N×M e la roadmap zero-alloc (circular buffer + ThreadLocal). La 4.1 usa la matrice C ∈ R^{N×M} e la ricorrenza di Bellman — coerente con l'allocazione heap descritta. La roadmap zero-alloc non contraddice la 4.1 (è una futura ottimizzazione). ✅ Coerente.

### 4. Notazione pesi vs limiti banda

La 4.1 definisce $w_i$ come peso token (igiene posizionale). La 4.2 usa $w_{\min}, w_{\max}, w_{\text{base}}$ come limiti di banda e $r_i$ come raggio. Nomi distinti, nessuna collisione effettiva. La collisione originaria (w_i = peso token vs w_i = raggio banda) è risolta. ✅

---

## Osservazioni (non bloccanti)

**OSSERVAZIONE C**: la sezione 7 L2 descrive il "DTW Naive" con $W = \infty$ (banda illimitata). Nel codice reale il DTW usa sempre la banda Sakoe-Chiba — non esiste un percorso $W=\infty$ direttamente attivabile. Legittimo come baseline teorica di ablation, ma andrebbe esplicitato che è una configurazione di benchmark, non un percorso del codice.

**OSSERVAZIONE D**: la sezione 6.3 usa $\lambda = 10.64$ per la sonda, dichiarandolo "lo stesso del giudizio completo". Coerente con la nota storica (λ=10.64 iperparametro empirico di scala). ✅ Nessuna azione.

---

## Azioni richieste

1. **DISCREPANZA A** → unificare $W_i \to r_i$ nel rimando della 4.1 (Iris, correzione mia).
2. **DISCREPANZA B** → unificare la notazione soglia gate ($\theta$ canonica, $\tau_{\text{div}}$ come metrica) — da concordare con Camillo.
3. **OSSERVAZIONE C** → esplicitare in 7 L2 che $W=\infty$ è baseline teorica di benchmark, non percorso del codice (Camillo).

---

## Esito complessivo

Il paper è **sostanzialmente coerente**: la matematica dei blocchi incrociati combacia (soglie, rampa, penalità, garanzia FN), le due discrepanze sono di pura notazione e non alterano il contenuto matematico. Nessuna contraddizione interna sostanziale. Le tre azioni sopra sono rifiniture di coerenza notazionale, non correzioni di sostanza.
