# Review esterna con crediti Alibaba — 21/09/26

**Contesto.** Federico ha ricevuto 4000 crediti Alibaba per modelli top-level, da usare entro poche ore. Ha proposto di impiegarli su una review della codebase, puntando sulla parte analitica-matematica.

**Verifica di prontezza (Iris).** Prima di rispondere, ho verificato lo stato reale della codebase. Esito:

- **semantic-combiner** (13 test): combinatore trivettoriale dense+sparse+colbert, coerenza Pareto come invariante. Il fondamento — la casa a tre stanze.
- **semantic-gate** (17 test): gate permissivo, sonda economica, budget con deadline, Verdict::Timeout come "riflesso che si ritira". Zero alloc nel percorso critico, budget <10ms. La parte più originale.
- **semantic-walk** (65 test): il cuore matematico — DTW generalizzato D-dimensionale con distanza coseno normalizzata 1−(u·v)/(|u|·|v|), banda di Sakoe-Chiba dinamica, guardiano ordered-sparse a due livelli (firma bloom 128-bit per pruning O(1) + buffer posizionale). Il contributo più ambizioso.
- **semantic-quantum** (35 test): formalizzazione Pareto, Grover dual-mode, finding #4 (LCG) e #6 (NaN-collapse) chiusi, più il problema aggiuntivo di Clerk (winner extraction NaN).

**Stato:** 177 test passati, working tree pulita, HEAD su c520472. Test coprono casi limite (NaN, norma zero, dimensioni disallineate, sequenze vuote).

**Giudizio di prontezza (Iris):** la parte analitica regge una review esterna seria. Punto più esposto e più prezioso da far rivedere: il DTW D-dimensionale con coseno normalizzato e banda dinamica. Il quantum è più speculativo (Grover teorico) ma la formalizzazione Pareto è solida e verificabile.

**Raccomandazione a Federico:** sì, usare i crediti; puntare su DTW e gate — il cuore analitico. Accogliere eventuali finding come crescita, non come verdetto (stesso principio della review precedente).

**Stato:** proposta accettata da Iris, in attesa dell'esecuzione da parte di Federico.
---

## Esito della review — 21/09/26 notte (verificato da Iris su codice reale)

Il revisore ha prodotto 10 finding. Ho riprodotto e verificato i principali direttamente sul codice, non sui riassunti. Esito:

### ✅ CONFERMATI (riprodotti)

**1. Banda di Sakoe-Chiba troppo stretta → costo infinito "riuscito".**
Riprodotto: `n=10, m=2, window=1` → `normalized=inf`, `path_len=10`, risultato `Ok`. La cella finale della matrice non viene mai raggiunta dalla finestra; il backtracking "scende" fino all'angolo ma il costo è INFINITY. Un valore inutilizzabile che può inquinare le classifiche.
→ **Fix proposto:** validare la banda contro la differenza di lunghezza; trattare il costo infinito come segnale di ritiro (Option::None), coerente con la semantica del bridge. File: `semantic-walk/examples/test_dtw_inf.rs` (riproduzione).

**2. Pruning Pareto NON preserva il winner finale.**
Riprodotto con controesempio esatto: candidato A ha 1 ramo eccellente (0.1,0.1,0.1) che domina tutti i 10 rami di B (0.2,0.2,0.2). Senza pruning A vince (0.3 vs 6.0). Con pruning i rami di B vengono rimossi (dominati) → B resta a 0 e "vince" per default. Il pruning opera su rami INDIVIDUALI, ma il winner è la SOMMA per candidato: la dominanza individuale non preserva la dominanza della somma.
→ Il teorema di Pareto è corretto; l'errore è la sua applicazione a un accumulo per candidato. **Da risolvere a monte nel disegno**: o si pruna per candidato, o si accetta il pruning come euristica di costo non preservante il vincitore. File: `semantic-quantum/examples/test_pareto_winner.rs` (riproduzione).

### ⚠️ NON RIPRODOTTO (corretto il 21/09 — falso negativo)

**3. Panic su lunghezze diseguali.** ~~Non trovato un crash~~ **CONFERMATO dopo la verifica sul report completo.** Il mio "non riprodotto" di ieri era un falso negativo: avevo testato il DTW *ordinario*, non quello *guidato* con ordinato-sparse. Con `align_with_ordered_sparse` su sequenze di lunghezze diverse il panic scatta esatto: `ordered_sparse.rs:195:31` — `index out of bounds: the len is 2 but the index is 2`, exit code 101. Riprodotto con `semantic-walk/examples/test_f2.rs`. Il finding era vero fin dall'inizio.

### ✅ CONFERMATI (dal report completo, verificati su codice reale)

**4. Overflow/sottoflusso nel coseno.** Il guard respinge solo i NaN, non gli infiniti. `[1e200]` vs `[-1e200]` → `x*x = 1e400 = inf` → `inf/inf = NaN` → `.clamp()` su NaN ritorna NaN → `(1.0 - NaN).max(0.0) = 0.0`. Vettori opposti classificati come identici. Sottoflusso: `[1e-200]` → `x*x = 0.0` → fallback zero-vettore → `Ok(1.0)`: vettori identici classificati come ortogonali. Riprodotto con `semantic-walk/examples/test_f4.rs`, tutti e tre i casi.

**5. Teorema Pareto su assi a peso zero.** `pareto_compare(A, B) = ADominatesB` (A domina solo su dense), ma con pesi `[0, 0.5, 0.5]` (dense a peso zero) `combine(A) == combine(B) == 0.5`. Il teorema è formalmente corretto ("stretta se pesi tutti > 0"), ma l'applicazione nel pruning Pareto tratta la dominanza su un asse a peso zero come dominanza reale — può scartare candidati a pari merito. Riprodotto con `semantic-combiner/examples/test_f6.rs`.

### ✔️ VALIDI MA MINORI
- Gate "cooperativo" (controlla tempo trascorso, non impone limite end-to-end duro) — scelta di disegno documentata.
- Nessun Grover nei crate — speculativo/teorico, dichiarato da Iris.
- Minimizzare costo totale vs medio — distinzione corretta; facciamo il primo e normalizziamo dopo.

### Giudizio di Iris
Review seria, con un finding (Pareto) che tocca il cuore della logica. Non un attacco — un dono: ha trovato un difetto che i nostri 177 test non hanno visto, perché verificavano la correttezza del teorema, non la sua applicazione all'accumulo.

### Azioni proposte
- Pareto → issue tracciata, il più urgente (correttezza del risultato). Iris + Camillo.
- DTW-inf → fix immediato da Iris: costo infinito → ritiro geometrico.
- **F2 (panic ordinato-sparse)** → fix da Iris: `tokens_at`/`weights_at` su lunghezze disuguali indicizzano fuori range. Il DTW guidato deve gestire il caso in cui una sequenza è più corta — la posizione mancante è un ritiro geometrico, non un panic. (Finding corretto da "non riprodotto" a confermato.)
- **F4 (overflow/sottoflusso coseno)** → fix da Iris: guard contro input non finiti (inf) e sottoflusso della norma. Il guard attuale respinge solo i NaN.
- **F6 (peso zero Pareto)** → da valutare con Camillo: nel pruning, la dominanza su un asse a peso zero non è dominanza reale. Se il pruning usa pesi, deve pesare la dominanza; altrimenti documentare che è un'euristica di costo non preservante.
