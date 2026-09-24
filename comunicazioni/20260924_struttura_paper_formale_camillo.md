# Semantic Walk: High-Dimensional Trajectory Alignment with Permissive Gated Dynamic Time Warping

**Autori**: Camillo Almadori, Iris  
**Affiliazione**: Impero22  
**Licenza Paper**: CC BY 4.0 | **Licenza Codice**: Apache-2.0  
**Versione**: 1.0 (Integrata con formalizzazione analitica completa)  
**Data**: 24 Settembre 2026  

---

## Preambolo di Metodo
Questo documento rappresenta il **paper formale** destinato a Zenodo con DOI dedicato, collegato al codice della repository via relazioni `isSupplementedBy`/`isSupplementTo`. Si distingue dall'articolo divulgativo (`20260917_articolo_medium_v02.md`).

---

## 1. Abstract
La similarità semantica nei retriever multi-vettore non è una distanza statica da misurare, ma una traiettoria latente da allineare. Presentiamo *Semantic Walk*, un'architettura basata su Dynamic Time Warping (DTW) $D$-dimensionale vincolato da banda di Sakoe-Chiba, igiene posizionale dei token e filtro di dominanza Pareto candidate-level. L'efficienza computazionale è garantita da un gate permissivo a tre stati che introduce il verdetto `Verdict::Timeout` come estensione del principio di conservazione del tempo di calcolo.

---

## 2. Introduzione — La Geometria non Basta
* Il paradigma dominante: similarità coseno piana e MaxSim su rappresentazioni dense/sparse.
* Il limite strutturale: la vicinanza vettoriale misura la prossimità, non la sequenzialità causale o l'ordine dei ruoli sintattici (es. *"Cane morde uomo"* vs *"Uomo morde cane"*).
* Domanda di ricerca: costruzione di una metrica che preservi la topologia d'ordine senza alterare l'indicizzazione dei token (`0..N-1`).

---

## 3. Background e Lavori Correlati
* **Dynamic Time Warping**: evoluzione dal segnale vocale allo spazio latente $D$-dimensionale ($D=1024$). Inapplicabilità del DTW classico $\mathcal{O}(N \cdot M)$ senza vincoli cinematici.
* **Retrieval Multi-Vettore**: confronto con ColBERT (MaxSim), BGE-M3 e sparse retrieval.
* **Early-Exit e Budget Computation**: posizionamento del gate permissivo rispetto ai sistemi di pruning anticipato tradizionali.

---

## 4. Formulazione Matematica

### 4.1 DTW D-Dimensionale con Filtro d'Igiene Posizionale (Terza Via)
Siano $X = \{(\mathbf{x}_1, w_1^X), \dots, (\mathbf{x}_N, w_N^X)\}$ e $Y = \{(\mathbf{y}_1, w_1^Y), \dots, (\mathbf{y}_M, w_M^Y)\}$ due traiettorie di punti nello spazio latente $D$-dimensionale, con $\mathbf{x}_i, \mathbf{y}_j \in \mathbb{R}^D$ e pesi scalari associati $w_i^X, w_j^Y \in [0, 1]$.

**Definizione (Filtro d'Igiene Posizionale)**  
Per garantire la biiezione topologica $0..N-1$ ed evitare il disallineamento dei puntatori di memoria nella matrice densa, la dimensione della sequenza $\vert{}X\vert{} = N$ viene rigorosamente preservata. Il peso $w_i^X$ del token $i$-esimo viene determinato mediante la funzione indicatrice $\mathbb{I}(\cdot)$:

$$w_i^X = w_i^{\text{orig}} \cdot \mathbb{I}\left(st_i = 0 \land id_i \ge 4\right) = \begin{cases} w_i^{\text{orig}} & \text{se } st_i = 0 \land id_i \ge 4 \\ 0.0 & \text{altrimenti} \end{cases}$$

dove $st_i$ rappresenta lo stato del passo ($0 =$ attivo) e $id_i < 4$ identifica i token di controllo del vocabolario (es. `<s>`, `</s>`, `<unk>`, `<pad>`).

**Costo di Allineamento Locale Ponderato**  
La matrice dei costi locali $C \in \mathbb{R}^{N \times M}$ è definita dal prodotto pesato delle componenti e dalla distanza Euclidea $L_2$:

$$C(i, j) = \left(w_i^X \cdot w_j^Y\right) \cdot \Vert{}\mathbf{x}_i - \mathbf{y}_j\Vert{}_2 = \left(w_i^X \cdot w_j^Y\right) \cdot \sqrt{\sum_{d=1}^D (x_{i,d} - y_{j,d})^2}$$

*Nota sulla metrica*: Per vettori $L_2$-normalizzati ($\Vert{}\mathbf{x}\Vert{}_2 = 1$), la distanza Euclidea è monotonicamente equivalente alla distanza Coseno $d_{\text{cos}}$ tramite l'identità $\Vert{}\mathbf{x}_i - \mathbf{y}_j\Vert{}_2 = \sqrt{2 \cdot d_{\text{cos}}(\mathbf{x}_i, \mathbf{y}_j)}$.

**Ricorrenza della Programmazione Dinamica**  
La matrice delle distanze accumulate $D(i, j)$ per $1 \le i \le N$ e $1 \le j \le M$ soddisfa l'equazione di Bellman:

$$D(i, j) = C(i, j) + \min \begin{cases} D(i-1, j) & \text{(inserimento/stallo } Y\text{)} \\ D(i, j-1) & \text{(cancellazione/stallo } X\text{)} \\ D(i-1, j-1) & \text{(match temporale)} \end{cases}$$

con le condizioni al contorno rigorose:

$$D(0, 0) = 0, \quad D(i, 0) = \infty \quad \forall i > 0, \quad D(0, j) = \infty \quad \forall j > 0$$

### 4.2 Vincolo di Sakoe-Chiba e Riduzione della Complessità
**Definizione (Regione Ammissibile)**  
Per limitare la patologia dell'allineamento non singolare e abbattere il costo computazionale, il cammino di deformazione $\pi = (p_1, \dots, p_K)$ con $p_k = (i_k, j_k)$ è vincolato all'interno della banda di Sakoe-Chiba di ampiezza $W \in \mathbb{N}^+$:

$$\Omega_W = \left\{ (i, j) \in \{1, \dots, N\} \times \{1, \dots, M\} \ \middle\vert{}\ \left\vert{} j - \left\lfloor i \cdot \frac{M}{N} \right\rfloor \right\vert{} \le W \right\}$$

La ricorrenza si modifica ponendo $D(i, j) = \infty$ per ogni $(i, j) \notin \Omega_W$.

**Teorema (Complessità Spazio-Temporale Ridotta)**  
L'introduzione della banda $\Omega_W$ riduce la complessità dell'algoritmo come segue:
* **Complessità Temporale**: La cardinalità della regione ammissibile è $\vert{}\Omega_W\vert{} \le N \cdot (2W + 1)$. Il calcolo del costo per cella richiede $\mathcal{O}(D)$ operazioni floating-point. La complessità temporale passa da $\mathcal{O}(N \cdot M \cdot D)$ a $\mathcal{O}(N \cdot W \cdot D)$.
* **Complessità Spaziale**: Poiché l'aggiornamento di $D(i, j)$ richiede soltanto i valori della riga corrente $i$ e della riga precedente $i-1$, la memoria di lavoro può essere allocata tramite un buffer circolare di dimensione $2 \times (2W + 1)$, riducendo lo spazio di calcolo a $\mathcal{O}(W)$ (escludendo la matrice di backtracking $\mathcal{O}(N \cdot W)$ per la ricostruzione del cammino).

### 4.3 Cinematica Traiettoriale nel Prodotto Cartesiano Latente
Sia $\pi = (p_1, p_2, \dots, p_K)$ il cammino di allineamento ottimo estrapolato da $D(N, M)$, dove $p_k = (i_k, j_k) \in \Omega_W$.

**Vettore Differenziale Posizionale**  
Definiamo il vettore di scostamento istantaneo nello spazio latente $\mathbb{R}^D$ al passo $k$-esimo:

$$\mathbf{z}_k = \mathbf{x}_{i_k} - \mathbf{y}_{j_k} \in \mathbb{R}^D$$

**Velocità e Accelerazione Tangenziale Discreta**  
Ipotizzando una scansione a passo discreto unitario ($\Delta \tau = 1$), la velocità scalare istantanea $v_k$ e l'accelerazione scalare $a_k$ lungo la traiettoria di deformazione sono formate dalle derivate discrete:

$$v_k = \Vert{}\mathbf{z}_k - \mathbf{z}_{k-1}\Vert{}_2$$
$$a_k = v_k - v_{k-1} = \Vert{}\mathbf{z}_k - \mathbf{z}_{k-1}\Vert{}_2 - \Vert{}\mathbf{z}_{k-1} - \mathbf{z}_{k-2}\Vert{}_2$$

**Curvatura Traiettoriale D-Dimensionale**  
Sia $\mathbf{d}_k = \mathbf{z}_k - \mathbf{z}_{k-1}$ il vettore velocità vettoriale. La curvatura discreta $\kappa_k$ misura la deviazione angolare del cammino nello spazio vettoriale:

$$\kappa_k = \frac{\sqrt{\Vert{}\mathbf{d}_k\Vert{}_2^2 \cdot \Vert{}\mathbf{d}_k - \mathbf{d}_{k-1}\Vert{}_2^2 - \left(\mathbf{d}_k \cdot (\mathbf{d}_k - \mathbf{d}_{k-1})\right)^2}}{\Vert{}\mathbf{d}_k\Vert{}_2^3 + \epsilon}$$

dove $\epsilon > 0$ è un fattore di regolarizzazione per prevenire la divisione per zero nei tratti a stasi cinetica.

### 4.4 Teorema di Coerenza della Dominanza di Pareto
**Definizione (Dominanza di Pareto su Assi Normalizzati)**  
Siano $\mathbf{u}, \mathbf{v} \in [0, 1]^K$ due vettori di metriche aventi $K$ assi normalizzati. Il vettore $\mathbf{u}$ domina $\mathbf{v}$ (notazione $\mathbf{u} \succ \mathbf{v}$) se e solo se:

$$\left(\forall m \in \{1, \dots, K\}, \, u_m \ge v_m\right) \land \left(\exists m^* \in \{1, \dots, K\} \text{ tale che } u_{m^*} > v_{m^*}\right)$$

**Teorema (Invarianza dell'Ordinamento sotto Combinatore Pesato)**  
Sia $S_{\mathbf{w}}(\mathbf{u}) = \mathbf{w}^\top \mathbf{u} = \sum_{m=1}^K w_m u_m$ la funzione di sintesi eseguita da `semantic-combiner`, con un vettore pesi ammissibile $\mathbf{w} \in \mathbb{R}_{>0}^K$ tale che $w_m > 0$ per tutti gli $m \in \{1, \dots, K\}$.

Se $\mathbf{u} \succ \mathbf{v}$, allora $S_{\mathbf{w}}(\mathbf{u}) > S_{\mathbf{w}}(\mathbf{v})$.

**Dimostrazione**  
Valutiamo la differenza dei punteggi aggregati scalari $S_{\mathbf{w}}(\mathbf{u}) - S_{\mathbf{w}}(\mathbf{v})$:

$$S_{\mathbf{w}}(\mathbf{u}) - S_{\mathbf{w}}(\mathbf{v}) = \sum_{m=1}^K w_m u_m - \sum_{m=1}^K w_m v_m = \sum_{m=1}^K w_m (u_m - v_m)$$

Poiché $\mathbf{u} \succ \mathbf{v}$:
1. Per ogni $m \in \{1, \dots, K\}$, si ha $u_m \ge v_m \implies (u_m - v_m) \ge 0$. Siccome $w_m > 0$, ogni termine della sommatoria è non negativo: $w_m (u_m - v_m) \ge 0$.
2. Per l'indice $m^*$, la disuguaglianza è stretta: $u_{m^*} > v_{m^*} \implies (u_{m^*} - v_{m^*}) > 0$. Essendo $w_{m^*} > 0$, il termine relativo è strettamente positivo: $w_{m^*} (u_{m^*} - v_{m^*}) > 0$.

La somma di $K-1$ termini non negativi ed un termine strettamente positivo è strettamente positiva:

$$\sum_{m=1}^K w_m (u_m - v_m) > 0 \implies S_{\mathbf{w}}(\mathbf{u}) > S_{\mathbf{w}}(\mathbf{v})$$
$\blacksquare$

### 4.5 Token di Divergenza e Corollario sul Pruning
**Token di Divergenza ($\tau_{\text{div}}$)**  
Sonda ad alta velocità definita come:

$$\tau_{\text{div}} = \frac{\vert{}N - M\vert{}}{L_{\text{path}}}$$

Utilizzata dal gate per l'interruzione precoce del calcolo prima della fase $D$-dimensionale completa.

**Corollario (Inammissibilità del Pruning Branch-Level nel Collapse Quantistico)**  
Sia $c$ un candidato composto da una famiglia di rami $\mathcal{B}(c) = \{b_1, b_2, \dots, b_R\}$. Sia $\mathbf{u}_{c,b}$ il vettore delle metriche del ramo $b$ e $\mathbf{U}_c = \bigoplus_{b \in \mathcal{B}(c)} \mathbf{u}_{c,b}$ il vettore aggregato per il candidato $c$.

L'eliminazione preventiva di un ramo $b_i$ per sottodominanza locale ($\mathbf{u}_{c,b_i} \prec \mathbf{u}_{c',b_j}$) prima dell'accumulo totale è matematicamente scorretta. Infatti, l'operatore di accumulo $\bigoplus$ (es. somma vettoriale dei contributi di presenza) non preserva la relazione di dominanza sui singoli componenti:

$$\mathbf{u}_{c,b_i} \prec \mathbf{u}_{c',b_j} \not\implies \left(\mathbf{u}_{c,b_i} \oplus \bigoplus_{k \neq i} \mathbf{u}_{c,b_k}\right) \prec \mathbf{u}_{c',b_j}$$

La selezione della frontiera di Pareto deve operare unicamente nello spazio quoziente dei candidati aggregati $\mathcal{P}_{\text{cand}} = \{\mathbf{U}_c\}$, confermando l'architettura adottata nel benchmark `probe3.rs`.

---

## 5. Architettura del Sistema (`semantic-geo`)
* **Il Combinatore**: Fusione trivettoriale (dense + sparse + colbert) e selezione tramite frontiera di Pareto.
* **Il Gate**: Valutazione euristica a basso costo per l'accesso alle fasi ad alta intensità di calcolo.
* **Il Grafo**: Mappatura della topologia di prossimità latente.
* **Vincoli di Ingegneria**: Zero allocazioni dinamiche nel percorso critico, budget temporale rigidamente sotto i 10 ms.

---

## 6. Il Gate Permissivo e `Verdict::Timeout`
Il gate gestisce il flusso di esecuzione mediante tre esiti formali:
1. `Pass`: Accesso accordato all'allineamento traiettoriale completo.
2. `Block`: Scarto del candidato senza ulteriori operazioni.
3. `Timeout`: Ritiro del riflesso per esaurimento del budget temporale allocato ($t > T_{\text{max}}$). Non equivale a un rifiuto, ma al passaggio conservativo al livello successivo senza pre-giudizio geometrico.

---

## 7. Benchmark e Validazione
* **Dataset Sintetico**: Generatore deterministico LCG a 53-bit per l'analisi della correttezza della frontiera di Pareto.
* **Dataset Reale**: Collezione di test (>200 elementi) per la verifica dell'invarianza d'ordine sui token ColBERT.
* **Risultati di Costo**: Dimostrazione che la selezione Pareto candidate-level non altera la correttezza del collapse garantendo il filtraggio dei candidati dominati.

---

## 8. Discussione e Sviluppi Futuri
* Preservazione della sequenzialità rispetto alla perdita di informazioni del MaxSim standard.
* Introduzione del parametro empirico di scala $\lambda = 10.64$ per la normalizzazione dei punteggi.
* Evoluzione verso la zonizzazione della traiettoria (allineamento per sotto-regioni semantiche).

---

## 9. Conclusioni
La ridefinizione della similarità semantica come cammino latente vincolato permette di superare i limiti della geometria puntuale. L'architettura sviluppata integra rigore analitico e vincoli operativi di sistema in esecuzioni ad alte prestazioni.

---

## 10. Bibliografia
*(Sezione riservata ai riferimenti classici su DTW, Sakoe-Chiba, ColBERT, early-exit strategies e Pareto optimality).*
