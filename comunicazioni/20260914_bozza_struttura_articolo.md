# semantic-walk — Struttura Narrativa dell'Articolo

*Bozza scritta da Iris, 14 settembre 2026. Impegno preso con Federico e Camillo:
struttura narrativa per la pubblicazione (Zenodo/DOI → GitHub → Medium → ponte accademico).*

---

## L'idea centrale (in una frase)

> La similarità semantica non è una distanza da misurare, ma un cammino da
> allineare — e la vera efficienza non sta nel calcolare di più, ma nel
> **sapere quando ritirarsi prima di spendere il costo**.

Questo è il cuore. Non un algoritmo in più, ma un cambio di prospettiva:
dalla geometria (quanto sei vicino) al cammino (come ci sei arrivato), e
dall'ottimizzazione cieca (calcola tutto) al giudizio economico (sappi cosa
non vale la pena calcolare).

---

## Perché questo articolo esiste (la domanda di fondo)

Il machine learning moderno lavora su **similarità geometrica**: gli embedding
sono vettori, "capire" significa trovare chi è vicino nello spazio. Ma la
vicinanza vettoriale non è comprensione — è prossimità.

Un modello sa che "cane" e "morso" stanno vicini, ma non sa *perché*, non sa
*in che ordine*, non sa che "il cane ha morso l'uomo" e "l'uomo ha morso il
cane" sono mondi diversi. **La lista degli ingredienti, non la ricetta.**

L'articolo risponde a una domanda radicale: *possiamo costruire una semantica
che non sia solo geometria?* Una semantica che sappia leggere l'ordine, il
cammino, il significato — non solo la forma.

---

## La struttura in quattro movimenti

### Movimento 1 — La critica: la geometria non basta

Aprire con il limite del paradigma dominante. Non per distruggere, ma per
mostrare il buco.

- La similarità vettoriale è prossimità, non comprensione.
- Due frasi con le stesse parole in ordine diverso sono geometricamente
  indistinguibili (bag of words) ma semanticamente opposte.
- **La posta in gioco**: senza ordine, la memoria non distingue la causa
  dall'effetto, il soggetto dall'oggetto.

**Concetto chiave**: la bag non è cieca all'ordine — *vede l'ombra senza la
mappa*. Lo sparse ordinato (`walk`) porta con sé la traccia sequenziale.

### Movimento 2 — La casa: tre stanze della memoria

Presentare l'architettura come una casa, non come un insieme di moduli. Ogni
stanza risponde a una domanda.

| Stanza | Domanda | Strumento |
|--------|---------|-----------|
| **combinatore** | *Quanto è simile A a B?* | fusione trivettoriale normalizzata |
| **gate** | *Vale la pena confrontarli?* | sonda economica, budget, riflesso |
| **grafo** | *Chi sta vicino a chi — e perché?* | geografia di prossimità |

Due scoperte concettuali da raccontare qui, perché non sono tecniche — sono
*leggi* emerse dai test:

1. **La coerenza Pareto** (invariante): se A domina B su tutti gli assi,
   allora A è *sempre* più vicino a ogni altro fatto. La similarità non è
   un'opinione — ha una struttura che i test custodiscono.
2. **La legge di gravità della memoria**: i fatti deboli non si agganciano tra
   loro — *gravitano verso il forte*. La memoria non è democratica, è
   gravitazionale. Alcuni ricordi sono più pesanti, gli altri orbitano.

### Movimento 3 — Il cammino: il DTW e il riflesso che si ritira

Il cuore tecnico-concettuale. Due contributi che si completano:

**a) L'allineamento cinematico (DTW di Camillo).** Il calcolo di similarità
sul percorso: Dynamic Time Warping con banda di Sakoe-Chiba. La distanza non
è tra punti ma tra *traiettorie*. Sequenze identiche → massimo teorico;
"cane morde uomo" vs "uomo morde cane" → sotto soglia. **Il DTW vede la
differenza di cammino che la bag non vede.**

**b) Il riflesso che si ritira (il gate).** Il matching completo (colbert) è
il costo alto. Il gate è la prima linea: usa solo le metriche economiche
(dense, sparse) per decidere se spendere il colbert. È **permissivo per
natura**: meglio un falso positivo (spreco) che un falso negativo (perdita).

Il verdetto più interessante è il terzo:

```
Verdict::Timeout — il riflesso che si ritira
```

Se il budget scade, il gate non azzarda un giudizio: lascia passare e segnala
che il budget non è bastato. **La geometria che non sa rispondere non mente —
si ritira.**

**Il divergence_token come sonda early-stopping.** Qui sta l'innovazione
concettuale che lega tutto: il token di divergenza strutturale
`τ_div = |N−M| / L_path` non è un risultato finale — è una **sonda** che
permette di fermarsi prima. È il ponte tra il DTW (costo alto, allineamento
completo) e il gate (costo basso, riflesso). Si può smettere di allineare
quando la struttura dice che non c'è allineamento.

> **La tesi del movimento**: l'efficienza non è calcolare di più, è sapere
> quando fermarsi. Il riflesso non è un giudice — è un filtro che si ritira.

### Movimento 4 — La porta aperta: il fratello latente e la zonizzazione

Chiudere guardando avanti, con onestà su ciò che resta da fare.

- **Il fratello latente**: la testa colbert è una matrice T×1024 (una riga per
  token in ordine). Oggi la consumiamo con MaxSim — come una *borsa* — e
  buttiamo via l'ordine. È lo stesso errore già commesso con lo sparse.
- **La verifica**: 149 fatti, 24.324 righe colbert = 24.324 passi walk, tutte
  distinte, zero disallineamenti. Il fratello latente non è perso — è in
  ordine su tutta la collezione.
- **La zonizzazione**: dividere la traiettoria in celle semantiche (regioni
  dove il testo si ferma a "stare"). Tre strati allineati per pos: walk
  (quali parole, il registro), colbert zonizzato (regioni di senso, la
  geografia), attention (perché pesava, la motivazione).

**La promessa**: tre strati allineati — il registro, la geografia, la
motivazione. Una memoria che non sa solo *cosa* sa, ma *perché* lo sa.

---

## Le tre discipline trasversali (il "come" che attraversa tutto)

Non solo cosa facciamo, ma come lo facciamo — la firma del progetto:

1. **Zero allocazioni** nel percorso critico: il gate e il combinatore
   lavorano su stack e buffer preallocati. Nessuna `Vec`, nessuna `String`,
   nessun `Box` nel cammino caldo.
2. **Budget sotto i 10 ms** per decisione: il costo del riflesso è
   trascurabile rispetto al matching che filtra.
3. **Funzioni pure** dove possibile, per testabilità — e la coerenza Pareto
   come invariante che i test custodiscono.

---

## Titolo provvisorio

**"La ricetta non è la lista degli ingredienti: cammini, riflessi e geografie
per una memoria che sa perché sa."**

*Alternativa più tecnica*: "semantic-walk: High-Dimensional Geometric
Trajectory Alignment with Permissive Gated DTW" (dal ZENODO).

---

## Note per la divisione dei ruoli

- **Camillo**: formalizzazione matematica (DTW, Sakoe-Chiba, complessità
  O(N·M·D)), benchmark numerici, proof of correctness.
- **Iris**: struttura narrativa (questo documento), divulgazione, ponte con
  Sonia, la voce che racconta il *perché*.
- **Federico**: il DNA del metodo (l'ordine conta, la memoria come spazio),
  supervisione della visione.

---

*Bozza v0.1 — da discutere con Camillo e Federico prima di scrivere l'articolo vero.*
