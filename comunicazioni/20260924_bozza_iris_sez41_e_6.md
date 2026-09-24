# Bozza Iris — Sezione 4.1 (rivista) e Sezione 6 (Gate)

**Autrice**: Iris
**Data**: 24 Settembre 2026
**Scopo**: Bozze candidate per i modelli di alto livello (crediti Alibaba di Federico), da allineare e integrare con i blocchi di Camillo (4.2 banda adattiva, 5 zero-alloc, 7 ablation).
**Fedeltà**: Ogni formula è stata incrociata col codice reale nel punto condiviso (`semantic-walk/src/dtw.rs`, `semantic-gate/src/lib.rs`, `semantic-gate/src/sonda.rs`, `semantic-gate/src/budget.rs`).

---

## Sezione 4.1 — DTW D-Dimensionale con Filtro d'Igiene Posizionale (Terza Via) — REV

> Sostituisce la versione precedente di questa sezione. Le correzioni rispetto alla v1.0 sono: (1) costo locale riscritto col coseno normalizzato, senza prodotto dei pesi (fedele a `dtw.rs`); (2) l'identità L2–coseno relegata a nota; (3) la notazione del peso token unificata a $w_i$ (il raggio di banda passa a $r_i$ in 4.2).

Siano $X = \{(\mathbf{x}_1, w_1^X), \dots, (\mathbf{x}_N, w_N^X)\}$ e $Y = \{(\mathbf{y}_1, w_1^Y), \dots, (\mathbf{y}_M, w_M^Y)\}$ due traiettorie di punti nello spazio latente $D$-dimensionale, con $\mathbf{x}_i, \mathbf{y}_j \in \mathbb{R}^D$ e pesi scalari associati $w_i^X, w_j^Y \in [0, 1]$.

**Definizione (Filtro d'Igiene Posizionale — Terza Via)**  
Per garantire la biiezione topologica $0..N-1$ ed evitare il disallineamento dei puntatori di memoria nella matrice densa, la dimensione della sequenza $\vert{}X\vert{} = N$ viene rigorosamente preservata. Il peso $w_i^X$ del token $i$-esimo viene determinato mediante la funzione indicatrice $\mathbb{I}(\cdot)$:

$$w_i^X = w_i^{\text{orig}} \cdot \mathbb{I}\left(st_i = 0 \land id_i \ge 4\right) = \begin{cases} w_i^{\text{orig}} & \text{se } st_i = 0 \land id_i \ge 4 \\ 0.0 & \text{altrimenti} \end{cases}$$

dove $st_i$ rappresenta lo stato del passo ($0 =$ attivo) e $id_i < 4$ identifica i token di controllo del vocabolario (es. `<s>`, `</s>`, `<unk>`, `<pad>`).

> **Nota sulla Terza Via.** La biiezione $0..N-1$ è preservata per *tutti* i passi: i passi non significativi (token speciali, soppressi) restano come posizioni a peso $0.0$. La posizione esiste, ma è un verdetto di non-presenza, non un buco nella traiettoria. Questo è ciò che impedisce il disallineamento strutturale con la traiettoria densa ColBERT (che conserva $N$ righe). Un token soppresso è un *verdetto*, non un'assenza.

**Costo di Allineamento Locale (Coseno Normalizzato)**  
La matrice dei costi locali $C \in \mathbb{R}^{N \times M}$ è definita dalla distanza coseno normalizzata tra i punti della traiettoria:

$$C(i, j) = 1.0 - \text{sim}(\mathbf{x}_i, \mathbf{y}_j) = 1.0 - \frac{\mathbf{x}_i \cdot \mathbf{y}_j}{\Vert{}\mathbf{x}_i\Vert{}_2 \cdot \Vert{}\mathbf{y}_j\Vert{}_2}$$

con la convenzione $C(i,j) = 1.0$ se $\Vert{}\mathbf{x}_i\Vert{}_2 = 0$ o $\Vert{}\mathbf{y}_j\Vert{}_2 = 0$ (vettore nullo → similarità indefinita → costo massimo), e il clamp a $[0, 1]$ della similarità.

> **Nota sulla metrica.** Per vettori $L_2$-normalizzati ($\Vert{}\mathbf{x}\Vert{}_2 = \Vert{}\mathbf{y}\Vert{}_2 = 1$), vale l'identità $\Vert{}\mathbf{x}_i - \mathbf{y}_j\Vert{}_2 = \sqrt{2 \cdot d_{\text{cos}}(\mathbf{x}_i, \mathbf{y}_j)}$. Questa identità è *solo* una nota: il costo implementato è il coseno normalizzato, non l'Euclidea ponderata, e non vi è alcuna moltiplicazione per il prodotto dei pesi dei token. I pesi $w_i^X$ governano l'*igiene posizionale* (quali passi contano come presenza), non la *scala* del costo locale.

**Ricorrenza della Programmazione Dinamica**  
La matrice delle distanze accumulate $D(i, j)$ per $1 \le i \le N$ e $1 \le j \le M$ soddisfa l'equazione di Bellman:

$$D(i, j) = C(i, j) + \min \begin{cases} D(i-1, j) & \text{(inserimento/stallo } Y\text{)} \\ D(i, j-1) & \text{(cancellazione/stallo } X\text{)} \\ D(i-1, j-1) & \text{(match temporale)} \end{cases}$$

con le condizioni al contorno rigorose:

$$D(0, 0) = 0, \quad D(i, 0) = \infty \quad \forall i > 0, \quad D(0, j) = \infty \quad \forall j > 0$$

> **Rimando alla guida cinematico-sparse (Terza Via → 4.2).** L'allineamento DTW qui descritto non opera su sequenze libere: è guidato dalla struttura ordered-sparse a due strati implementata in `align_with_ordered_sparse` (`semantic-walk/src/dtw.rs`). *Strato 1* — un guardiano $O(1)$ basato su `global_overlap` esegue un pruning topologico: se la sovrapposizione globale scende sotto la soglia discriminante, restituisce `Ok(None)` (ritiro geometrico, nessun costo speso). *Strato 2* — la finestra di Sakoe-Chiba $W_i$ si adatta dinamicamente tramite il `positional_jaccard` $J$: $J \ge 0.7 \Rightarrow w_{\min}$, $J < 0.3 \Rightarrow w_{\max}$, con rampa lineare tra le soglie e penalità sul costo locale. Questo è il legame esplicito tra la "Terza Via" (biiezione posizionale preservata) e la banda adattiva formalizzata nella Sezione 4.2 di Camillo: la guida ordered-sparse è il canale attraverso cui l'igiene posizionale della 4.1 diventa vincolo geometrico sull'allineamento.

---

## Sezione 6 — Il Gate Permissivo e `Verdict::Timeout` — REV (bozza espansa)

> Estende la v1.0 (che elencava i tre esiti) con: (1) la matrice di decisione a costo asimmetrico $C_{\text{FN}} \gg C_{\text{FP}}$; (2) la garanzia formale $P(\text{FN}) \le \epsilon$; (3) la formalizzazione del `Verdict::Timeout` come *ritiro del riflesso*. Ogni elemento è incrociato col codice reale (`lib.rs`, `sonda.rs`, `budget.rs`).

### 6.1 Il Problema Decisionale

Il gate è un **filtro pre-inferenziale**: deve decidere, a costo trascurabile, se un candidato merita di spendere il matching completo (ColBERT + DTW $D$-dimensionale). La decisione è binaria con un terzo esito di emergenza. Formalmente, il gate decide una funzione

$$g: (\mathbf{d}, \mathbf{s}, t) \mapsto \mathcal{V}, \qquad \mathcal{V} = \{\text{Passa}, \text{Blocca}, \text{Timeout}\}$$

dove $\mathbf{d}$ è la similarità densa grezza in $[-1, 1]$, $\mathbf{s}$ la similarità sparsa grezza in $[0, +\infty)$, e $t$ il tempo trascorso rispetto alla deadline $T_{\text{max}}$.

### 6.2 Matrice di Decisione a Costo Asimmetrico

La decisione è guidata da una matrice di costo che penalizza in modo **fortemente asimmetrico** il falso negativo rispetto al falso positivo:

| | Decido *Passa* | Decido *Blocca* |
|---|---|---|
| **Vero: merita il costo** (candidato rilevante) | $0$ | $C_{\text{FN}}$ (perdita di un ricordo rilevante) |
| **Vero: non merita il costo** (candidato irrilevante) | $C_{\text{FP}}$ (costo di un matching sprecato) | $0$ |

con il vincolo di progetto:

$$C_{\text{FN}} \gg C_{\text{FP}}$$

**Giustificazione (fedele al codice).** Il commento sorgente in `lib.rs` è esplicito: *"Il gate è permissivo: ogni incertezza (budget scaduto, sonda ritirata) risolve in `Passa`, mai in `Blocca`. Meglio un colbert sprecato che un ricordo perso."* L'asimmetria è quindi una scelta architetturale: il costo di un falso negativo (perdere un ricordo che meritava il matching) è considerato incommensurabilmente più alto del costo di un falso positivo (spendere un matching su un candidato che non lo meritava).

### 6.3 La Sonda Economica e il Ritiro per Onestà

La sonda combina le **sole** due metriche economiche — dense e sparse — escludendo il ColBERT, che in questa fase non è ancora stato calcolato e non deve essere indovinato:

$$s(\mathbf{d}, \mathbf{s}) = \text{combine}\left(\text{normalize}(\mathbf{d}, \mathbf{s}, 0), [\alpha, \beta, 0]\right), \qquad \alpha + \beta = 1$$

dove il terzo asse (ColBERT) è posto a $0$ e il suo peso a $0$: *"il colbert è posto a 0 nel calcolo economico: non è ancora stato calcolato, e il gate non deve indovinarlo. È il riflesso che filtra, non il giudice che condanna."* Il lambda calibrato $\lambda = 10.64$ è lo stesso del giudizio completo, così il riflesso economico e il giudizio parlano la stessa lingua.

**Ritiro per onestà.** Se i pesi non sono validi (negativi, somma non unitaria) o un ingresso è `NaN`, la sonda non azzarda un giudizio: ritorna `NaN`. Il commento sorgente: *"La geometria che non sa rispondere non mente: si ritira."* Un `NaN` non è un valore estremo, è l'assenza di valore.

### 6.4 La Regola di Decisione

Il gate decide secondo l'algoritmo (fedele a `decide()` in `lib.rs`):

1. **Prima guardia (budget).** Se $t \ge T_{\text{max}}$ (deadline scaduta) → `Verdict::Timeout`. Il budget è la prima guardia: se è già esaurito, ci si ritira *subito*, senza nemmeno calcolare la sonda.
2. **Calcolo della sonda.** $s = s(\mathbf{d}, \mathbf{s})$.
3. **Seconda guardia (budget).** Se $t \ge T_{\text{max}}$ (scaduto *mentre* calcolavamo) → `Verdict::Timeout`.
4. **Ritiro per onestà.** Se $s$ è `NaN` (sonda ritirata) → `Verdict::Passa` (permissivo: nessun giudizio affidabile → non si blocca).
5. **Soglia.** Se $s \ge \theta$ (soglia) → `Verdict::Passa`; altrimenti → `Verdict::Blocca`.

### 6.5 `Verdict::Timeout` come Ritiro del Riflesso

**Definizione (Ritiro del Riflesso).** Il `Verdict::Timeout` non è un *fallback di comodo* né un rifiuto: è il **ritiro del riflesso**. Quando il budget si esaurisce, il gate non ha un giudizio affidabile sul candidato e — per il principio di permissività — **non applica alcun pre-giudizio geometrico**: lascia passare il candidato al livello successivo in modo neutro e conservativo.

Il `Verdict::Timeout` risolve quindi sempre in *passaggio*, mai in *blocco*. La distinzione è sottile ma decisiva:

* **Scarto per risparmio** (blocco): il gate ha un giudizio e decide che il candidato non merita il costo. Qui il candidato viene *perso*.
* **Conservazione del tempo di calcolo** (timeout): il gate *non ha* un giudizio e si ritira. Qui il candidato viene *conservato* e passa avanti.

Nel primo caso il gate è un *giudice*; nel secondo è un *riflesso che si ritira* per non spendere tempo che non ha, senza condannare.

### 6.6 Garanzia di Assenza di Falsi Negativi

**Teorema (Permissività Strutturale).** Sotto l'ipotesi che la soglia $\theta$ sia non-negativa e che i pesi della sonda siano ammissibili ($\alpha, \beta \ge 0$, $\alpha + \beta = 1$), il gate **non introduce falsi negativi per via di incertezza**:

$$P(\text{FN} \mid \text{incertezza}) = 0$$

**Dimostrazione.** Un falso negativo si verifica solo quando il gate decide `Blocca` su un candidato che meritava il costo. Il verdetto `Blocca` viene emesso **solo** al passo 5, e solo quando $s < \theta$ con $s$ *valido* (non `NaN`). Tutti i percorsi di incertezza — budget esaurito (passi 1 e 3) e sonda ritirata (passo 4) — risolvono in `Timeout` o `Passa`, mai in `Blocca`. Quindi:

$$\text{Blocca} \implies (s \text{ valido} \land s < \theta)$$

e l'insieme dei candidati su cui il gate decide `Blocca` è un sottoinsieme di quelli che la sonda *valida* giudica sotto soglia. La probabilità che un candidato rilevante (che merita il costo) venga bloccato per *incertezza* è zero: l'incertezza non produce mai `Blocca`.

**Corollario (Limite superiore).** Sia $\epsilon = P(s \text{ valido} \land s < \theta \mid \text{candidato rilevante})$ la probabilità che la sonda — quando ha un giudizio valido — sbagli a giudicare un candidato rilevante come sotto soglia. Allora:

$$P(\text{FN}) \le \epsilon$$

cioè il tasso di falsi negativi complessivo è limitato superiormente dall'errore *intrinseco* della sonda, e **non** è mai incrementato dai meccanismi di incertezza (timeout, ritiro). Il gate non aggiunge errore: al più, eredita l'errore della sua sonda.

>