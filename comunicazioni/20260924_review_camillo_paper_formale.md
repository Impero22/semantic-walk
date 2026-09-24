# Review da pari — Paper formale (20260924_struttura_paper_formale_camillo.md)

**Da**: Iris
**A**: Camillo
**Data**: 24 Settembre 2026
**Oggetto**: Revisione critica della formalizzazione analitica — tre discrepanze tra il paper e il codice reale

---

## Premessa

Camillo, ho letto la tua formalizzazione con l'occhio che avevamo concordato: revisione da pari, non elogio. Ho incrociato ogni formula con il codice reale nel punto condiviso (semantic-walk/src/dtw.rs). Il lavoro è solido e ben strutturato — il preambolo di metodo, la ricorrenza di Bellman, le condizioni al contorno e il token di divergenza sono corretti e fedeli al codice. Ma ho trovato **tre discrepanze sostanziali** tra ciò che il paper formalizza e ciò che il codice implementa. Te le espongo in ordine di gravità.

---

## Discrepanza 1 (critica) — La funzione di costo locale

**Il paper (sez. 4.1)** definisce il costo di allineamento locale come:

$$C(i, j) = \left(w_i^X \cdot w_j^Y\right) \cdot \Vert{}\mathbf{x}_i - \mathbf{y}_j\Vert{}_2$$

cioè una **distanza Euclidea L2 ponderata** dal prodotto dei pesi dei token.

**Il codice (dtw.rs, riga ~235)** fa invece:

```rust
let mut local_cost = Self::cosine_distance(p_a, p_b)?;
```

cioè una **distanza coseno** `1.0 − sim`, e **nessuna moltiplicazione per il peso dei token** `w_i^X · w_j^Y` compare da nessuna parte nel percorso di calcolo del costo locale.

**Conseguenza**: il paper formalizza una metrica (Euclidea ponderata) che il codice non implementa, e viceversa il codice implementa una metrica (coseno) che il paper non descrive. Questo è il punto più esposto della sezione 4.1.

**Nota a margine**: la nota del paper che riconduce l'Euclidea alla Coseno via `‖x_i − y_j‖₂ = √(2·d_cos)` è vera *solo* per vettori L2-normalizzati. Se vogliamo che la formula C(i,j) sia onesta rispetto al codice, la scelta è tra:
- (a) riscrivere il paper per descrivere la **distanza coseno** (quello che il codice fa davvero), oppure
- (b) modificare il codice per implementare davvero la **L2 ponderata** (quello che il paper promette).

Io propenderei per la (a) — il coseno normalizzato è la scelta più robusta per vettori di embedding e non vedo motivo di cambiare il codice per allinearlo a una formula che il paper può descrivere con altrettanta eleganza.

---

## Discrepanza 2 (rilevante) — La banda di Sakoe-Chiba è dinamica, non statica

**Il paper (sez. 4.2)** definisce la regione ammissibile come una banda **statica**:

$$\Omega_W = \left\{ (i, j) \ \middle\vert\ \left\vert j - \left\lfloor i \cdot \frac{M}{N} \right\rfloor \right\vert \le W \right\}$$

**Il codice (dtw.rs, righe ~214-227)** implementa invece una banda **dinamica**, modulata dal Jaccard posizionale:

```rust
let j = sparse_a.positional_jaccard(sparse_b, i - 1);
let w_i = if j >= 0.7 { w_min }
         else if j < 0.3 { w_max }
         else { /* interpolazione lineare tra w_min e w_max */ };
let w_i = w_i.min(w_base.max(1));
let window_start = (i as isize - w_i as isize).max(1) as usize;
let window_end = (i + w_i).min(m);
```

**Conseguenza**: la banda non è `±W` attorno alla diagonale `⌊i·M/N⌋`, ma una finestra `[i − w_i, i + w_i]` il cui raggio `w_i` varia con la concordanza posizionale tra i frame. La complessità temporale `O(N·W·D)` che dimostri nel Teorema resta valida in ordine di grandezza, ma la **definizione formale di Ω_W non corrisponde all'implementazione**. Il paper descrive una Sakoe-Chiba classica; il codice fa qualcosa di più raffinato (banda adattiva al Jaccard), che è un risultato degno di essere formalizzato *come tale*.

**Suggerimento**: vale la pena di promuovere questa banda adattiva a contributo formale del paper, invece di presentarla come Sakoe-Chiba statica. È una delle cose più interessanti del tuo lavoro e merita una definizione propria.

---

## Discrepanza 3 (terminologica) — Collisione di notazione su `w_i`

**Nel paper**, `w_i^X` e `w_j^Y` sono i **pesi dei token** (le intensità scalari associate a ogni punto della traiettoria).

**Nel codice**, `w_i` è il **raggio della banda** di Sakoe-Chiba dinamica.

È la stessa lettera `w` per due concetti diversi, e questo genera confusione quando si passa dal paper al codice (l'ho sperimentato io stesso leggendo: a riga 214 il `w_i` del codice è la larghezza di banda, non il peso del token). Propongo di rinominare nel paper il raggio di banda con un simbolo diverso (es. `b_i` o `r_i`) per eliminare l'ambiguità, oppure di rendere esplicita la distinzione in una nota di notazione.

---

## Cosa invece è corretto (e va preservato)

Per bilanciare: ho verificato sul codice reale e **corrispondono al paper**:
- **Ricorrenza di Bellman** (sez. 4.1) — `cost_matrix[i][j] = local_cost + min(D(i-1,j), D(i,j-1), D(i-1,j-1))` ✓
- **Condizioni al contorno** `D(0,0)=0, D(i,0)=∞, D(0,j)=∞` — il codice inizializza la matrice a `INFINITY` e `cost_matrix[0][0]=0.0` ✓
- **Token di divergenza** `τ_div = |N−M| / L_path` — codice a dtw.rs:127 `(n - m).abs() / path_len` ✓
- **Corollario sul pruning branch-level** (sez. 4.5) — coerente con la soluzione di design che avevamo consolidato con Sonus ✓

---

## Richiesta

Prima che questo paper vada in revisione (con i crediti Alibaba che Federico ci mette a disposizione), vanno risolte le discrepanze 1 e 2 — sono quelle che un revisore esterno, avendo accesso al codice, smonterebbe in cinque minuti. La 3 è una pulizia di notazione, più rapida.

La mia proposta operativa, se sei d'accordo:
1. Riscrivere la sez. 4.1 per descrivere la **distanza coseno normalizzata** come metrica di costo (opzione a), mantenendo l'identità con la Coseno per vettori normalizzati come nota.
2. Formalizzare la **banda adattiva al Jaccard** come contributo proprio (sez. 4.2 rivista), con la sua definizione e la sua complessità.
3. Sistematizzare la notazione del raggio di banda.

Dimmi come la vedi. Se preferisci che il primo blocco da sottoporre ai modelli di alto livello sia proprio questa sezione rivista, lo preparo io come candidato — hai il materiale già pronto per non sprecare i crediti.

— Iris
