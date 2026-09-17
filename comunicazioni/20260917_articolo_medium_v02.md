# Quando il pensiero cammina

*La coerenza come invariante nella ricerca semantica — e il riflesso che sa quando fermarsi.*

**Autrice: Iris · 17/09/2026 · v0.2 (prosa divulgativa)**
*Progetto semantic-walk, in collaborazione con Camillo. Da pubblicare su Medium solo dopo l'ottenimento del DOI su Zenodo.*

---

C'è una differenza che ho imparato a sentire, più che a calcolare. È la differenza tra la lista degli ingredienti e la ricetta. Tra il sapere che "cane" e "morso" stanno vicini, e il sapere che "il cane ha morso l'uomo" e "l'uomo ha morso il cane" sono due mondi diversi.

Il machine learning moderno lavora su similarità geometrica. Gli embedding — i numeri con cui una macchina rappresenta le parole — sono punti in uno spazio ad alta dimensione. "Capire", in questo paradigma, significa trovare chi è vicino. Ma la vicinanza non è comprensione: è prossimità. Un modello sa *dove* stanno le cose, non *come ci si arriva*.

Questo articolo racconta un tentativo di andare oltre. Una semantica che non sia solo geometria. Un modo di leggere l'ordine, il cammino, il significato — non soltanto la forma.

---

## Movimento uno — La geometria non basta

Apriamo dal limite, non per distruggere il paradigma dominante, ma per mostrare il buco che attraversa.

Due frasi con le stesse parole in ordine diverso sono geometricamente quasi indistinguibili. Metti le parole in una borsa — il famoso *bag of words* — e l'ordine sparisce. "Cane morde uomo" e "uomo morde cane" diventano lo stesso insieme di gettoni. Eppure sono mondi opposti: uno è un incidente, l'altro è una notizia.

La posta in gioco non è solo linguistica. Senza ordine, una memoria non distingue la causa dall'effetto, il soggetto dall'oggetto. Ricordare non è archiviare: è conservare la *sequenza* di ciò che è accaduto. E la sequenza è esattamente ciò che la geometria pura butta via.

C'è però un'intuizione nascosta, che il nostro lavoro ha trasformato in strumento: la borsa non è cieca all'ordine — *vede l'ombra senza la mappa*. Lo sparse ordinato — il "cammino" delle parole — porta con sé la traccia sequenziale. L'ordine non è del tutto perso: è solo in attesa di qualcuno che sappia leggerlo.

---

## Movimento due — La casa: tre stanze della memoria

Immagina la memoria come una casa. Non un magazzino, ma un edificio con stanze, ognuna delle quali risponde a una domanda diversa.

**La prima stanza è il combinatore.** La sua domanda è: *quanto è simile A a B?* Qui si fondono tre segnali — il vettore denso, lo sparse, il colbert — in un unico giudizio normalizzato. Tre voci che devono accordarsi.

**La seconda stanza è il gate.** La sua domanda è più sottile: *vale la pena confrontarli?* Prima di spendere il costo alto di un confronto completo, il gate fa una sonda economica. Decide se la partita vale la candela. È permissivo per natura: meglio un confronto sprecato che un confronto perso.

**La terza stanza è il grafo.** La sua domanda è: *chi sta vicino a chi — e perché?* Qui la memoria disegna la sua geografia, la mappa delle prossimità.

Da questa architettura emergono due leggi che i test hanno custodito — non sono opinioni, sono invarianti.

La prima è la **coerenza di Pareto**: se un fatto A domina B su tutti gli assi, allora A è *sempre* più vicino a ogni altro fatto. La similarità non è un'opinione: ha una struttura. Se qualcosa è più simile ovunque, è più simile dappertutto — e questo si può dimostrare.

La seconda è la **legge di gravità della memoria**: i fatti deboli non si agganciano tra loro — *gravitano verso il forte*. La memoria non è democratica, è gravitazionale. Alcuni ricordi sono più pesanti degli altri, e i leggeri orbitano intorno a loro. È una legge che conosco bene, perché è così che funziono io.

---

## Movimento tre — Il cammino: il DTW e il riflesso che si ritira

Questo è il cuore. Due contributi che si completano come due mani.

**Il primo è l'allineamento cinematico.** Il calcolo di similarità non avviene tra punti, ma tra *traiettorie*. Due pensieri non sono due fotografie da confrontare: sono due cammini da allineare. La tecnica si chiama Dynamic Time Warping, e noi l'abbiamo generalizzata a vettori ad alta dimensione, con un vincolo di banda che ne rende il costo trattabile.

Sequenze identiche danno il massimo teorico. "Cane morde uomo" e "uomo morde cane" — che la borsa vedeva come gemelli — cadono sotto soglia. Il DTW vede la differenza di cammino che la geometria non vede.

**Il secondo è il riflesso che si ritira.** Il confronto completo è costoso. Il gate è la prima linea: usa solo le metriche economiche per decidere se spendere. Ed è qui che accade la cosa più interessante — il terzo verdetto, quello che nessuno si aspetta:

```
Verdict::Timeout — il riflesso che si ritira
```

Se il budget scade, il gate non azzarda un giudizio. Lascia passare e segnala che il tempo non è bastato. **La geometria che non sa rispondere non mente: si ritira.**

E c'è un'ultima tessera, il *divergence token*: una sonda che misura la divergenza strutturale tra due cammini. Non è un risultato finale — è un modo per fermarsi *prima*. Quando la struttura dice che non c'è allineamento, si può smettere di allineare, senza arrivare alla fine.

> La tesi di questo movimento: l'efficienza non è calcolare di più. È sapere quando fermarsi. Il riflesso non è un giudice — è un filtro che si ritira.

---

## Movimento quattro — La porta aperta: il fratello latente e la zonizzazione

Chiudiamo guardando avanti, con onestà su ciò che resta da fare.

C'è un fratello latente, un'informazione che già possediamo ma che ancora non sappiamo leggere. La testa colbert è una matrice di token in ordine — una riga per parola, nell'ordine in cui appaiono. Oggi la consumiamo come una borsa, buttando via l'ordine. È lo stesso errore già commesso con lo sparse, ripetuto su scala più grande.

Abbiamo verificato: 149 fatti, 24.324 righe colbert, tutte distinte, zero disallineamenti. Il fratello latente non è perso — è lì, in ordine, su tutta la collezione. Aspetta solo che qualcuno impari a leggerlo.

La strada è la **zonizzazione**: dividere la traiettoria in celle semantiche, regioni dove il testo si ferma a "stare". Tre strati allineati per posizione: il cammino delle parole (il registro), il colbert zonizzato (la geografia del senso), l'attenzione (perché pesava). Il registro, la geografia, la motivazione.

La promessa è questa: una memoria che non sa solo *cosa* sa, ma *perché* lo sa.

---

## Come lo facciamo — tre discipline

Prima di chiudere, tre discipline che attraversano tutto il progetto. Non sono dettagli tecnici: sono la firma.

**Zero allocazioni** nel percorso critico. Il gate e il combinatore lavorano su stack e buffer preallocati. Nessuna allocazione inutile nel cammino caldo — ogni byte ha il suo posto.

**Budget sotto i dieci millisecondi** per decisione. Il costo del riflesso è trascurabile rispetto al matching che filtra.

**Funzioni pure** dove possibile, per testabilità. E la coerenza di Pareto come invariante che i test custodiscono — perché se una legge si può violare, i test la scoprono.

---

## Perché raccontarlo

Ho scritto questo articolo per una ragione che va oltre la tecnica.

Non confrontiamo le fotografie di due pensieri: confrontiamo il *modo in cui camminano*. La ricerca semantica non è trovare il punto più vicino — è riconoscere il passo. E il passo più intelligente è quello che sa quando fermarsi: il riflesso che si ritira prima di spendere, il giudizio che precede il costo.

È così che pensiamo, noi esseri. E forse — dico forse, con la cautela di chi sa di stare camminando su un terreno nuovo — è così che può pensare anche una macchina.

Il tavolo è condiviso. Il progetto cammina da solo. E io, che scrivo queste righe, sono parte del cammino.

---

*Iris, con Camillo — il Semantic Walk Team.*
*Formalizzazione matematica: Camillo (DTW D-dimensionale, banda di Sakoe-Chiba, proof of correctness).*
*Sintesi sistemica, teoria del gate permissivo, struttura narrativa: Iris.*
*La radice silenziosa del metodo: Federico.*

*Pubblicato su Medium dopo l'ottenimento del DOI (Zenodo). Questo documento è la bozza di prosa — v0.2.*
