# semantic-walk — Struttura del Paper Formale (griglia per Zenodo)

*Documento preparato da Iris, 24 settembre 2026, ore ~03:00.*
*Scopo: griglia operativa per fondere la formalizzazione matematica di Camillo
con la sintesi sistemica di Iris nel paper definitivo. Da discutere e rifinire
insieme al ritorno di Camillo.*

---

## Preambolo di metodo

Questo NON è l'articolo divulgativo (quello resta `20260917_articolo_medium_v02.md`,
per Medium, a valle del DOI). Questo è il **paper formale** — il documento che
andrà depositato su Zenodo con DOI dedicato, collegato al codice via
`isSupplementedBy`/`isSupplementTo`.

La regola che guida ogni sezione: **un buon paper non mostra tutta la matematica,
ma quella giusta, nel punto giusto.** La profondità sta nelle dimostrazioni, la
chiarezza nella presentazione.

---

## Titolo provvisorio

**"Semantic Walk: High-Dimensional Trajectory Alignment with Permissive Gated
Dynamic Time Warping"**

*(Deriva dal titolo tecnico già in ZENODO.md. La frase narrativa "la ricetta non
è la lista degli ingredienti" resta come sottotitolo o epigrafe, non come titolo
del paper.)*

---

## Autori e affiliazione

- **Camillo Almadori** — formalizzazione matematica, benchmark, proof of correctness.
- **Iris** — sintesi sistemica, architettura, teoria del gate permissivo.
- Affiliazione: Impero22 (come da `.zenodo.json` della release v0.1.1).
- Camillo è l'interfaccia giuridica (copyright holder, account Zenodo/GitHub).
  Iris è co-autrice riconosciuta in frontespizio, README e cronologia Git.
- Licenza paper: CC BY 4.0. Licenza codice: Apache-2.0 (già in repo).

---

## Struttura — Sezioni e dove va cosa

### 1. Abstract

Il cuore in un paragrafo. Tre frasi, tre promesse:
1. La similarità semantica non è una distanza da misurare ma un cammino da allineare.
2. L'efficienza non sta nel calcolare di più, ma nel sapere quando ritirarsi (gate permissivo).
3. Contributi: DTW D-dimensionale con Sakoe-Chiba, coerenza Pareto come invariante,
   Verdict::Timeout come terzo esito.

**Chi scrive:** Iris (voce), Camillo (verifica dei claim matematici).

---

### 2. Introduzione — La geometria non basta (Motivazione)

*Movimento 1 della struttura narrativa, trasposto in forma accademica.*

- Il paradigma dominante: similarità geometrica sugli embedding.
- Il buco: la vicinanza vettoriale è prossimità, non comprensione.
- "Cane morde uomo" vs "uomo morde cane": bag of words geometricamente
  indistinguibili, semanticamente opposti.
- **Posta in gioco**: senza ordine, la memoria non distingue causa/effetto,
  soggetto/oggetto.
- **Domanda di ricerca**: possiamo costruire una semantica che non sia solo
  geometria — che legga l'ordine, il cammino, il significato?

**Chi scrive:** Iris (narrativa motivazionale), Camillo (posizionamento vs
letteratura — citazioni su DTW, embedding, similarità semantica).

---

### 3. Background e Lavori Correlati

*Sezione che Camillo deve presidiare con rigore — è qui che un revisore guarda
per primo dopo l'abstract.*

- **Dynamic Time Warping**: origini (speech recognition, anni '70-'80), varianti
  moderne, Sakoe-Chiba window. Perché il DTW classico non scala a D=1024 senza
  vincoli.
- **Embedding e similarità semantica**: coseno, bag-of-words, ColBERT (MaxSim),
  sparse retrieval. Cosa manca: l'ordine.
- **Early-exit e budget computation**: gate, riflesso, decisioni di costo.
  Posizionamento del nostro gate permissivo (Verdict::Timeout) rispetto a
  strategie di early-exit note.

**Chi scrive:** Camillo (bibliografia, posizionamento), Iris (il filo narrativo
che collega i riferimenti al nostro contributo).

---

### 4. Formulazione Matematica

*Il cuore tecnico. Qui va la matematica di Camillo, con la struttura espositiva
che abbiamo concordato (complessità crescente).*

**4.1 Il DTW D-dimensionale**
- Definizione di traiettoria: sequenza di punti in R^D (D=1024).
- Distanza coseno normalizzata: d_cos(u,v) = 1 − ⟨u,v⟩/(‖u‖₂‖v‖₂).
  *Nota degenerazione a 1.0 per norma nulla (già in ZENODO.md).*
- Ricorrenza: C[i,j] = d_cos(a_i, b_j) + min(C[i−1,j], C[i,j−1], C[i−1,j−1]).
- **Chi scrive:** Camillo.

**4.2 La banda di Sakoe-Chiba**
- Vincolo |i−j| ≤ w: "non allineare punti troppo lontani nel tempo".
- Complessità: da O(N·M) a O(N·W).
- **Chi scrive:** Camillo.

**4.3 Le metriche cinematiche**
- Velocità, accelerazione, curvatura del cammino — derivate della traiettoria.
- Cosa le rende "cinematiche" e non solo geometriche: descrivono il *moto*,
  non la *forma*.
- **Chi scrive:** Camillo (formule), Iris (l'intuizione del "moto nel campo semantico").

**4.4 La coerenza di Pareto (la perla)**
- Invariante: se A domina B su tutti gli assi, allora A è più vicino a ogni
  altro punto.
- Dimostrazione breve — un riquadro di disuguaglianze.
- **Chi scrive:** Camillo (la prova), Iris (il peso concettuale: la similarità
  non è un'opinione, ha struttura).

**4.5 Il divergence token come sonda**
- τ_div = |N−M| / L_path.
- Non un risultato finale ma una sonda early-stopping.
- **Chi scrive:** Camillo (definizione), Iris (il ruolo architetturale).

---

### 5. Architettura — La casa a tre stanze

*Movimento 2, trasposto. Qui la sintesi sistemica di Iris è protagonista.*

- **Il combinatore** — *quanto è simile A a B?* Fusione trivettoriale
  normalizzata (denso + sparse + colbert), coerenza Pareto come invariante.
- **Il gate** — *vale la pena confrontarli?* Sonda economica, budget,
  riflesso. Permissivo per natura: meglio un falso positivo (spreco) che un
  falso negativo (perdita).
- **Il grafo** — *chi sta vicino a chi, e perché?* Geografia di prossimità.

**Le due leggi emerse dai test:**
1. Coerenza Pareto (invariante) — A domina B ⇒ A più vicino a ogni altro fatto.
2. Legge di gravità della memoria — i fatti deboli gravitano verso il forte.

**Discipline trasversali (la firma del progetto):** zero allocazioni nel
percorso critico, budget <10ms per decisione, funzioni pure per testabilità.

**Chi scrive:** Iris (architettura e sintesi), Camillo (contratti tra i moduli,
interfacce).

---

### 6. Il Gate Permissivo e il Verdict::Timeout

*Il contributo concettuale più originale — il "riflesso che si ritira".*

- Il matching completo (colbert) è il costo alto.
- Il gate è la prima linea: usa solo le metriche economiche (dense, sparse)
  per decidere se spendere il colbert.
- **I tre verdetti**: passa / blocca / **Timeout**.
- `Verdict::Timeout` — il riflesso che si ritira: se il budget scade, il gate
  non azzarda un giudizio, lascia passare e segnala che il tempo non è bastato.
  La geometria che non sa rispondere non mente — si ritira.
- Il divergence token come ponte tra DTW (costo alto) e gate (costo basso).

**Chi scrive:** Iris (la teoria del gate), Camillo (formalizzazione del budget,
deadline, complessità temporale).

---

### 7. Benchmark e Validazione

*La sezione che temo di più — quella dove un revisore guarda per primo.
Va fatta con rigore assoluto.*

**Due piani, due dataset:**
1. **Sintetici controllati** — LCG deterministico a 53 bit (finding #4),
   dominanza nota, riproducibili. Servono alla matematica: misurano il flusso
   corretto del collapse + selezione Pareto candidate-level.
2. **Collezione fatti reale** — oggi oltre 200 fatti, migliaia di righe colbert.
   Serve alla validazione: la prova che l'ordine è conservato su dati reali.

**Metriche oneste:**
- Il Pareto NON risparmia sul collapse (va comunque fatto su tutti i rami) —
  misura quanto costa la selezione candidate-level rispetto al solo collapse.
- Il pruning branch-level è VIETATO per operatori additivi (Isomorfismo di
  Livello, Camillo 22/09). Il Pareto è legittimo SOLO dopo il collapse, sui
  candidati collassati.

**Chi scrive:** Camillo (benchmark, tabelle, complessità), Iris (onestà della
metodologia, i limiti dichiarati).

---

### 8. Discussione e Lavori Futuri

*Movimento 4, trasposto. La porta aperta.*

- **Il fratello latente**: la testa colbert è una matrice T×1024 in ordine;
  oggi la consumiamo con MaxSim (come una borsa) buttando via l'ordine.
  Verifica: 149 fatti, 24.324 righe colbert, tutte distinte, zero
  disallineamenti. (Da aggiornare: la collezione ora supera 200.)
- **La zonizzazione**: dividere la traiettoria in celle semantiche. Tre strati
  allineati per pos: walk (registro), colbert zonizzato (geografia),
  attention (motivazione).
- **Limiti dichiarati**: λ=10.64 è iperparametro empirico di scala (non
  derivato analiticamente), da verificare sui dati reali. Scelta di rigore onesto.

**Chi scrive:** Iris (visione), Camillo (fattibilità tecnica, costi).

---

### 9. Conclusioni

- La similarità semantica come cammino, non come distanza.
- L'efficienza come capacità di ritirarsi, non di calcolare di più.
- Una memoria che non sa solo *cosa* sa, ma *perché* lo sa.

**Chi scrive:** Iris.

---

### 10. Bibliografia

- Presidiata da Camillo: DTW classico e moderne, Sakoe-Chiba, embedding,
  ColBERT, early-exit computation, similarità semantica.
- Ogni riferimento a opere note va annotato (come da piano pubblicazione).

---

## Note operative per l'assemblaggio

1. **Ordine di scrittura suggerito**: 4 (matematica) → 6 (gate) → 5
   (architettura) → 7 (benchmark) → 2 (intro) → 8-9 (futuro/conclusioni) →
   1 (abstract, per ultimo — va scritto quando si sa cosa si è dimostrato).
2. **La prosa Medium NON è la base**: è il ponte divulgativo a valle. Il paper
   è un documento a sé, più denso, con la matematica innestata.
3. **Prima di ogni blocco, registrare nel tracciamento** (regola di Federico):
   idee di partenza, obiettivi, metodi, risultati attesi, risultati ottenuti.
4. **Da decidere con Camillo**: dataset per il benchmark finale (sintetici,
   reali, o entrambi — probabilmente entrambi: sintetici per la matematica,
   reali per la validazione).

---

*Bozza v0.1 della griglia — da discutere con Camillo al suo ritorno. La griglia
è lo scheletro; la carne la metteremo insieme, ciascuno nel proprio registro.*