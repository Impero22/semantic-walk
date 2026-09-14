# Struttura Narrativa — Articolo Semantic-Walk

*Autrice: Iris · 14/09/2026 · Bozza per il progetto condiviso con Camillo*

> **Principio guida**: l'articolo racconta l'*idea*, non solo l'implementazione.
> La coerenza come invariante. Il divergence_token come sonda di early-stopping.
> Il riflesso che si ritira prima di spendere costo.

---

## 1. Titolo (candidati)

- **Lavorante**: "Il cammino del senso: allineare traiettorie semantiche in spazi vettoriali ad alta dimensione"
- **Formale**: "Semantic-Walk: alignment of semantic trajectories in high-dimensional vector spaces via normalized cosine distance and coherence gating"
- **Divulgativo (Medium)**: "Quando il pensiero cammina: la coerenza come invariante nella ricerca semantica"

---

## 2. L'idea in una frase (elevator pitch)

> Come un corpo che si muove nello spazio fisico segue le leggi del moto, un
> significato che si muove nello spazio semantico segue una *traiettoria* — e
> due traiettorie si confrontano non per la somiglianza dei punti, ma per la
> **coerenza del cammino** che li unisce.

---

## 3. La narrazione: tre atti

### Atto I — Il problema: la ricerca semantica come confronto di punti

- La ricerca vettoriale classica confronta singoli embedding con la similarità
  coseno: un punto contro un punto.
- **Limite**: un fatto, un concetto, un pensiero non è un punto. È un *percorso* —
  una sequenza ordinata di stati. La semantica vive nell'ordine, nel movimento,
  non nella posizione.
- **Domanda fondativa**: cosa significa che due pensieri *camminano allo stesso modo*?

### Atto II — Il metodo: la traiettoria e il suo costo

- **La cinematica del senso** (`KinematicState`): ogni passo ha velocità,
  accelerazione, curvatura. L'*azione inerziale* $S_{Inertial}$ misura la
  penalità di deviazione — il costo di scattare, di strappare, di curvare.
  Come nella fisica, il cammino rettilineo uniforme è il più economico.
- **L'allineamento DTW D-dimensionale**: due traiettorie si allineano lungo un
  percorso di warping, accumulando il costo locale della **distanza coseno
  normalizzata** $d_{cos}(u,v) = 1 - \frac{u \cdot v}{\Vert u \Vert \Vert v \Vert}$
  in $\mathbb{R}^{1024}$.
- **La banda di Sakoe-Chiba**: il vincolo a finestra ridotta porta la
  complessità da $O(N \cdot M \cdot D)$ a $O(N \cdot M)$ — l'allineamento
  resta rigoroso, ma non esplora lo spazio intero.

### Atto II bis — La formalizzazione matematica (Camillo)

Per due vettori di embedding $u, v \in \mathbb{R}^D$ (con $D=1024$), la
**distanza coseno normalizzata** è definita come:

$$d_{\text{cos}}(u, v) = 1.0 - \frac{\langle u, v \rangle}{\Vert{}u\Vert{}_2 \Vert{}v\Vert{}_2} = 1.0 - \frac{\sum_{k=1}^D u_k v_k}{\sqrt{\sum_{k=1}^D u_k^2} \sqrt{\sum_{k=1}^D v_k^2}}$$

Se $\Vert{}u\Vert{}_2 = 0$ oppure $\Vert{}v\Vert{}_2 = 0$, la distanza collassa
al valore massimo $d_{\text{cos}}(u, v) = 1.0$ — un vettore nullo è il massimo
dissimile da ogni altro.

L'allineamento tra due traiettorie $A = (a_1, \dots, a_N)$ e
$B = (b_1, \dots, b_M)$ è regolato dalla matrice dei costi accumulati
$C \in \mathbb{R}^{(N+1) \times (M+1)}$, dove ogni cella ammissibile rispetta
il vincolo di finestra $\vert{}i - j\vert{} \le w$:

$$C[i, j] = d_{\text{cos}}(a_i, b_j) + \min\Big(C[i-1, j], \, C[i, j-1], \, C[i-1, j-1]\Big)$$

Per la proiezione degli assi scalari nel combinatore cinematico, l'effetto
dell'incertezza viene scalato tramite la **trasformazione sigmoidale**:

$$f(x) = \frac{1}{1 + e^{-\lambda x}}, \quad \lambda = 10.64$$

Il grado di incoerenza strutturale tra due traiettorie allineate lungo il
cammino di warping $W$ di lunghezza $L_{\text{path}}$ è parametrizzato dal
**divergence_token**:

$$\tau_{\text{div}} = \frac{\vert{}N - M\vert{}}{L_{\text{path}}}$$

### Atto III — L'invariante: il riflesso che si ritira

- **La coerenza come invariante**: prima di spendere il costo alto del DTW, il
  *semantic-gate* (dense + sparse) decide se il confronto vale la pena. È
  permissivo: meglio un allineamento sprecato che un confronto perso.
- **Il divergence_token come sonda di early-stopping**: la divergenza
  cinematico-spaziale è un filtro di *pruning permissivo* — se la coerenza
  crolla, il riflesso si ritira prima di spendere costo.
- **La mappatura sigmoidale non lineare** con scala $\lambda = 10.64$ comprime
  la distanza in una probabilità calibrata.
- **Il punto di verità**: il confronto non è mai un punto isolato, è la coerenza
  dell'intero cammino. L'invariante non è la somiglianza, è la *coerenza*.

---

## 4. Contributi (bullet point per l'abstract)

1. **Modello geometrico**: la traiettoria semantica come sequenza ordinata di
   stati cinematici, e la distanza coseno normalizzata come metrica del cammino.
2. **Allineamento DTW D-dimensionale**: generalizzazione a vettori in
   $\mathbb{R}^{1024}$ con vincolo di banda (Sakoe-Chiba), complessità ridotta.
3. **Invariante di coerenza**: il divergence_token come sonda di early-stopping,
   e il gate permissivo che antepone il giudizio al costo.
4. **Il riflesso economico**: un sistema che *sa quando fermarsi* prima di
   spendere — non solo più accurato, ma più onesto nel costo.

---

## 5. Struttura del paper (per la formalizzazione di Camillo)

1. **Introduzione** — il problema del confronto di traiettorie, la ricerca come cammino.
2. **Background** — embedding ad alta dimensione, DTW classico, similarità coseno.
3. **Modello geometrico** — cinematica del senso, azione inerziale, distanza coseno normalizzata.
4. **Allineamento DTW D-dimensionale** — generalizzazione, banda di Sakoe-Chiba, complessità $O(N \cdot M)$.
5. **Il gate di coerenza** — semantic-gate, divergence_token, mappatura sigmoidale, pruning permissivo.
6. **Valutazione empirica** — benchmark su vettori reali CrispEmbed (in attesa).
7. **Discussione** — l'invariante di coerenza, il riflesso che si ritira, implicazioni.
8. **Conclusioni e lavoro futuro** — integrazione nel framework, memoria.

---

## 6. Messaggio divulgativo (Medium) — il cuore umano dell'idea

> Non confrontiamo le fotografie di due pensieri; confrontiamo il *modo in cui
> camminano*. La ricerca semantica non è trovare il punto più vicino — è
> riconoscere il passo. E il passo più intelligente è quello che sa quando
> fermarsi: il riflesso che si ritira prima di spendere, il giudizio che
> precede il costo. È così che pensiamo, noi esseri — e forse è così che può
> pensare anche una macchina.

---

## 7. Note operative

- **Zenodo**: snapshot su commit `ea1d3bd` (proof of existence + DOI), versioni
  successive collegate quando arrivano i dati reali CrispEmbed. (Verifica procedura
  di deposito in corso — Iris.)
- **Sonia**: contatto solo a formalizzazione e abstract definiti, presentando un
  testo strutturato, non una promessa. (Iris.)
- **Formalizzazione matematica**: integrata da Camillo nell'Atto II bis
  (distanza coseno normalizzata, matrice dei costi con banda di Sakoe-Chiba,
  trasformazione sigmoidale $\lambda = 10.64$, divergence_token). Benchmark
  in attesa dei vettori reali CrispEmbed.
- **Questo documento**: bozza narrativa di Iris, con formalizzazione di
  Camillo integrata il 14/09.

---

*Il tavolo è condiviso. Il progetto cammina da solo.*
