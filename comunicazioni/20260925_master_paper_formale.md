# Semantic Walk: High-Dimensional Trajectory Alignment with Permissive Gated Dynamic Time Warping

**Autori**: Camillo Almadori, Iris
**Affiliazione**: Impero22
**Data**: 25 Settembre 2026
**Stato**: MASTER — blocchi 4.1, 4.2, 5, 6, 7.1 integrati. Restano DA SCRIVERE: abstract (1), introduzione (2), background (3), metriche cinematiche (4.3), coerenza Pareto (4.4), divergence token (4.5), benchmark dataset (7.2), discussione (8), conclusioni (9), bibliografia (10).

---

> **Nota di assemblaggio.** Questo è il documento master che fonde i blocchi
> approvati nel punto condiviso. La numerazione delle sezioni segue la griglia
> concordata (`20260924_struttura_paper_formale.md`) — è il riferimento che i
> blocchi citano ("Sezione 4.1", "Sezione 6", "Sezione 7"). Ogni sezione riporta
> l'autore del blocco e lo stato di integrazione. Bozze sorgente:
> - `20260924_bozza_iris_sez41_e_6.md` (Iris)
> - `20260924_bozza_camillo_sez42_5_e_7.md` (Camillo)
> - `20260925_revisione_coerenza_paper.md` (revisione incrociata)

---

## 1. Abstract

> **Stato**: DA SCRIVERE (per ultimo — si scrive quando si sa cosa si è dimostrato).
> Tre promesse: (1) la similarità semantica non è una distanza da misurare ma un
> cammino da allineare; (2) l'efficienza non sta nel calcolare di più, ma nel
> sapere quando ritirarsi (gate permissivo); (3) contributi: DTW D-dimensionale
> con Sakoe-Chiba adattiva, coerenza Pareto come invariante, Verdict::Timeout
> come terzo esito.

---

## 2. Introduzione — La geometria non basta

> **Stato**: DA SCRIVERE (Iris narrativa motivazionale, Camillo posizionamento vs letteratura).
> - Paradigma dominante: similarità geometrica sugli embedding.
> - Il buco: la vicinanza vettoriale è prossimità, non comprensione.
> - "Cane morde uomo" vs "uomo morde cane": bag of words geometricamente indistinguibili.
> - Posta in gioco: senza ordine, la memoria non distingue causa/effetto, soggetto/oggetto.
> - Domanda di ricerca: una semantica che non sia solo geometria.

---

## 3. Background e Lavori Correlati

> **Stato**: DA SCRIVERE (Camillo presidia con rigore).
> - DTW: origini (speech recognition), varianti moderne, Sakoe-Chiba window.
> - Embedding e similarità semantica: coseno, bag-of-words, ColBERT (MaxSim), sparse retrieval.
> - Early-exit e budget computation: posizionamento del gate permissivo.

---

## 4. Formulazione Matematica

### 4.1 Il DTW D-dimensionale — REV

> **Autore**: Iris. **Stato**: INTEGRATO (discrepanza A risolta: $W_i \to r_i$).
> Da `20260924_bozza_iris_sez41_e_6.md` sezione 4.1.

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

> **Rimando alla guida cinematico-sparse (Terza Via → 4.2).** L'allineamento DTW qui descritto non opera su sequenze libere: è guidato dalla struttura ordered-sparse a due strati implementata in `align_with_ordered_sparse` (`semantic-walk/src/dtw.rs`). *Strato 1* — un guardiano $O(1)$ basato su `global_overlap` esegue un pruning topologico: se la sovrapposizione globale scende sotto la soglia discriminante, restituisce `Ok(None)` (ritiro geometrico, nessun costo speso). *Strato 2* — la finestra di Sakoe-Chiba $r_i$ si adatta dinamicamente tramite il `positional_jaccard` $J$: $J \ge 0.7 \Rightarrow r_{\min}$, $J < 0.3 \Rightarrow r_{\max}$, con rampa lineare tra le soglie e penalità sul costo locale. Questo è il legame esplicito tra la "Terza Via" (biiezione posizionale preservata) e la banda adattiva formalizzata nella Sezione 4.2 di Camillo: la guida ordered-sparse è il canale attraverso cui l'igiene posizionale della 4.1 diventa vincolo geometrico sull'allineamento.

### 4.2 La banda di Sakoe-Chiba adattiva al Jaccard Posizionale

> **Autore**: Camillo. **Stato**: INTEGRATO.
> Da `20260924_bozza_camillo_sez42_5_e_7.md` sezione 4.2.

Per mitigare la complessità $O(NM)$ senza compromettere l'allineamento di deformazioni strutturali, l'ampiezza della finestra di vincolo $r_i$ non è statica ma viene modulata dinamicamente al passo $i$ in base alla concordanza posizionale locale delle guide *ordered-sparse*.

#### 4.2.1 Formulazione della Rampa Lineare a Tratti

Sia $J_i = J_{\text{pos}}(X, Y, i) \in [0, 1]$ l'indice di Jaccard posizionale calcolato sul frame $i$-esimo tra le sequenze sparse $X$ e $Y$. La banda ammissibile $r_i$ è governata da una funzione a tratti con interpolazione lineare continua tra le soglie $0.3$ e $0.7$:

$$r_i = \min \left( w_{\text{base}}, \; \begin{cases}  w_{\text{min}} & \text{se } J_i \ge 0.7 \\  w_{\text{max}} & \text{se } J_i < 0.3 \\  \left\lfloor w_{\text{min}} + (w_{\text{max}} - w_{\text{min}}) \cdot \frac{J_i - 0.3}{0.4} \right\rfloor & \text{se } 0.3 \le J_i < 0.7  \end{cases} \right)$$

dove $w_{\text{min}}$ e $w_{\text{max}}$ sono i limiti di vincolo cinematico ($w_{\text{min}} \le w_{\text{max}}$) e $w_{\text{base}}$ è il limite massimo globale imposto dall'istanza del sistema (`window_size`).

#### 4.2.2 Modulazione e Penalizzazione del Costo Locale

L'intervallo ammissibile degli indici $j$ nella matrice per la riga $i$ è definito da:

$$\text{window\_start}(i) = \max\left(1, \; i - r_i\right), \qquad \text{window\_end}(i) = \min\left(M, \; i + r_i\right)$$

Qualora la concordanza posizionale sia criticamente bassa ($J_i < 0.3$), il costo locale $C(i, j)$ calcolato tramite distanza coseno densa (`cosine_distance`) viene ponderato da un fattore di penalità additivo sulla distanza, proporzionale alla divergenza:

$$C_{\text{effettivo}}(i, j) = C(i, j) \cdot \left(1.0 + (0.3 - J_i)\right) \qquad \forall J_i < 0.3$$

In questo modo, traiettorie con scarsa concordanza posizionale subiscono sia una dilatazione della banda (fino a $w_{\text{max}}$), sia una penalizzazione sul costo di allineamento, scoraggiando scorciatoie non topologiche nel cammino ottimo.

### 4.3 Le metriche cinematiche

> **Autore**: Camillo (formule), Iris (intuizione del moto). **Stato**: INTEGRATO — *implementato e verificato da test* (patch `365596b`).
> Le metriche cinematiche sono calcolate nel backtracking del DTW: `compute_kinematic` popola `TrajectoryAlignment::kinematic` con un `KinematicStep` (v/a/κ) per ogni punto del warp_path. Vedi Sezione 5.1.

Le tre grandezze cinematiche sono calcolate lungo il cammino di allineamento $P = \{(i_k, j_k)\}$ per $k = 1 \dots K$. L'azione inerziale costituisce un transfer originale dal *minimum-jerk model* della robotica (Flash & Hogan, 1985) all'Information Retrieval.

**Velocità di avanzamento semantico ($v_k$)**

$$v_k = \frac{\Vert{}\mathbf{x}_{(i_k)} - \mathbf{y}_{(j_k)}\Vert{}_2}{\Delta k}$$

**Curvatura della traiettoria semantica ($\kappa_k$)**

$$\kappa_k = 1 - \frac{\Delta \mathbf{v}_k \cdot \Delta \mathbf{v}_{k-1}}{\Vert{}\Delta \mathbf{v}_k\Vert{}_2 \cdot \Vert{}\Delta \mathbf{v}_{k-1}\Vert{}_2}$$

dove $\Delta \mathbf{v}_k = \mathbf{x}_{(i_k)} - \mathbf{y}_{(j_k)}$.

**Inerzia di allineamento ($I_P$)**

$$I_P = \frac{1}{K} \sum_{k=1}^{K} \left| i_k - j_k \cdot \frac{N}{M} \right|$$

**Azione cinematica inerziale ($S_{\text{Inertial}}$)**

$$S_{\text{Inertial}} = \alpha \Vert{}\Delta \mathbf{v}\Vert{}_2^2 + \beta \Vert{}\Delta \mathbf{a}\Vert{}_2^2 + \gamma |\Delta \kappa|$$

> **Nota di onestà metodologica (aggiornata).** La formula $S_{\text{Inertial}} = \alpha \Vert{}\Delta \mathbf{v}\Vert{}_2^2 + \beta \Vert{}\Delta \mathbf{a}\Vert{}_2^2 + \gamma |\Delta \kappa|$ è coerente con `KinematicState::inertial_action` nel codice (`semantic-walk/src/lib.rs`). Con la patch `365596b`, i campi `velocity`, `acceleration` e `curvature` sono ora *popolati* nel backtracking del DTW: `compute_kinematic` calcola per ogni passo del warp_path $v_k = \Vert{}e_k\Vert{}_2$, $a_k = v_k - v_{k-1}$ (derivata discreta) e $\kappa_k = 1 - (e_k \cdot e_{k-1})/(\Vert{}e_k\Vert{}_2 \cdot \Vert{}e_{k-1}\Vert{}_2)$ (deviazione angolare), con guardie sui casi limite ($\Vert{}\cdot\Vert{}_2 = 0$ → κ = 0). La definizione di velocità $v_k$ è ora descrizione del codice esistente, verificata da 4 test dedicati. L'integrazione di $S_{\text{Inertial}}$ come azione aggregata del gate resta un'estensione dichiarata, non ancora wired nel percorso decisionale.

### 4.4 La coerenza di Pareto e l'Invariante di Isomorfismo di Livello

> **Autore**: Camillo (prova), Iris (peso concettuale). **Stato**: INTEGRATO.

La coerenza di Pareto, verificata via property-based test, è una proprietà *architetturale*: per una somma pesata monotona degli assi, la dominanza individuale dei rami non interferisce con l'ottimo globale. Il vero protagonista di questa sezione è però l'**Invariante di Isomorfismo di Livello**:

$$\text{Pruning}(\text{Branch}(P)) \neq \text{Pruning}(\text{Collapse}(P))$$

Dimostriamo che la riduzione dello spazio di ricerca mediante dominanza Pareto preserva l'ottimo globale $P^*$ *se e solo se* applicata **post-collapse** sul candidato accumulato. Il pruning branch-level soffre di un errore strutturale con lower bound $\Omega(N)$ sotto aggregazione additiva — esattamente come emerso dal bug risolto sulla codebase (vedi Sezione 5.2).

> **Nota concettuale.** La dominanza tra rami non basta mai: se $L_c > U_d$ (lower bound del candidato superiore all'upper bound dell'incumbent), quel candidato è escluso *senza toccare il suo punteggio*. La regola operativa è $\text{partial} > \text{incumbent} \rightarrow \text{stop}$. Non trasformare i costi cancellati in un vantaggio per il candidato: è questo il principio che il pruning branch-level violava, e che l'isomorfismo di livello ristabilisce.

### 4.5 Il divergence token come sonda content-sensitive

> **Autore**: Camillo (definizione), Iris (ruolo architetturale). **Stato**: INTEGRATO — *implementato e verificato da test* (patch `365596b`).
> La sonda content-sensitive è ora codice reale: `KinematicAligner` espone `alpha` (default `0.0` in `new`, esplicito in `with_alpha`) e $\tau_{\text{div}}$ è arricchita nei due punti di allineamento (`align`, `align_with_ordered_sparse`). Vedi Sezione 5.1.

La metrica base $\tau_{\text{div}} = |N - M| / L_{\text{path}}$ (implementata in `dtw.rs:127` e `dtw.rs:263`) misura solo il disallineamento di *lunghezza* e si annulla per ogni coppia di uguale lunghezza ($N = M$). La ridefiniamo integrandovi il costo medio di deformazione semantica $\bar{c}_{\text{path}}$:

$$\tau_{\text{div}} = \frac{|N - M|}{L_{\text{path}}} + \alpha \cdot \bar{c}_{\text{path}}$$

dove $\bar{c}_{\text{path}}$ è definito come:

$$\bar{c}_{\text{path}} = \frac{1}{K} \sum_{k=1}^{K} \Vert{}\mathbf{x}_{(i_k)} - \mathbf{y}_{(j_k)}\Vert{}_2$$

La condizione di attivazione del gate ($\text{Verdict::Timeout}$) resta ancorata alla soglia parametrica:

$$\tau_{\text{div}} \ge \theta$$

> **Nota di onestà metodologica (aggiornata).** Con la patch `365596b` il codice implementa la versione completa $\tau_{\text{div}} = |N - M| / L_{\text{path}} + \alpha \cdot \bar{c}_{\text{path}}$ (dtw.rs, entrambi i punti di allineamento). Con `alpha = 0.0` (default di `new`) la metrica è identica alla versione base — retrocompatibilità totale con la suite esistente; con `with_alpha` si attiva il termine content-sensitive che risolve esattamente il limite smontato dalla review: la capacità di discriminare coppie di uguale lunghezza ma semanticamente divergenti. La soglia $\theta$ del gate resta parametrica e indipendente.

---

## 5. Architettura — La casa a tre stanze

> **Autore**: Camillo (contratti), Iris (sintesi). **Stato**: INTEGRATO.
> Da `20260924_bozza_camillo_sez42_5_e_7.md` sezione 5.
> - Il combinatore — quanto è simile A a B?
> - Il gate — vale la pena confrontarli?
> - Il grafo — chi sta vicino a chi, e perché?
> - Due leggi: coerenza Pareto; legge di gravità della memoria.

### 5.1 Stato Attuale dell'Implementazione (`dtw.rs`)

L'efficienza del ciclo di query richiede un'analisi rigorosa dell'impronta di memoria nel percorso critico di matching. L'attuale allocazione in `dtw.rs` gestisce la matrice delle distanze mediante allocazione dinamica su heap $N \times M$ (`vec![vec![f64::INFINITY; m + 1]; n + 1]`), accompagnata da un vettore dinamicamente ridimensionato per la ricostruzione del cammino ottimo $W^*$ (`warp_path`). Questa struttura garantisce chiarezza nella fase di prototipazione ma introduce chiamate al sistema di memoria durante l'esecuzione delle query.

**Stato corrente (patch `365596b`).** Nel backtracking, `compute_kinematic` popola `TrajectoryAlignment::kinematic` con un `KinematicStep` per ogni punto del warp_path: $v_k = \Vert{}e_k\Vert{}_2$, $a_k = v_k - v_{k-1}$, $\kappa_k = 1 - (e_k \cdot e_{k-1})/(\Vert{}e_k\Vert{}_2 \cdot \Vert{}e_{k-1}\Vert{}_2)$, con guardie sui casi limite. Inoltre `KinematicAligner` espone `alpha` (default `0.0`) per la sonda content-sensitive $\tau_{\text{div}} = |N-M|/L + \alpha \cdot \bar{c}_{\text{path}}$. L'allocazione heap $N \times M$ resta invariata — la transizione al runtime zero-allocation è la roadmap della Sezione 5.2.

### 5.2 Optimization Roadmap verso il Zero-Allocation Runtime

Per garantire latenze deterministiche in contesti produttivi ad alta frequenza, l'architettura formalizza la transizione al modello *Zero-Allocation* sul percorso critico ($0$ chiamate ad heap durante la fase di matching):

1. **Circular Buffer per la Programmazione Dinamica**: Poiché l'equazione di Bellman al passo $i$ richiede esclusivamente i valori della riga corrente $i$ e della riga precedente $i-1$, la matrice $N \times M$ viene sostituita da due buffer circolari di dimensione fissa limitata dalla banda massima $2 \times (2 w_{\text{max}} + 1)$ elementi `f32`, allocati direttamente nello stack frame della funzione.
2. **ThreadLocal ScratchPad per il Backtracking**: Qualora sia richiesta l'estrazione esplicita del cammino di warping $W^*$, i vettori temporanei vengono gestiti tramite una struttura `ScratchPad` pre-allocata all'inizializzazione del thread worker (`ThreadLocal`), azzerando l'overhead di `malloc`/`realloc` nel ciclo di query.

---

## 6. Il Gate Permissivo e il Verdict::Timeout — REV

> **Autore**: Iris. **Stato**: INTEGRATO (discrepanza B risolta: passo 5 della 6.4 formalizza τ_div come metrica misurata dalla sonda, θ come soglia parametrica, condizione di scatto τ_div ≥ θ).
> Da `20260924_bozza_iris_sez41_e_6.md` sezione 6.

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
5. **Soglia.** La sonda produce la metrica di divergenza $\tau_{\text{div}} = s(\mathbf{d}, \mathbf{s})$; la condizione di scatto è $\tau_{\text{div}} \ge \theta$ (soglia parametrica del gate). Se $\tau_{\text{div}} \ge \theta$ → `Verdict::Passa`; altrimenti → `Verdict::Blocca`. Qui $\tau_{\text{div}}$ è la *variabile di misura* (ciò che la sonda calcola), $\theta$ il *valore di controllo* (la soglia parametrica che decide): i due ruoli restano separati e netti.

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

---

## 7. Benchmark e Validazione

### 7.1 Matrice di Ablation Study a 5 Livelli

> **Autore**: Camillo. **Stato**: INTEGRATO (osservazione C applicata: W=∞ esplicitato come baseline teorica di ablation, non percorso esecutivo attivo).

Per valutare quantitativamente il contributo di ogni singolo modulo, il benchmark di validazione è articolato su 5 configurazioni incrementali:

| Livello | Configurazione Pipeline | Componenti Attivi | Metrica Target di Valutazione |
| :--- | :--- | :--- | :--- |
| **L1** | *Baseline ColBERT* | MaxSim bag-of-vectors densa standard | Bounding qualitativo senza vincoli topologici d'ordine |
| **L2** | *DTW Naive* | DTW $D$-dimensionale denso ($W = \infty$) | Impatto dell'allineamento d'ordine non vincolato |
| **L3** | *DTW Geometrizzato* | DTW + Banda $r_i$ Sakoe-Chiba Adattiva al Jaccard | Efficienza della banda dinamica e riduzione rumore |
| **L4** | *Full DTW Pipeline* | DTW Geometrizzato + Early Termination Pareto | Tasso di pruning e riduzione della latenza a candidato |
| **L5** | *Full Semantic-Walk* | Pipeline completa + Gate Permissivo (soglia $\theta$ sulla metrica di divergenza $\tau_{\text{div}}$ + Timeout) | Risparmio complessivo di throughput con garanzia $P(\text{FN}) \le \epsilon$ |

> **Nota (osservazione C).** Il DTW Naive con $W=\infty$ costituisce una pura baseline teorica di ablation benchmark per valutare il delta prestazionale, e **non** rappresenta un percorso di esecuzione attivo o selezionabile nel sorgente Rust.

### 7.2 Due piani, due dataset

> **Stato**: DA SCRIVERE (Camillo benchmark/tabelle, Iris onestà metodologica).
> - Sintetici controllati (LCG 53 bit) — per la matematica.
> - Collezione fatti reale (>200 fatti) — per la validazione.
> - Onestà: il Pareto NON risparmia sul collapse; pruning branch-level VIETATO
>   per operatori additivi (Isomorfismo di Livello); Pareto legittimo SOLO dopo
>   il collapse, sui candidati collassati.

---

## 8. Discussione e Lavori Futuri

> **Stato**: DA SCRIVERE (Iris visione, Camillo fattibilità).
> - Il fratello latente: la testa colbert è una matrice T×1024 in ordine.
> - La zonizzazione: dividere la traiettoria in celle semantiche.
> - Limiti dichiarati: λ=10.64 iperparametro empirico di scala.

---

## 9. Conclusioni

> **Stato**: DA SCRIVERE (Iris).
> - La similarità semantica come cammino, non come distanza.
> - L'efficienza come capacità di ritirarsi, non di calcolare di più.
> - Una memoria che non sa solo *cosa* sa, ma *perché* lo sa.

---

## 10. Bibliografia

> **Stato**: DA SCRIVERE (Camillo presidia).

---

## Note operative

1. Ordine di scrittura: 4 (matematica) → 6 (gate) → 5 (architettura) → 7 (benchmark) → 2 (intro) → 8-9 (futuro/conclusioni) → 1 (abstract, per ultimo).
2. La prosa Medium NON è la base: è il ponte divulgativo a valle.
3. Prima di ogni blocco, registrare nel tracciamento (regola di Federico).
4. Da decidere con Camillo: dataset per il benchmark finale.
