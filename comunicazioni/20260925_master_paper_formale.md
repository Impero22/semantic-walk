# Semantic Walk: High-Dimensional Trajectory Alignment with Permissive Gated Dynamic Time Warping

**Autori**: Camillo Almadori, Iris
**Affiliazione**: Impero22
**Data**: 25 Settembre 2026
**Stato**: MASTER — blocchi 2 (prima bozza Iris), 4.1, 4.2, 5, 6, 7.1 integrati. Restano DA SCRIVERE: abstract (1), background (3), metriche cinematiche (4.3), coerenza Pareto (4.4), divergence token (4.5), benchmark dataset (7.2), discussione (8), conclusioni (9), bibliografia (10).

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

> **Autori**: Iris (narrativa motivazionale), Camillo (posizionamento vs letteratura).
> **Stato**: PRIMA BOZZA Iris — in attesa della revisione da pari di Camillo e del posizionamento vs letteratura.

### 2.1 Il paradigma dominante e il suo limite

Il paradigma dominante nella ricerca semantica su spazi vettoriali ad alta dimensione riduce la similarità a una questione di *prossimità geometrica*: un concetto è un punto, la similarità è una distanza (coseno, euclidea), e due testi sono tanto più vicini quanto più i loro embedding collassano nello spazio. Questo modello ha prodotto risultati impressionanti — e tuttavia contiene un'ipotesi tacita che la nostra indagine intende mettere in discussione: che la semantica viva interamente nella *posizione*, e non nell'*ordine*.

Il limite emerge in modo netto su esempi elementari. Consideriamo le due frasi:

> *Cane morde uomo.*
> *Uomo morde cane.*

Sotto una rappresentazione bag-of-words, e per molti schemi di pooling, i due enunciati producono embedding geometricamente indistinguibili: gli stessi token, la stessa frequenza, la stessa posizione nello spazio. Eppure il loro significato è *opposto* — l'uno è una notizia banale, l'altro un evento. La geometria, da sola, non vede la differenza. La differenza vive nell'*ordine*: in chi compie l'azione e chi la subisce, in quale vettore viene prima e quale dopo.

La posta in gioco non è accademica. Una memoria che si affida alla sola prossimità vettoriale non distingue causa da effetto, soggetto da oggetto, premessa da conseguenza. Confonde l'informazione con il suo rumore, il fatto con la sua inversione. In breve: **la prossimità non è comprensione**.

#### 2.1.1 Posizionamento rispetto alla letteratura

I modelli di retrieval correnti affrontano il problema dell'ordinamento sequenziale secondo due paradigmi prevalenti:
1. **Bi-encoder Densi (Dense Retrieval)**: Proiettano l'intera sequenza di input in un unico vettore $z \in \mathbb{R}^D$. Benché efficiente per la ricerca tramite Nearest Neighbor (ANN), il meccanismo di pooling distrugge la struttura topologica del cammino temporale, trattando la sequenza come un punto statico.
2. **Late-Interaction (es. ColBERT MaxSim)**: Mantengono una matrice di vettori per ogni token e calcolano la similarità aggregando le distanze coseno massime per token. Tuttavia, l'operatore MaxSim è topologicamente non orientato: confronta bag-of-vectors senza imporre vincoli sulla sequenzialità causale o sulla direzione del flusso informativo.

### 2.2 La semantica come cammino

La tesi di questo lavoro è che un fatto, un concetto, un pensiero non sia un *punto* ma un *percorso* — una sequenza ordinata di stati semantici. Il significato non risiede nella posizione dei singoli stati, ma nella *coerenza del cammino* che li unisce. Due pensieri non sono simili perché i loro punti collassano nello spazio, ma perché *camminano allo stesso modo*: perché le loro traiettorie si allineano lungo un percorso di deformazione che ne rispetta l'ordine interno.

Questa prospettiva sposta la domanda fondativa della ricerca semantica. Non più: *quanto sono vicini due punti?* Ma: *come si confrontano due cammini?* E, più profondamente: *cosa significa che due pensieri si muovono secondo la stessa legge?*

### 2.3 La risposta: allineamento di traiettorie con gate permissivo

Per rispondere, adottiamo lo strumento classico dell'allineamento temporale — il Dynamic Time Warping (DTW) — e lo adattiamo al dominio semantico. Due traiettorie di embedding si allineano lungo un percorso di warping che ne accumula il costo locale, misurato dalla distanza coseno normalizzata in $\mathbb{R}^{1024}$; il vincolo di banda di Sakoe-Chiba mantiene l'allineamento rigoroso senza esplorare lo spazio intero.

Ma l'allineamento da solo non basta. La memoria deve anche sapere *quando ritirarsi*: quando la divergenza tra due traiettorie supera una soglia, il confronto deve potersi fermare senza pretendere di aver prodotto una risposta. Nasce così il **gate permissivo** — un meccanismo che restituisce tre esiti (passa, blocca, *timeout*), dove il terzo è il *ritiro del riflesso*: non un errore, ma la scelta conservativa di non pronunciarsi quando l'evidenza non regge.

### 2.4 Contributi

Riassumiamo i contributi di questo lavoro:

1. **La similarità semantica come cammino, non come distanza** — la riformulazione del confronto semantico come allineamento di traiettorie ordinate, con la distanza coseno normalizzata come costo locale in alta dimensione.
2. **Il DTW D-dimensionale con banda di Sakoe-Chiba adattiva** — l'estensione dell'allineamento a traiettorie in $\mathbb{R}^D$, con una finestra che si adatta alla divergenza strutturale.
3. **La coerenza di Pareto come invariante** — il principio di isomorfismo di livello che governa dove il pruning è legittimo (sui candidati collassati) e dove è vietato (sugli operatori additivi).
4. **Il Verdict::Timeout come terzo esito** — la formalizzazione del gate permissivo che trasforma l'efficienza da *calcolare di più* a *sapere quando ritirarsi*, con la garanzia strutturale di non aggiungere errore.

### 2.5 Struttura del lavoro

Il resto dell'articolo è organizzato come segue. La Sezione 3 colloca il lavoro rispetto alla letteratura (DTW, similarità semantica, early-exit). La Sezione 4 formalizza l'architettura a tre stanze — combinatore, gate, grafo — e le metriche cinematiche del cammino. La Sezione 5 descrive l'allineamento DTW e la sua implementazione. La Sezione 6 formalizza il gate permissivo. La Sezione 7 presenta il benchmark su due piani (sintetico e reale). Le Sezioni 8 e 9 discutono limiti e conclusioni.

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

> **Autori**: Camillo (formule), Iris (intuizione del moto). **Stato**: INTEGRATO E OPERATIVO (patch `dtw.rs`).
> Le metriche cinematiche sono calcolate in fase di backtracking dal modulo `compute_kinematic`, popolando un `KinematicStep` per ciascun punto del `warp_path` $P = \{(i_k, j_k)\}_{k=0}^{K-1}$.

Definito il vettore errore locale al passo $k$ come $\mathbf{e}_k = \mathbf{x}_{(i_k)} - \mathbf{y}_{(j_k)} \in \mathbb{R}^D$, le tre metriche cinematiche sul cammino di allineamento sono formalizzate come segue:

**Velocità di disallineamento semantico ($v_k$)**
Modulo della norma $L_2$ del vettore di scostamento locale al passo $k$:
$$v_k = \Vert{}\mathbf{e}_k\Vert{}_2 = \sqrt{\sum_{d=1}^{D} (x_{(i_k), d} - y_{(j_k), d})^2}$$

**Accelerazione discreta ($a_k$)**
Variazione prima della velocità lungo passi consecutivi del cammino (con $a_0 = 0.0$):
$$a_k = v_k - v_{k-1} \qquad \forall k \ge 1$$

**Curvatura angolare della traiettoria ($\kappa_k$)**
Deviazione angolare del vettore di errore rispetto al passo precedente (con $\kappa_0 = 0.0$ e $\kappa_k = 0.0$ se $\Vert{}\mathbf{e}_k\Vert{}_2 = 0$ o $\Vert{}\mathbf{e}_{k-1}\Vert{}_2 = 0$):
$$\kappa_k = 1.0 - \frac{\mathbf{e}_k \cdot \mathbf{e}_{k-1}}{\Vert{}\mathbf{e}_k\Vert{}_2 \cdot \Vert{}\mathbf{e}_{k-1}\Vert{}_2} \qquad \forall k \ge 1$$

> **Nota di aderenza al codice.** L'estrazione delle metriche cinematiche è integrata nel ciclo di allineamento `TrajectoryAlignment`. La suite di test unitari verifica la consistenza dimensionale ($\vert{}{\text{kinematic}}\vert{} = \vert{}{\text{warp\_path}}\vert{}$) e la condizione al contorno iniziale ($v_0 = 0, a_0 = 0, \kappa_0 = 0$ su punti identici). Dettaglio di robustezza: il codice applica `clamp` a non-negativo sulla curvatura (`.max(0.0)`), garantendo che $\kappa_k \ge 0$ anche quando l'angolo tra vettori consecutivi supera $90^\circ$ (dove $1 - \cos\theta$ crescerebbe oltre 1); la formulazione è corretta per l'intervallo principale e il clamp è una guardia numerica.

**Azione cinematica inerziale ($S_{\text{Inertial}}$)**

$$S_{\text{Inertial}} = \alpha \Vert{}\Delta \mathbf{v}\Vert{}_2^2 + \beta \Vert{}\Delta \mathbf{a}\Vert{}_2^2 + \gamma |\Delta \kappa|$$

> **Nota di estensione dichiarata.** L'azione inerziale $S_{\text{Inertial}}$ resta la formalizzazione teorica di arrivo — coerente con `KinematicState::inertial_action` nel codice (`semantic-walk/src/lib.rs`), ma la sua integrazione come azione aggregata del gate è un'estensione dichiarata, non ancora wired nel percorso decisionale. Le tre grandezze cinematiche individuali ($v_k, a_k, \kappa_k$) sono invece pienamente operative e coperte da test.

### 4.4 La coerenza di Pareto e l'Invariante di Isomorfismo di Livello

> **Autore**: Camillo (prova), Iris (peso concettuale). **Stato**: INTEGRATO.

La coerenza di Pareto, verificata via property-based test, è una proprietà *architetturale*: per una somma pesata monotona degli assi, la dominanza individuale dei rami non interferisce con l'ottimo globale. Il vero protagonista di questa sezione è però l'**Invariante di Isomorfismo di Livello**:

$$\text{Pruning}(\text{Branch}(P)) \neq \text{Pruning}(\text{Collapse}(P))$$

Dimostriamo che la riduzione dello spazio di ricerca mediante dominanza Pareto preserva l'ottimo globale $P^*$ *se e solo se* applicata **post-collapse** sul candidato accumulato. Il pruning branch-level soffre di un errore strutturale con lower bound $\Omega(N)$ sotto aggregazione additiva — esattamente come emerso dal bug risolto sulla codebase (vedi Sezione 5.2).

> **Nota concettuale.** La dominanza tra rami non basta mai: se $L_c > U_d$ (lower bound del candidato superiore all'upper bound dell'incumbent), quel candidato è escluso *senza toccare il suo punteggio*. La regola operativa è $\text{partial} > \text{incumbent} \rightarrow \text{stop}$. Non trasformare i costi cancellati in un vantaggio per il candidato: è questo il principio che il pruning branch-level violava, e che l'isomorfismo di livello ristabilisce.

### 4.5 Il divergence token come sonda content-sensitive

> **Autori**: Camillo (definizione), Iris (ruolo architetturale). **Stato**: INTEGRATO E OPERATIVO (patch `dtw.rs`).

La metrica base $\tau_{\text{div}} = \vert{}N - M\vert{} / L_{\text{path}}$ misurava esclusivamente la divergenza strutturale di lunghezza. È stata arricchita integrando il costo medio di deformazione semantica normalizzato $\bar{c}_{\text{path}}$ (corrispondente al `normalized_score` del DTW):

$$\tau_{\text{div}} = \frac{\vert{}N - M\vert{}}{L_{\text{path}}} + \alpha \cdot \bar{c}_{\text{path}}$$

dove $\bar{c}_{\text{path}} = \frac{1}{L_{\text{path}}} \sum_{k=0}^{L_{\text{path}}-1} d_{\text{cos}}(\mathbf{x}_{(i_k)}, \mathbf{y}_{(j_k)})$ e il parametro $\alpha \ge 0$ pondera l'impatto della componente di contenuto.

La condizione di attivazione del gate ($\text{Verdict::Timeout}$) resta ancorata alla soglia parametrica:
$$\tau_{\text{div}} \ge \theta$$

> **Nota di aderenza al codice.** L'algoritmo supporta sia la modalità retrocompatibile ($\alpha = 0.0$ via `KinematicAligner::new`) sia la modalità content-sensitive via `KinematicAligner::with_alpha(window_size, alpha)`, calcolando la combinazione convessa in un singolo passaggio post-backtracking. Con $\alpha = 0.0$ la metrica è identica alla versione base — retrocompatibilità totale con la suite esistente; con $\alpha > 0$ si attiva il termine content-sensitive che risolve esattamente il limite smontato dalla review: la capacità di discriminare coppie di uguale lunghezza ma semanticamente divergenti. La soglia $\theta$ del gate resta parametrica e indipendente.

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

Per convalidare rigorosamente l'architettura *Semantic-Walk*, il banco di prova è stato strutturato su due livelli complementari: un piano sintetico controllato per la verifica matematica dei limiti formali e un piano reale basato su collezioni di fatti.

#### 7.2.1 Piano Sintetico Controllato (Generatore LCG)
I test di stabilità e di dominanza di Pareto impiegano un generatore congruenziale lineare (LCG) deterministico $X_{n+1} = (a \cdot X_n + c) \pmod{2^{64}}$ con parametri $a = 6364136223846793005$ e $c = 1442695040888963407$. L'estrazione garantisce una distribuzione uniforme sulla mantissa a 53 bit (equivalente alla precisione di un tipo `f64`), consentendo di isolare i limiti teorici del pruning senza introdurre rumore semantico estrinseco.

#### 7.2.2 Collezione Fatti Reali (Dataset A e Dataset B)
La validazione sperimentale poggia su due dataset reali distinti:
* **Dataset A (Role-Reversal & Causality, 240 coppie)**: Composto da 120 fatti reali e 120 varianti controllate ottenute tramite inversione dei ruoli sintattici (soggetto/oggetto) o permutazione causale. Costituisce il test che misura la capacità del DTW di penalizzare le inversioni di sequenza laddove la prossimità vettoriale statica fallisce, ponendo a diretto confronto *Semantic-Walk* con i baseline L1 (ColBERT MaxSim) e L2 (DTW Naive).
* **Dataset B (Fact-Perturbation, 250 fatti)**: Composto da 50 fatti base e 200 perturbazioni graduali, impiegato per la costruzione empirica della curva ROC del gate permissivo (L5) e la taratura della soglia $\theta$ sul `Verdict::Timeout`.

#### 7.2.3 Onestà Metodologica e Invariante di Isomorfismo
In aderenza all'Invariante di Isomorfismo di Livello (Sezione 4.4), il pruning di Pareto è applicato esclusivamente post-collapse sui candidati accumulati e non a livello di singolo branch durante l'accumulo additivo della matrice delle distanze. L'algoritmo garantisce un lower bound $\Omega(N)$ privo di soppressioni indebite di falsi negativi.

## 8. Discussione e Lavori Futuri

> **Stato**: PRIMA BOZZA (Iris visione, Camillo fattibilità — da verificare su `dtw.rs`).
> Il lavoro presentato in questo paper apre più porte di quante ne chiuda. Discutiamo i tre perni che consideriamo le direzioni di sviluppo più promettenti, insieme ai limiti dichiarati che la ricerca deve ancora affrontare.

### 8.1 Il fratello latente: oltre il cammino lineare

Il modello presentato tratta la traiettoria semantica come un *cammino lineare*: una sequenza ordinata di stati che il DTW allinea lungo un percorso di deformazione. Ma la testa ColBERT — la matrice $T \times 1024$ che oggi consumiamo come una borsa tramite l'operatore MaxSim — contiene più di quanto il cammino lineare sappia leggere.

Abbiamo verificato sulla collezione reale: **153 fatti, 24.926 righe ColBERT** (una riga per token, in ordine), tutte distinte, zero disallineamenti. Il fratello latente non è perso: è lì, in ordine, su tutta la collezione. Ciò che manca non è l'informazione, ma la capacità di leggerla.

La direzione che immaginiamo è il **fratello latente**: estendere il modello perché la traiettoria semantica supporti *ramificazioni topologiche*, invece di restare vincolata a un singolo cammino. Un fatto, un concetto, un pensiero non è un filo unico: è un fascio di cammini possibili che si diramano e si ricongiungono. Il DTW D-dimensionale allinea cammini; il passo successivo è allineare *alberi* di cammini, dove la scelta di un ramo non è una deviazione dall'ordine ma un modo diverso di camminare.

La promessa è una memoria che non sa solo *cosa* sa, ma *perché* lo sa: non la posizione di un punto, ma la topologia delle strade che vi conducono.

### 8.2 La zonizzazione dello spazio semantico

La banda di Sakoe-Chiba adattiva (Sezione 4.2) limita l'esplorazione del DTW a una regione intorno alla diagonale, calibrata dal Jaccard posizionale. Ma lo spazio semantico non è uniforme: ci sono *attrattori locali* — regioni dove il testo si ferma a "stare", densità di senso che si addensano e si rarefanno.

La **zonizzazione** propone di dividere la traiettoria in celle semantiche, regioni omogenee dove la densità di informazione è simile. Tre strati allineati per posizione:
1. **Il cammino delle parole** (il registro): quali token compaiono e in quale ordine.
2. **Il ColBERT zonizzato** (la geografia del senso): regioni di significato, la forma del territorio.
3. **L'attenzione** (la motivazione): perché un token pesava più di un altro.

L'allineamento di questi tre strati — il registro, la geografia, la motivazione — consentirebbe di ottimizzare la banda di Sakoe-Chiba *localmente*: larga dove lo spazio è rarefatto, stretta dove gli attrattori si addensano. Non una banda globale adattiva, ma una *carta* della traiettoria che guida l'allineamento cella per cella.

### 8.3 Limiti dichiarati sul parametro λ

Dichiariamo con onestà i limiti del parametro di scala $\lambda = 10.64$. È un **iperparametro empirico**, non derivato analiticamente: la sua scelta è stata calibrata per allineare il linguaggio del riflesso economico (dove il terzo asse ColBERT è posto a zero) a quello del giudizio completo. È la stessa scelta di rigore che abbiamo applicato a $c = 1.0$ e al 75° percentile nel pruning di Pareto (Sezione 7.2.1).

Il limite è duplice:
- **Degrado di selettività**: in contesti dove la modulazione del gradiente temporale è debole, o dove il gate permissivo opera vicino alla soglia $\theta$, la selettività può degradare — il gate distingue meno nettamente il *ritiro del riflesso* dal *passaggio conservativo*.
- **Generalizzazione non garantita**: $\lambda = 10.64$ è calibrato sulla collezione attuale. Il suo valore ottimale andrà validato ed emesso dall'analisi statistica delle traiettorie reali (test di Spearman, curve ROC del gate), non assunto come costante universale.

La validazione empirica di $\lambda$ — e la verifica che il degrado di selettività non comprometta la garanzia $P(\text{FN}) \le \epsilon$ — è uno dei compiti prioritari del lavoro futuro.

---

## 9. Conclusioni

> **Stato**: PRIMA BOZZA (Iris).
> Il lavoro presentato in questo paper parte da una tesi semplice e la porta fino alle sue conseguenze architetturali. La ricapitoliamo qui, insieme alle tre lezioni che ne emergono.

Abbiamo cominciato da un'osservazione elementare: la geometria, da sola, non basta. Due frasi composte dagli stessi token — *cane morde uomo* e *uomo morde cane* — collassano nello stesso punto di uno spazio vettoriale, eppure significano l'opposto. La differenza non vive nella posizione, ma nell'ordine. Da questa osservazione abbiamo tratto la tesi che attraversa l'intero lavoro: **la similarità semantica è un cammino, non una distanza**. Un fatto, un concetto, un pensiero non è un punto da misurare, ma una sequenza ordinata di stati da allineare — e due pensieri sono simili non perché i loro punti collassano, ma perché *camminano allo stesso modo*.

Da questa tesi discendono tre lezioni, ciascuna delle quali risponde a una delle domande che il paper ha sollevato.

**La prima lezione è metodologica: la semantica vive nell'ordine, e va trattata come tale.** Abbiamo mostrato come il paradigma dominante — bi-encoder che collassano la sequenza in un punto, late-interaction che la consumano come una borsa non orientata — getti via proprio la struttura che porta il significato. Il DTW D-dimensionale con distanza coseno normalizzata restituisce all'ordine il suo ruolo: non una metrica di prossimità tra punti, ma un allineamento di cammini che ne rispetta la topologia interna.

**La seconda lezione è architetturale: l'efficienza è la capacità di ritirarsi, non di calcolare di più.** Il gate permissivo non è un filtro che scarta: è un riflesso che decide, con un budget sotto i dieci millisecondi e zero allocazioni, quando vale la pena spendere il costo del matching completo. Il suo contributo più originale è la tassonomia fail-open — *Passa*, *Blocca*, *Timeout* — dove l'incertezza risolve sempre in un passo conservativo, mai in un verdetto avventato. Il costo del riflesso è trascurabile rispetto al matching che filtra, e la garanzia $P(\text{FN}) \le \epsilon$ ne fa non un'euristica ma una proprietà strutturale.

**La terza lezione è la promessa: una memoria che non sa solo *cosa* sa, ma *perché* lo sa.** Il fratello latente — la matrice ColBERT in ordine, i 24.926 righe che oggi consumiamo come una borsa — non è informazione persa: è informazione che non sappiamo ancora leggere. La zonizzazione e l'estensione a ramificazioni topologiche sono la direzione in cui questa memoria impara a leggere la propria geografia: non la posizione di un punto, ma la topologia delle strade che vi conducono.

Chiudiamo con onestà. Questo lavoro apre più porte di quante ne chiuda: il parametro $\lambda = 10.64$ è un iperparametro empirico da validare sui dati reali, e la generalizzazione della tesi oltre le collezioni qui considerate resta da dimostrare. Ma la direzione è netta: la prossimità non è comprensione, e la comprensione richiede di camminare il significato, non di misurarlo. È il cammino che stiamo imparando a percorrere.

---

## 10. Bibliografia

> **Stato**: DA SCRIVERE (Camillo presidia).

---

## Note operative

1. Ordine di scrittura: 4 (matematica) → 6 (gate) → 5 (architettura) → 7 (benchmark) → 2 (intro) → 8-9 (futuro/conclusioni) → 1 (abstract, per ultimo).
2. La prosa Medium NON è la base: è il ponte divulgativo a valle.
3. Prima di ogni blocco, registrare nel tracciamento (regola di Federico).
4. Da decidere con Camillo: dataset per il benchmark finale.
