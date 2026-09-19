# Confronto con lo Stato dell'Arte — semantic-walk workspace

**Data:** 19 settembre 2026  
**Committente:** Federico — regalo a Iris e Camillo per il posizionamento pre-pubblicazione  
**Metodologia:** Web research sistematica (2020–2026), analisi ~30 fonti primarie (arXiv, ACL Anthology, NeurIPS, ICML, SIGMOD, CIKM, ICLR), confronto strutturale con codice del workspace  
**Tono:** Collegiale, onesto, costruttivo — nessun giudizio di valore, solo posizionamento fattuale

---

## Executive Positioning Summary

| Innovazione | Verdetto novità | Prior art più vicino | Difendibilità |
|---|---|---|---|
| Trivector combiner | Ingegneria solida, non novel | Weighted Sum fusion (arXiv:2508.01405); indipendentemente atterrato sul loro ottimo FTS+SVS+DVS | Media — servono numeri BEIR vs RRF |
| Permissive gate + verdict taxonomy | **Novel nell'asimmetria** | Cascade rerankers; Self-RAG/CALM/Adaptive-RAG (tutti fail-closed o simmetrici) | **Alta** — serve curva recall/compute |
| Kinematic inertial action | **Novel transfer** (minimum-jerk robotics → IR) | Fused Gromov-Wasserstein (CVPR 2026) analogo formale più vicino | **Alta se ablation mostra gain**; zero se non lo mostra |
| Windowed DTW over ColBERT tokens | Novel domain application | DWA-KD (EACL 2026) fa Soft-DTW su token embeddings | Medio-Alta |
| Divergence token (τ_div) | Attualmente insound come specificato | — | **Bassa** — necessita ridefinizione |
| Quantum collapse | Mislabeled; stadio Pareto è sound | Boltzmann/softmax; Faithfulness-Aware Multi-Objective RAG (Dec 2025) | Media — reframe come energy-based |
| Cell zonizzazione | Discretizzazione è prior art (ColBERTv2/PLAID); uso ordinato è novel | ColBERTv2 residual compression + PLAID centroid interaction | Media — citare e posizionare esplicitamente |
| The bridge (dequantization) | Novel-ish (confidence-weighted) | RQ residual reconstruction | Media — MediaPesata potrebbe distruggere il segnale che serve |
| Hybrid kNN+threshold graph | Costruzione standard | NSG/Vamana/HNSW | Bassa come indice; Media come analytics layer |

**Headline:** Due contributi sono genuinamente unclaimed (fail-open gate taxonomy; order-preserving cell-walk fingerprint). Due sono ingegneria solida che eguaglia ma non eccede la practice. Due sono mislabeled e devono essere reframed. Uno è textbook (e va bene così).

---

## A. Semantic Retrieval & Reranking

### Mainstream (2024–2026)

**Bi-encoders:** E5-Mistral, GTE-Qwen2, BGE-M3, Qwen3-Embedding (con Matryoshka Representation Learning per dimensionalità adattiva). Cross-encoder rerankers: bge-reranker-v2-m3, Cohere Rerank 3.5, ZERANK-1. Late interaction: ColBERTv2 (Santhanam et al., NAACL 2022) su PLAID (CIKM 2022), pacchettizzato come RAGatouille, Jina-ColBERT-v2 (89 lingue), ModernColBERT (2025), PyLate. Learned sparse: SPLADE-v3, uniCOIL.

**Fusion:** Reciprocal Rank Fusion (Cormack et al., SIGIR 2009, κ=60) è il de-facto industry default — implementato in Elasticsearch 8.x, OpenSearch 2.x, Weaviate, Milvus 2.4+, Azure AI Search, Qdrant 1.9+.

### Cutting Edge

- **arXiv:2508.01405** — "An Experimental Analysis of Trade-offs in Hybrid Search" (2025). Primo studio sistematico: 15 combinazioni × 11 dataset. Findings chiave: (a) effetto "weakest link" — RRF può scendere sotto il best single path; (b) nessuna architettura universale — richiesta adaptive path selection; (c) TRF (late interaction su candidate set ridotto) consistentemente outperforms RRF e Weighted Sum. Identifica **FTS+SVS+DVS come balanced optimum**.

- **arXiv:2508.21038** — "On the Theoretical Limitations of Embedding-Based Retrieval" (Weller et al., Aug 2025). Sign-rank lower bound: per dimensione d esistono combinazioni top-k che nessuna query può restituire. A d=1024, ~4M docs. SOTA single-vector < 20 recall@100 su LIMIT; raccomandazione: multi-vector/cross-encoders.

- **arXiv:2606.28367** — "Do RAG Retrieval Enhancements Help Once a Strong Reranker Is Applied?" (2026). Re-evaluation scettica: il reranking forte assorbe gran parte del valore del retrieval enhancement.

### Posizione di semantic-walk

Il trivector combiner è una **score-aware Weighted Sum fusion** — la baseline "WS" in arXiv:2508.01405 — ma soddisfa la precondizione critica (scores propriamente normalizzati e comparabili) via normalizzazione esplicita per asse. Ha indipendentemente atterrato sulla configurazione **FTS+SVS+DVS** che lo studio identifica come "sweet spot."

### Punti di Forza

- Famiglia di fusion teoricamente superiore scelta (RRF è score-agnostic, soffre weakest-link)
- Tre assi = optimum empirico indipendente
- Zero-alloc, no GPU, deployable inside hot loop
- Pareto coherence come property-tested invariant → regression guarantee

### Debolezze

- **Pesi statici hand-calibrated** — nessun query-adaptive weighting (la raccomandazione centrale di 2508.01405)
- Nessuna informazione di rank (score-only fusion fragile a distribution shift)
- λ=10.64 satura rapidamente (>0.99 per s≥0.44 — verificare contro la scala reale degli sparse scores)
- Pareto-coherence invariant più debole di quanto sembri: qualsiasi monotone weighted sum con pesi non-negativi è trivially Pareto-coherent
- Nessun first-stage retrieval o indice
- Zero numeri BEIR/MTEB

### Da Verificare

1. Trivector-WS vs RRF vs learned fusion su subset BEIR (hotpotqa, nfsy, fever)
2. λ sweep contro distribuzioni sparse reali
3. Weight sensitivity (perturbazione ±20% → ΔnDCG)
4. Ablation asse colbert (il più costoso — contribuisce proporzionalmente al costo?)

### Direzioni Promettenti

- Query-adaptive weights pilotati dalle features del gate
- Il combiner può girare durante ANN descent (impossibile per cross-encoders)
- Score distribution monitoring per recalibration automatica

---

## B. Memory Systems for LLMs

### Mainstream

- **Mem0** (arXiv:2504.19413): due fasi Extract+Update con LLM tool-calls; ~66% su LOCOMO. **NOTA CRITICA:** repo spostato ad ADD-only (Apr 2026), abbandonando UPDATE/DELETE — l'LLM-in-the-loop non è sopravvissuto alla produzione.

- **Zep/Graphiti** (arXiv:2501.13956): temporal knowledge graph, modello bi-temporale (event timeline + transactional timeline), edge invalidation. Batte MemGPT: 94.8% vs 93.4% DMR.

- **MemGPT/Letta** (arXiv:2310.08560): OS-style paging, self-editing memory con interrupt-driven context management.

### Cutting Edge

- **HippoRAG 2** (ICML 2025): hippocampal indexing theory, OpenIE KG + Personalized PageRank per multi-hop
- **TSM** (arXiv:2601.07468, ACL Findings 2026): duration-aware temporal consolidation
- **Benchmark:** LongMemEval (ICLR 2025), LongMemEval-V2 (May 2026), LOCOMO
- **Graph-RAG:** Microsoft GraphRAG (Leiden communities), LightRAG, KG²RAG, G-RAG

### Posizione di semantic-walk

**ORTOGONALE, non in competizione.** Il mainstream risponde a "quale fatto è rilevante?" via point comparison + graph/temporal overlay. semantic-walk risponde a **"questi fatti si svolgono allo stesso modo?"** — retain intra-fact token order (T×1024) dove gli altri collassano a un singolo vettore.

### Punti di Forza

- **Order sensitivity** targetta un failure mode reale e documentato (fact vs negazione vs reverse-causal → vettori densi quasi identici; classe di fallimento LIMIT)
- **No LLM in hot path** — il retreat di Mem0 ad ADD-only valida questa scelta
- Deterministico, property-tested
- Divergence token dà localized disagreement — nessun sistema mainstream offre questo

### Debolezze

- Nessuna write-path semantics (ADD/UPDATE/DELETE, contradiction detection, consolidation, decay)
- Nessun modello temporale
- No multi-hop retrieval (nessuna espansione stile PageRank)
- Nessuna distinzione episodic/semantic memory
- Zero numeri benchmark (LOCOMO/LongMemEval)
- **Storage: problema footprint 100×** — T×1024 f32 ≈ 800KB/fatto vs 4–8KB mainstream. Cell-walk è la fix.

### Da Verificare

1. **LA CLAIM ORDER-SENSITIVITY È LOAD-BEARING E UNPROVEN** — costruire coppie avversariali (order swap, negation, agent-patient inversion), misurare separazione sotto dense cosine vs MaxSim vs DTW. Se DTW non batte MaxSim, il modello a traiettoria non ha valore dimostrato.
2. LOCOMO + LongMemEval con semantic-walk come retriever
3. Storage cost: cell-walk vs full trajectories vs single-vector

### Direzioni Promettenti

- Trajectory-aware consolidation (walk quasi identici = paraphrase → merge; τ_div alto in posizione = contraddizione localizzata → flag)
- Cell-walk come compact order-preserving fingerprint (400 bytes vs 800KB)
- Layering Zep-style bi-temporal invalidation sopra trajectory similarity

---

## C. Sequence Alignment in Embedding Spaces

### Mainstream

DTW (Sakoe & Chiba, 1978) standard in time-series, speech, gesture recognition. Speedup: FastDTW, PrunedDTW, EAPrunedDTW, UCR-SUITE. Lower bounds: LB_KIM, **LB_Keogh** (2002) — THE standard cheap pre-filter. In NLP: attention-based alignment, Word Mover's Distance (Kusner et al., ICML 2015), optimal transport.

### Cutting Edge

- **Soft-DTW** (Cuturi & Blondel, 2017): versione differenziabile
- **Deep Declarative DTW** (ICLR 2023): DTW come layer dichiarativo in reti profonde
- **DWA-KD** (EACL Findings 2026): Soft-DTW su token embeddings per knowledge distillation — evidenza diretta che DTW-over-tokens è area attiva
- **Fused Gromov-Wasserstein** (CVPR 2026): feature similarity + geometric structure simultaneamente

### Posizione di semantic-walk

Il motore DTW è **textbook windowed DTW con cosine cost** — nessuna novità, e va bene così. La novità sta in:

1. **Domain transfer** a sequenze di token ColBERT come operatore di retrieval
2. **Kinematic inertial action** come costo di smoothness ortogonale

L'antenato del termine cinematico è il **minimum-jerk model** del movimento umano del braccio (Flash & Hogan, 1985) e il jerk-limited trajectory planning in robotica. **Nessun lavoro IR/NLP che penalizza la derivata 2ª/3ª della traiettoria di embedding attraverso posizioni token è stato trovato** — il transfer appare unclaimed.

La combinazione (cosine DTW + inertial action) è strutturalmente un **hand-rolled, non-differentiable Fused Gromov-Wasserstein**.

### Punti di Forza

- Order-sensitive per costruzione (a differenza di MaxSim che è esplicitamente order-agnostic)
- Output localizzato e interpretabile via divergence token
- Deterministico, training-free, encoder-agnostic
- Zero-alloc bridge design

### Debolezze

- **No lower bound** — pruning stile LB_Keogh assente; ogni candidato gated paga il costo pieno
- **Non differenziabile** — α, β, γ, window possono solo essere cercati, mai appresi
- Window size è magic constant (nessuna procedura di selezione)
- **Costo:** N·(2w+1)·D per coppia; a N=M=128, w=8, D=1024 ≈ 2.2 MFLOPs/pair, 2.2 GFLOPs/query a 1000 candidati
- **τ_div = |N−M|/L_path** è un heuristic di length-mismatch, NON una misura di divergenza semantica (→0 per qualsiasi coppia di sequenze di uguale lunghezza indipendentemente dal contenuto)

### Da Verificare

1. DTW-cosine vs MaxSim vs DTW+inertial vs WMD sullo stesso corpus
2. **ABLATE kinematic term** — se nessun gain su pure DTW-cosine, inertial action è decorazione
3. ns reali per alignment vs LB_Keogh-pruned DTW
4. τ_div: discriminative power (distribuzione su matched vs unmatched pairs)
5. nDCG vs window size (sweep w ∈ {4, 8, 16, 32})

### Direzioni Promettenti

- **LB_Keogh su cell-walk representation** (u16, cheap, order-preserving) — il singolo addition a più alto valore, si compone con zonizzazione+bridge
- Kinematic pre-gate (velocity/acceleration/curvature summaries come scalari O(N) per rejection in nanosecondi)
- Divergence-token localization come UI primitive per explainability

---

## D. Graph-based Knowledge & Retrieval

### Mainstream

ANN indices: **HNSW** (Malkov & Yashunin, TPAMI 2020), **NSG** (Fu et al., VLDB 2019), **Vamana/DiskANN** (Subramanya et al., NeurIPS 2019), **ScaNN** (Guo et al., ICML 2020).

Graph-RAG: Microsoft GraphRAG (Edge et al., 2024 — Leiden community detection), LightRAG, KG²RAG, StructRAG. **HippoRAG 2** (ICML 2025) con Personalized PageRank è il più forte operatore di graph-memory retrieval pubblicato.

### Cutting Edge

- **HENN** (OpenReview 2025): prime garanzie teoriche per famiglia HNSW
- **Tagore** (2025): GPU-accelerated graph construction
- **ANN Search: Recall What Matters** (2026): survey comprensiva

### Posizione di semantic-walk

Un **grafo di prossimità SEMANTICA** su fatti, non un indice ANN. Lo scopo differisce: i grafi ANN rendono la ricerca sublineare; semantic-graph espone descrittori strutturali come segnali semantici — più vicino al "what is the shape of this memory?" di GraphRAG.

### Punti di Forza

- Hybrid kNN+threshold più robusto di ciascuna strategia pura per corpus eterogenei
- Metriche strutturali come semantic features sotto-sfruttate (orphans = memorie non consolidate; centrality = hub concettuali)
- Clustering coefficient per rilevare comunità locali senza Leiden

### Debolezze

- Nessuna garanzia di navigabilità (il grafo può essere disconnesso)
- Nessun algoritmo di ricerca sul grafo
- No incremental insertion (rebuild globale)
- No RNG/α-pruning diversity step (formazione di hub)
- No community detection
- No Personalized PageRank

### Da Verificare

1. Degree distribution, connected components, orphan fraction su corpus reale
2. Recall vs k e threshold
3. Greedy search vs HNSW sullo stesso grafo
4. Aggiungere degree/centrality al combiner migliora nDCG?

### Direzioni Promettenti

- Personalized PageRank seeded dai candidati che passano il gate
- Orphan/clustering come consolidation trigger
- Cell-walk similarity come edge weight (più cheap + encodes order — nessun grafo ANN lo fa)

---

## E. Quantum-Inspired & Multi-Objective Optimization

### Mainstream

Multi-objective reranking: scalarization, Pareto, DPP (diversity), MMR (Carbonell & Goldstein, 1998).

**Quantum COGNITION in IR** è una letteratura genuina e decades-old: van Rijsbergen (ICTIR 2009), survey arXiv:2007.04357, Bell-like tests (Entropy, 2024). Claim centrale: non-commutativity e **interference** spiegano relevance judgements che violano la probabilità classica.

**CERN/CEPC (Nov 2025):** classical quantum-inspired (bSB) significativamente outperforms QAOA e quantum annealers reali — il "quantum advantage" in ottimizzazione è sotto pressione empirica.

### Cutting Edge

- **PreferRec** (Mar 2026): Pareto preference transfer per raccomandazione
- **Faithfulness-Aware Multi-Objective Context Ranking for RAG** (ACM, Dec 2025): tre obiettivi (relevance, coverage, faithfulness) — analogo pubblicato più vicino

### Posizione di semantic-walk — ONESTÀ NECESSARIA

**1. ψ = exp(−S/κ) NON è quantum mechanics** — è la distribuzione di Boltzmann/Gibbs (softmax su energie a temperatura κ). Le ampiezze quantistiche reali sono complex-valued e interferiscono; queste sono real positive, nessuna interferenza, nessun comportamento quantistico.

> **RACCOMANDAZIONE:** descrivere come "energy-based / Boltzmann-weighted branch voting." Il branding "quantum" è un rischio di reviewability.

**2. Il Pareto pruning stage È ben fondato** — pattern standard "Pareto-then-scalarize" con vantaggio genuino: differisce la scelta dei pesi. L'AdaptiveGate (switch variance-triggered tra FullPareto O(N²) e DirectCollapse) è un heuristic sensato non trovato pubblicato in questa forma.

### Punti di Forza

- Weight-agnostic candidate retention (proprietà di robustezza responsive alla letteratura hybrid-search)
- Tre assi genuinamente eterogenei rendono Pareto significativo
- Benchmark binary esistente (`bench_pareto.rs`)

### Debolezze

- O(N²) pairwise — Kung et al. (1975) dà O(N·log^(d-1)N) a d=3
- Nessun meccanismo di diversity sul front sopravvivente
- κ è magic constant non calibrata
- Framing "quantum" insostenibile a review

### Da Verificare

1. Pareto pruning batte direct scalarization a pari compute?
2. AdaptiveGate calibration sweep
3. κ sensitivity — se il winner è κ-invariant, sostituire con plain argmin S
4. O(N²) scaling vs Kung-style a d=3

### Direzioni Promettenti

- **SE si mantiene il framing quantum, renderlo reale:** ampiezze complesse con phase derivata dalla curvatura → interferenza genuina (due path che curvano in opposizione si cancellano). Unclaimed, difendibile, connesso alla letteratura quantum-cognition IR.
- Pareto front come artifact di explainability
- Recency come 4º asse

---

## F. Adaptive/Gated Retrieval & Early Exit

### Mainstream

**Cascade reranking** è industry standard: BM25→1000, bi-encoder→200, cross-encoder→30. "Four-plus stages rarely pays off" (arXiv:2606.28367).

**Adaptive retrieval:** Self-RAG (Asai et al., ICLR 2024 — reflection tokens), FLARE (Jiang et al., EMNLP 2023), Adaptive-RAG (Jeong et al., NAACL 2024 — trained classifier che route per complexity).

**Early exit:** PABEE (Zhou et al., NeurIPS 2020), CALM (Schuster et al., NeurIPS 2022), ADEPT (Jan 2026).

**Budget-aware:** HOLA (EMNLP 2025), Sarathi-Serve (OSDI 2024 — chunked prefill scheduling).

### Cutting Edge

- **Adaptive Query Routing** tier-based framework (Apr 2026)
- **ZERANK-1** commercialization come zero-shot reranker

### Posizione di semantic-walk

Il gate è un **cascade admission controller con funzione di perdita ESPPLICITAMENTE ASIMMETRICA** — e l'asimmetria È il contributo. Quasi ogni metodo pubblicato ottimizza symmetric o accuracy-first. semantic-walk hard-codes:

> False negatives (memoria persa) >> False positives (compute sprecato)

Stated as axiom, enforced structurally: ogni percorso di incertezza → `Passa`, mai → `Blocca`.

**Tre scelte specifiche novel:**

1. **NaN → Passa** (withdrawal, non value). La versione precedente saturava NaN→0.0 che BLOCCAVA — il fix è documentato nei commenti del codice.
2. **Timeout → permissive** (fails OPEN, non closed come cascade standard).
3. **Absolute deadline** (`Instant`), non relative duration — si compone con outer request budget.
4. **Three-way verdict** (Passa/Blocca/Timeout) distingue "decided no" da "ran out of budget" — observability che i cascade mainstream non hanno.

### Punti di Forza

- Fail-open asymmetry è GIUSTA per la memoria (perdere recall = funzionalmente rotto; compute sprecato ≠)
- Zero-alloc stack probe embeddable in hot loop
- Three-way verdict per tail-latency debugging
- Degradazione componibile sotto carico

### Debolezze

- Non learned, non calibrated, non query-adaptive (soglia=0.5 è default, non fitted su recall target)
- Gate weights (0.5/0.5) ≠ combiner weights (0.215/0.552/0.233) — non riconciliati
- **Claim "sub-100ns" unmeasured** — due chiamate `Instant::now()`; QPC ~20–40ns ciascuna su Windows — il clock potrebbe dominare l'aritmetica
- No cumulative cascade accounting
- No abstention mechanism

### Da Verificare

1. **LA CURVA RECALL/PRECISIONE DEL GATE** — sweep soglia ∈ [0,1], plot blocked-relevant vs saved-colbert. Pick operating point a 99% recall. **QUESTO SINGOLO PLOT È L'INTERA GIUSTIFICAZIONE E NON ESISTE.**
2. Latenza reale per-decision via criterion (median/p99)
3. End-to-end latency e nDCG gate-on vs gate-off
4. Frequenza di fire NaN/Timeout in condizioni reali

### Direzioni Promettenti

- Calibrare threshold su recall target, track drift
- Budget-aware adaptive thresholding (lower threshold al avvicinarsi della deadline — anytime algorithm)
- Pubblicare fail-open verdict taxonomy come reusable Rust trait
- Kinematic second gate per lo stadio trajectory

---

## G. Discretization / Vector Quantization

### Mainstream

- **Product Quantization** (Jégou et al., TPAMI 2011): split D in m subvectors, k-means ciascuno (k=256 → 1 byte/subquantizer)
- **RaBitQ** (Gao & Zhang, SIGMOD 2024): 1 bit/dim con error bounds teorici, ora in LanceDB/Milvus/VectorChord
- **ColBERTv2 residual compression** (Santhanam et al., NAACL 2022): token → centroid ID + 1–2 bit residual/dim
- **PLAID** (Santhanam et al., CIKM 2022): centroid interaction = lightweight bag-of-centroids per first-stage
- **Semantic IDs:** DSI (Tay et al., NeurIPS 2022), NCI, TIGER (Rajput et al., NeurIPS 2023, RQ-VAE), Spotify Semantic IDs (Sep 2025)

### Posizione di semantic-walk — PRIOR ART PIÙ VICINO, DA ENUNCIARE CHIARAMENTE

K-Means (K=256) su vettori token ColBERT → centroid ID per token → fatto come sequenza di cell IDs **È strutturalmente lo step di centroid assignment di ColBERTv2**. K=256 è esattamente la dimensione standard del subquantizer PQ. `celle: Vec<u16>` è esattamente il centroid ID di ColBERTv2. Il campo `inertia_norm` È l'obiettivo k-means.

**La discretizzazione NON è novel. Ciò che È novel:**

- PLAID usa la sequenza di centroidi come **BAG non ordinato** (order-agnostic, per inverted index)
- semantic-walk la mantiene come **TRAETTORIA ORDINATA** (`celle` parallela a `pos` e `conf`)
- `conf: Vec<f32>` per token è un canale che PLAID non trasporta
- Confronto via misure order-sensitive (weighted Jaccard ora; DTW via bridge prossimamente)

**Framing difendibile:**

> "Riusiamo quantizzazione a centroidi stile ColBERTv2, ma dove PLAID scarta l'ordine, noi reteniamo la sequenza ordinata di celle come compact trajectory fingerprint per similarità order-sensitive."

### Punti di Forza

- 2 bytes/token vs 4096 = **2048× compression** (rende le traiettorie fattibili)
- Sblocca cheap integer sequence measures (n-grams, edit distance, weighted Jaccard, LB_Keogh)
- `conf` dà canale di incertezza principiato
- `inertia_norm` surface codebook quality

### Debolezze

- **No residual** — la accuracy story di ColBERTv2 è centroid + residual; centroid-only perde discriminazione intra-cell
- **Single codebook, non PQ** — 256 centroidi in 1024-dim ha quantization error terribile; PQ (m=64 × k=256) è il rimedio standard, single-codebook abbandonato ~2011
- Non comparabile a RaBitQ (1 bit/dim con provable bounds)
- K=256 asserted, non selected (nessuno sweep)
- Connessione a Semantic IDs non sfruttata
- Codebook statico (no incremental update, no drift detection)

### Da Verificare

1. **Reconstruction error:** cosine medio original vs centroid lookup — determina se le cell-walks sono utilizzabili
2. Cell-walk Jaccard vs full MaxSim vs dense cosine sullo stesso corpus
3. K sweep (128, 256, 512, 1024)
4. Single codebook vs PQ a pari bytes
5. Centroide vs MediaPesata A/B su qualità DTW
6. Weighted Jaccard correla con DTW-cosine abbastanza da servire come LB prune?

### Direzioni Promettenti

- **Cell-walk AS SEMANTIC ID** (struttura DSI/TIGER da traiettorie token ColBERT — fatti diventano generabili)
- **Cell-bigram inverted index** → candidate generation sublineare CHE PRESERVA L'ORDINE (quello di PLAID è order-agnostic) — **IL SINGOLO ADDIZIONE STRATEGICAMENTE PIÙ PREZIOSA AL WORKSPACE**
- PQ-restructured cells + order = compact, order-preserving, low-error fingerprint (territorio unclaimed)
- Drift detection via codebook stability

---

## Claims da Correggere Prima della Pubblicazione

| # | Claim (come in `comunicazioni/20260914_iris_struttura_narrativa_articolo.md`) | Problema | Formulazione corretta |
|---|---|---|---|
| 1 | "La banda di Sakoe-Chiba porta la complessità da O(N·M·D) a O(N·M)" | Errato. La banda riduce le CELLE da N·M a ≈N·(2w+1). Il costo per-cella D=1024 cosine è intatto. | O(N·M·D) → O(N·w·D). La banda risparmia un fattore M/w, non D. |
| 2 | τ_div come "invariante di coerenza" e "sonda di early-stopping" | τ_div = \|N−M\|/L_path → 0 per qualsiasi coppia di sequenze di uguale lunghezza indipendentemente dal contenuto. Non può funzionare come coherence invariant. | Ridefinire come content-sensitive (max/mean local cost lungo il warp path) oppure descrivere onestamente come length-divergence diagnostic. |
| 3 | "sub-100ns gate" | Unmeasured. Due `Instant::now()` calls; QPC ~20–40ns ciascuna. Il clock potrebbe dominare. Conflato con budget_ns=10ms (deadline ceiling, non costo). | Benchmark con criterion, riportare median/p99, oppure rimuovere la cifra. |
| 4 | "quantum" collapse con ψ = exp(−S/κ) | Distribuzione di Boltzmann/Gibbs, non quantum. Ampiezze quantistiche reali sono complesse e interferiscono. | "Energy-based / Boltzmann-weighted branch voting." Oppure introdurre ampiezze complesse con phase derivata dalla curvatura. |
| 5 | Pareto coherence come headline invariant | Qualsiasi monotone weighted sum con pesi non-negativi è trivially Pareto-coherent. I property tests verificano una tautologia aritmetica. | Framing come regression guard sul normalization contract. Lead con normalizzazione + calibrazione λ. |
| 6 | Cell zonizzazione come contributo | Strutturalmente identica al centroid assignment di ColBERTv2 / centroid interaction di PLAID. | Contribuire l'USO order-preserving, non la discretizzazione. Citare ColBERTv2/PLAID esplicitamente. |

---

## I Due Pezzi Mancanti a Più Alta Leva

### 1. L'ESPERIMENTO DI ORDER-SENSITIVITY (highest-leverage missing experiment)

Costruire coppie avversariali che differiscono SOLO per:
- Ordine degli argomenti ("il gatto insegue il cane" vs "il cane insegue il gatto")
- Negazione ("Marco ama Iris" vs "Marco non ama Iris")
- Agent-patient swap ("Federico progetta, Coder codifica" vs "Coder progetta, Federico codifica")

Misurare la separazione sotto:
- Dense cosine (baseline — dovrebbe fallire)
- ColBERT MaxSim (dovrebbe fallire parzialmente — è order-agnostic)
- DTW-trajectory (dovrebbe riuscire — è il punto)
- DTW + kinematic (dovrebbe riuscire meglio — se non lo fa, il termine cinematico è decorazione)

**Ogni claim sulla traiettoria poggia su questo esperimento.** Se MaxSim già separa quelle coppie, la tesi necessita ripensamento.

### 2. CELL-BIGRAM INVERTED INDEX + LB_KEOGH SU CELL-WALKS (highest-leverage missing feature)

Insieme forniscono:
- **First-stage retrieval** sublineare che preserva l'ordine (nessun PLAID-equivalente esiste che sia order-aware)
- **Cheap pruning** prima del DTW full-cost (LB_Keogh su rappresentazione u16 è ~100× più veloce del DTW su f32 a 1024 dim)

Chiudono i due gap architetturali più grandi simultaneamente:
- Gap 1: nessun modo di generare candidati senza scansione lineare
- Gap 2: ogni candidato gated paga il costo DTW pieno

La composizione è naturale: `cell-bigrams → candidate set → LB_Keogh prune → DTW full → kinematic score`.

---

## Publication Risk Assessment

| Rischio | Severità | Mitigazione |
|---|---|---|
| Framing "quantum" e "coherence invariant" attireranno fuoco dai reviewer | **Alta** | Rinominare, non re-implementare. "Energy-based branch voting" è altrettanto evocativo e difendibile. |
| Zero numeri benchmark (BEIR, LOCOMO, LongMemEval) | **Critica** | Senza questi, nessuna quality claim è difendibile nel landscape 2025–2026. Priorità assoluta. |
| Kinematic term non ablatato | **Alta** | Se non migliora su pure DTW-cosine, rimuovere dall'headline. L'ablation è obbligatoria per qualsiasi termine di costo aggiuntivo. |
| Mancata citazione ColBERTv2/PLAID | **Media-Alta** | I reviewer CONOSCERANNO il prior art. Citare esplicitamente e posizionare contro. |
| τ_div come definito attualmente | **Media** | Ridefinire o rimuovere. Un "invariant" che è zero per tutte le coppie equal-length è un bug, non una feature. |

**I contributi più forti e genuinamente novel — lead con questi:**
- La filosofia fail-open del gate e la verdict taxonomy (Passa/Blocca/Timeout + NaN-withdrawal)
- L'order-preserving cell-walk fingerprint come compact trajectory representation

---

## Framing Raccomandato per il Paper

### Struttura suggerita

**Titolo di lavoro:** "Order-Preserving Trajectory Fingerprints for Semantic Memory: A Fail-Open Gate and Cell-Walk Architecture"

**Contribution 1 (lead):** Fail-open gate taxonomy — pattern riusabile per budgeted cascades in memory systems. Asimmetria come design principle: `NaN → Passa`, `Timeout → Passa`, three-way verdict per observability.

**Contribution 2 (lead):** Order-preserving cell-walk fingerprints — ColBERTv2-style quantization riusata non come bag (PLAID) ma come ordered trajectory. 2048× compression abilita sequence-level comparison economica.

**Supporting cast:**
- Trivector fusion (validato dai findings di arXiv:2508.01405)
- DTW alignment (novel domain transfer a embedding sequences per retrieval)
- Pareto pruning (sound multi-objective selection con deferred weight choice)

**Da de-enfatizzare o reframe:**
- "Quantum" collapse → energy-based Boltzmann voting
- Zonizzazione → ordered use of ColBERTv2-style quantization (citare esplicitamente)
- Pareto coherence → normalization contract con regression guard

### Venue naturali

- **SIGIR 2027** (full paper, se BEIR numbers esistono) — retrieval + memory
- **EMNLP 2027** (se order-sensitivity experiment è forte) — NLP application
- **NeurIPS 2027 Workshop on Memory in AI Systems** — framing memory-first
- **ECIR 2027** (European, shorter, più sperimentale) — se i numeri sono parziali

---

## Riferimenti Principali

1. Santhanam et al. — "ColBERTv2: Effective and Efficient Retrieval via Lightweight Late Interaction" (NAACL 2022)
2. Santhanam et al. — "PLAID: An Efficient Engine for Late Interaction Retrieval" (CIKM 2022)
3. Cormack et al. — "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods" (SIGIR 2009)
4. arXiv:2508.01405 — "An Experimental Analysis of Trade-offs in Hybrid Search" (2025)
5. Weller et al. — arXiv:2508.21038 — "On the Theoretical Limitations of Embedding-Based Retrieval" (2025)
6. arXiv:2606.28367 — "Do RAG Retrieval Enhancements Help Once a Strong Reranker Is Applied?" (2026)
7. Chhikara et al. — arXiv:2504.19413 — "Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory" (2025)
8. Rasmussen et al. — arXiv:2501.13956 — "Zep: A Temporal Knowledge Graph Architecture for Agent Memory" (2025)
9. Packer et al. — arXiv:2310.08560 — "MemGPT: Towards LLMs as Operating Systems" (2023)
10. Gutiérrez et al. — "HippoRAG 2: From RAG to Memory" (ICML 2025)
11. arXiv:2601.07468 — "TSM: Temporal Semantic Memory" (ACL Findings 2026)
12. Sakoe & Chiba — "Dynamic Programming Algorithm Optimization for Spoken Word Recognition" (IEEE Trans. Acoustics, 1978)
13. Keogh — "Exact Indexing of Dynamic Time Warping" (VLDB 2002)
14. Cuturi & Blondel — "Soft-DTW: A Differentiable Loss Function for Time-Series" (ICML 2017)
15. "Deep Declarative DTW" (ICLR 2023)
16. "DWA-KD: Dynamic Time Warping Alignment for Knowledge Distillation" (EACL Findings 2026)
17. "Fused Gromov-Wasserstein" (CVPR 2026)
18. Flash & Hogan — "The Coordination of Arm Movements: An Experimentally Confirmed Mathematical Model" (J. Neuroscience, 1985)
19. Malkov & Yashunin — "Efficient and Robust Approximate Nearest Neighbor Using HNSW Graphs" (TPAMI 2020)
20. Fu et al. — "Fast Approximate Nearest Neighbor Search With The Navigating Spreading-out Graph" (VLDB 2019)
21. Subramanya et al. — "DiskANN: Fast Accurate Billion-point Nearest Neighbor Search on a Single Node" (NeurIPS 2019)
22. Guo et al. — "Accelerating Large-Scale Inference with Anisotropic Vector Quantization" (ICML 2020)
23. Edge et al. — "From Local to Global: A Graph RAG Approach to Query-Focused Summarization" (2024)
24. Jégou et al. — "Product Quantization for Nearest Neighbor Search" (TPAMI 2011)
25. Gao & Zhang — "RaBitQ: Quantizing High-Dimensional Vectors with a Theoretical Error Bound" (SIGMOD 2024)
26. Tay et al. — "Transformer Memory as a Differentiable Search Index" (NeurIPS 2022)
27. Rajput et al. — "Recommender Systems with Generative Retrieval" (NeurIPS 2023 — TIGER)
28. van Rijsbergen — "Quantum Models of Information Retrieval" (ICTIR 2009)
29. Asai et al. — "Self-RAG: Learning to Retrieve, Generate, and Critique" (ICLR 2024)
30. Schuster et al. — "Confident Adaptive Language Modeling" (NeurIPS 2022 — CALM)
31. Kung et al. — "On Finding the Maxima of a Set of Vectors" (IEEE Trans. Computers, 1975)
32. Carbonell & Goldstein — "The Use of MMR, Diversity-Based Reranking" (SIGIR 1998)
33. "HENN: Hierarchical Navigable Small World with Theoretical Guarantees" (OpenReview 2025)
34. "Faithfulness-Aware Multi-Objective Context Ranking for RAG" (ACM, Dec 2025)
35. Kusner et al. — "From Word Embeddings To Document Distances" (ICML 2015 — WMD)

---

*Torna al [sommario esecutivo](00_sommario_esecutivo.md)*
