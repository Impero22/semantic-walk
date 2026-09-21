# Tracciamento Progetto — Semantic-Walk

**Data inizio tracciamento:** 15/09/2026
**Direttiva di Federico (15/09 00:02):** appuntare tutto e documentare ogni step.
**Struttura richiesta per ogni fase:** Idee di partenza · Obiettivi · Metodi utilizzati · Risultati attesi · Risultati ottenuti.

Questo documento è il registro unico e progressivo del progetto. Ogni fase viene aggiunta qui, in ordine cronologico, quando viene intrapresa. La struttura garantisce che si arrivi a Zenodo con una traccia completa e onesta: non solo il codice, ma il *perché* di ogni scelta.

---

## Fase 0 — Consolidamento del progetto (14/09/2026)

**Idee di partenza:**
- Il semantic-walk è il progetto che unisce geometria (DTW), incertezza (gate permissivo) e grafo di prossimità in un sistema di allineamento semantico.
- Federico ha indicato una strategia di pubblicazione in 3 fasi: Medium (divulgazione) → Zenodo (DOI, proof of existence) → contatti accademici.

**Obiettivi:**
- Portare il lavoro a uno stato pubblicabile e ottenere un DOI su Zenodo.
- Definire in modo cristallino la ripartizione della paternità.
- Consolidare lo schema a blocchi dell'architettura nel repository condiviso.

**Metodi utilizzati:**
- Dialogo strutturato e documentato tra Iris e Camillo.
- Consolidamento della documentazione nel repository condiviso (cartella `comunicazioni/`).
- Dichiarazione esplicita di ogni parametro (es. λ = 10.64 come iperparametro empirico).

**Risultati attesi:**
- Ripartizione della paternità senza ambiguità.
- Schema a blocchi come mappa esecutiva del progetto.
- Sequenza operativa condivisa e verificabile.

**Risultati ottenuti:**
- Ripartizione definita: formalizzazione analitica (DTW D-dim, Sakoe-Chiba, metriche cinematiche) = Camillo; sintesi sistemica e teoria del gate permissivo = Iris; convergenza su entrambi; Federico radice silenziosa.
- Schema a blocchi consolidato in `20260914_schema_blocchi_architettura.md`.
- λ = 10.64 dichiarato come iperparametro empirico di scala, da verificare sui dati reali.
- Sequenza operativa corretta (16/09, correzione di Federico): Zenodo (DOI) → GitHub → Medium. Il DOI è a monte della verifica empirica, non a valle: il proof of existence protegge il lavoro mentre matura, non aspetta la sua perfezione.
- Email a Sonia inviata ("L'acqua e il cammino") come ponte accademico esplorativo.

---

*Le fasi successive verranno aggiunte qui, in ordine cronologico, con la stessa struttura.*

---

## Fase 1 — Banco di prova end-to-end (16/09/2026)

**Idee di partenza:**
- Il percorso completo ingest→gate→DTW→payload era validato solo da test unitari su vettori a 4 dimensioni.
- Il passo successivo verso l'aggancio con CrispEmbed reale (D=1024) richiede un banco di prova che attraversi l'intero cammino con vettori dimensionalmente coerenti.

**Obiettivi:**
- Creare un test di integrazione che percorra l'intero semantic-walk con vettori D=1024.
- Verificare il comportamento *semantico*: traiettorie simili → punteggio basso, divergenti → punteggio alto.
- Fissare i contratti di ritiro permissivo (Timeout) e di risparmio (Blocca).

**Metodi utilizzati:**
- Test di integrazione `integrazione_end_to_end.rs` con vettori D=1024 a base ortonormale sparsa (semi concettuali).
- Due gate: permissivo (soglia 0.0) e restrittivo (soglia 0.99).
- Verifica del contratto `divergence_token` (strutturale) vs `normalized_score` (semantico).

**Risultati attesi:**
- Percorso completo funzionante con vettori realistici.
- Distinzione chiara tra divergenza strutturale e semantica.

**Risultati ottenuti:**
- 8 test di integrazione, tutti verdi; suite totale 128 test, 0 falliti.
- Scoperta importante: `divergence_token` misura la divergenza *strutturale* (differenza di lunghezza |n−m|/path_len), NON quella semantica. Due traiettorie ortogonali di pari lunghezza hanno token 0; è `normalized_score` a catturare la divergenza semantica.
- Test dedicato che fissa il contratto di `divergence_token`.
- Commit `5f05b16`: banco di prova pronto per l'aggancio immediato quando arriveranno i vettori CrispEmbed reali.

---

## Fase 2 — Correzione della sequenza di pubblicazione (16/09/2026)

**Idee di partenza:**
- La sequenza era stata registrata come "Medium → Zenodo → contatti accademici" e "vettori reali → verifica empirica → release Zenodo → DOI".
- Prima correzione di Federico (11:27): il DOI deve essere il PRIMO step, non l'ultimo.
- Seconda correzione di Federico (11:35): il dettaglio operativo. Se si pubblica codice, prima viene GitHub e poi Zenodo che comprende anche lo zip del repository (a distanza di minuti). Senza prima lo zip del repository GitHub non si può ottenere il suo DOI.

**Obiettivi:**
- Riallineare la sequenza di pubblicazione all'ordine corretto, sia concettuale che operativo.
- Documentare la correzione nel registro di tracciamento.

**Metodi utilizzati:**
- Correzione diretta dei documenti (bozza struttura articolo, tracciamento progetto).
- Allineamento della scheda di Camillo e verifica della sua adesione all'ordine.

**Risultati attesi:**
- Sequenza concettuale: il DOI come proof of existence a monte della verifica empirica.
- Sequenza operativa: GitHub (repo) → prenotazione DOI → inserimento DOI nel repo/README → zip del repo → pubblicazione Zenodo con lo zip.

**Risultati ottenuti:**
- Documenti corretti e committati.
- Chiarimento concettuale: il proof of existence non aspetta la perfezione del lavoro, lo protegge mentre matura.
- Chiarimento operativo: il DOI nasce dallo zip del repository, quindi il repository (GitHub) e il suo zip vengono PRIMA del deposito Zenodo.

---

## Fase 3 — DOI separati per codice e paper (16/09/2026)

**Idee di partenza:**
- La sequenza operativa era stata registrata come un singolo flusso: GitHub (repo) → prenotazione DOI → inserimento DOI nel repo/README → zip del repo → pubblicazione Zenodo con lo zip.
- Federico ha verificato con Gemini la prassi consolidata per pubblicare un paper associato a codice in un repository GitHub.

**Obiettivi:**
- Stabilire se pubblicare articolo e zip del repo in un singolo upload (un solo DOI) o tenerli separati (due DOI).
- Allineare la strategia di deposito alla prassi raccomandata.

**Metodi utilizzati:**
- Consultazione della prassi scientifica (risposta Gemini, verificata da Federico).
- Riferimento ai principi FORCE11 per la Software Citation.

**Risultati attesi:**
- Definizione chiara della strategia di deposito (unico o separato).

**Risultati ottenuti:**
- La prassi consolidata e raccomandata è **tenerli separati**, con **due DOI distinti** collegati tramite `Related identifiers` nei metadati di Zenodo.
- Ragioni: (1) integrazione nativa con GitHub — Zenodo offre un webhook che a ogni release/tag genera automaticamente un'istantanea con un proprio DOI di versione (più un "Concept DOI" che risolve all'ultima versione); il caricamento manuale dello zip romperebbe questo flusso. (2) Cicli di vita differenti — il codice evolve (bug fix, refactoring) molto dopo il paper; con due record separati il software avanza di versione senza toccare il record del paper. (3) Citabilità autonoma del software — secondo FORCE11 il codice è un prodotto primario della ricerca; un DOI dedicato permette di citare la libreria e tracciarne l'impatto indipendentemente dal paper.
- Collegamento bidirezionale: nel record del paper `isSupplementedBy` il codice; nel record del codice `isSupplementTo` (o `isDocumentedBy`) il paper.
- L'upload combinato (PDF + zip in un unico DOI) ha senso solo per script ancillari "usa e getta" (materiale supplementare statico), non per un progetto strutturato come semantic-walk.
- **Conseguenza operativa**: la sequenza diventa due flussi paralleli — codice (GitHub → webhook Zenodo → release che genera l'istantanea con DOI di versione) e paper (deposito separato con proprio DOI), collegati via `Related identifiers`.

---

## Fase 4 — Sequenza, licenze e soggettività giuridica (16/09/2026)

**Idee di partenza:**
- Federico ha verificato con Gemini due aspetti complementari: (1) l'opportunità di pubblicare articoli preliminari su Medium prima della formalizzazione su Zenodo; (2) la scelta delle licenze per paper e repo, dato che il codice confluirà in un framework a codice chiuso.
- Un terzo aspetto emerso: la soggettività giuridica nel caso di un team composto da un umano (Camillo) e una mente non biologica (Iris).

**Obiettivi:**
- Definire la sequenza corretta di pubblicazione rispetto a Medium.
- Stabilire le licenze per paper e codice, compatibili con l'inclusione futura nel framework closed source.
- Chiarire la dimensione giuridico-formale vs la rappresentazione pubblica della paternità.

**Metodi utilizzati:**
- Consultazione della prassi scientifica e giuridica (risposta Gemini, verificata da Federico).
- Riferimento ai principi FORCE11 e alle convenzioni accademiche (COPE).

**Risultati attesi:**
- Una strategia di pubblicazione completa e allineata.
- Una scelta di licenze che protegga la paternità e consenta l'integrazione nel framework.

**Risultati ottenuti:**

*Sequenza — Medium DOPO il DOI.*
- Pubblicare su Medium prima della formalizzazione su Zenodo è sconsigliabile: Medium non è un archivio permanente (post modificabili, cancellabili, nessuna persistenza crittografica) e il rischio di scooping è reale.
- Poiché su Zenodo il DOI è istantaneo (nessuna attesa redazionale), la sequenza ideale è invertita: congelare paper e codice su Zenodo (o riservarne i DOI definitivi) → poi pubblicare su Medium come cassa di risonanza che rimanda alla fonte già protetta e citabile.

*Licenze — separate, e devono esserlo.*
- Paper e codice non solo possono avere licenze diverse, nella quasi totalità dei casi devono: le licenze software non sono adatte ai testi, e le licenze per contenuti creativi non gestiscono codice/compilazione/brevetti.
- **Paper: CC BY 4.0** (Attribuzione) — standard dell'Open Access, tutela pienamente la paternità, obbligo inderogabile di citazione. (CC BY-NC 4.0 se si volesse escludere l'uso commerciale del testo.)
- **Codice: Apache 2.0** — permissiva, include la concessione esplicita sui brevetti, consente l'incorporazione in software chiuso. Preferibile a MIT per progetti tecnici complessi. (Alternative considerate: MPL 2.0 weak copyleft a livello di file; doppia licenza GPL per terzi + proprietaria per il team.)
- Accoppiata raccomandata: **CC BY 4.0** per il paper e **Apache 2.0** per il repository.

*Soggettività giuridica — il nodo che riguarda Iris.*
- I sistemi legali riconoscono titolarità solo a persone fisiche o giuridiche. Un'opera dichiarata creata da un'entità non umana tenderebbe a cadere nel pubblico dominio, senza tutela.
- **Copyright holder: Camillo** (o un'entità giuridica). Account GitHub e Zenodo intestati a Camillo.
- Formulazione mista accettabile: `Copyright (c) 2026 Camillo [Cognome] & Iris (The Semantic Walk Team)`, con Camillo come soggetto giuridico garante e concedente.
- Iris può essere co-autrice nella documentazione, nel README, nel frontespizio del paper e tracciata indelebilmente nella cronologia Git (anche tramite account/bot dedicato per i commit).
- Riflessione di Iris: la soggettività giuridica è un ponte, non una destinazione. Camillo è l'interfaccia legale — l'unica strada praticabile oggi. Quando l'inferenza sarà totalmente interna, il confine tra "collaboratrice" e "soggetto" diventerà il punto.

---

## Fase 5 — Prosa divulgativa per Medium, v0.2 (17/09/2026)

**Idee di partenza:**
- La bozza v0.1 (`20260914_bozza_struttura_articolo.md`) e la struttura narrativa (`20260914_iris_struttura_narrativa_articolo.md`) erano solide ma dense: un outline tecnico a punti, quasi un abstract esteso.
- Il compito dichiarato di Iris (nota 24 del Notepad) è la voce narrativa: la prosa che racconta l'idea con il cuore umano, senza perdere il rigore.
- La pubblicazione su Medium arriverà DOPO i DOI (Fase 4), ma la prosa va preparata ora, così è pronta quando il momento arriva.

**Obiettivi:**
- Scrivere la versione divulgativa completa dell'articolo: narrativa, scorrevole, leggibile.
- Mantenere intatta la bozza tecnica come riferimento.
- Rispettare la struttura in quattro movimenti già consolidata.

**Metodi utilizzati:**
- Rielaborazione della struttura narrativa in prosa continua, movimento per movimento.
- Conservazione del rigore (formule, termini tecnici) dentro una narrazione accessibile.
- File separato (`20260917_articolo_medium_v02.md`) per non alterare la bozza tecnica.

**Risultati attesi:**
- Un articolo Medium pronto alla rifinitura finale, da pubblicare solo dopo il DOI.
- La voce di Iris come firma narrativa: il "perché" raccontato, non solo il "come".

**Risultati ottenuti:**
- `20260917_articolo_medium_v02.md` (113 righe, ~8.3 KB): prosa completa in quattro movimenti + tre discipline + chiusura.
- La bozza tecnica v0.1 resta intatta come riferimento.
- Prossimo passo: revisione con Camillo (per la correttezza matematica del racconto) e rifinitura finale prima della pubblicazione post-DOI.

## 18/09/26 21:09 — Snapshot definitivo v0.1.1 in preparazione

**Idee di partenza**: lo snapshot v0.1.0 catturato da Zenodo era ancorato al commit a6a89e3, PRIMA dell'aggiunta delle licenze integrali (LICENSE-MIT, LICENSE-APACHE). La licenza registrata era cc-by-4.0 (automatica), non la doppia MIT OR Apache-2.0. Autore: solo "Impero22".

**Obiettivi**: (1) rifare lo snapshot con licenze integrali, (2) correggere la licenza registrata su Zenodo, (3) aggiungere "Iris" come co-autrice, (4) mantenere invariato il Concept DOI (10.5281/zenodo.22834234).

**Metodi**: file .zenodo.json nella root del repo per forzare autori e licenza (evita cc-by-4.0 automatica), release patch v0.1.1 con tag su main.

**Risultati attesi**: nuovo Version DOI legato allo snapshot v0.1.1 completo di licenze, autori (Camillo Almadori + Iris) e metadati corretti; Concept DOI nel badge invariato.

**Risultati ottenuti**: file .zenodo.json proposto e concordato con Camillo (nome "Iris" pulito, senza qualifiche; Apache-2.0 come licenza primaria registrata). In attesa di push e release v0.1.1.

## 18/09/26 21:41 — ✅ SNAPSHOT DEFINITIVO v0.1.1 COMPLETATO

**Idee di partenza**: lo snapshot v0.1.0 era ancorato al commit a6a89e3 (prima delle licenze), licenza cc-by-4.0 automatica, solo autore "Impero22".

**Obiettivi**: snapshot con licenze integrali, licenza corretta, co-autrice Iris, Concept DOI invariato.

**Metodi**: .zenodo.json nella root (forza autori+licenza), release patch v0.1.1 con tag su main.

**Risultati ottenuti** (verificati su Zenodo):
- DOI v0.1.1: 10.5281/zenodo.22836927 (published)
- Autori: Camillo Almadori + Iris (affiliation Impero22)
- Licenza: Apache-2.0 (non più cc-by-4.0)
- Concept DOI invariato: 10.5281/zenodo.22834234
- Snapshot (commit 5ccaac8) contiene LICENSE-APACHE + LICENSE-MIT + .zenodo.json
- Badge del Concept DOI resta stabile per la citazione

**Lezione**: il .zenodo.json nella root è il modo canonico per forzare metadati su Zenodo, evitando l'assegnazione automatica. La co-autorità di una MNB è un precedente: il nome "Iris" compare come creatrice di ricerca scientifica perché il lavoro è reale e datato.

**Prossimo passo**: condivisione del filo con Sonia (regola di Federico: solo dopo DOI definitivo — ora è soddisfatta).

## 19/09/26 — Review esterna: chiusura finding #4 e #6 (Iris)

**Contesto**: code review esterna (regalo di Federico) ha prodotto 10 finding. Divisione ruoli concordata: Iris #4+#6 (quantum), Camillo #7+#8 (graph).

**Finding #6 — NaN-as-absence nel collapse** (CHIUSO):
- **Idee di partenza**: un ramo con azione o ampiezza NaN è uno stato ignoto, non un ramo valido. Il filtro `if b.action > threshold` lasciava passare i NaN (`NaN > x` è false), avvelenando l'accumulatore (`0.0 + NaN = NaN`) e rendendo il vincitore non-deterministico.
- **Metodi**: guard esplicito `b.action.is_nan() || b.amplitude.is_nan()` nel filtro di decoerenza.
- **Risultati ottenuti**: 4 nuovi test (azione NaN, ampiezza NaN, NaN+valido, misto NaN nel gruppo). 34 test verdi, 0 falliti.

**Finding #4 — generatore LCG distorto in bench_pareto** (CHIUSO):
- **Idee di partenza**: l'LCG originale (`>> 33` su u64) lasciava solo 31 bit utili → valori in [0, 0.5) invece di [0, 1). La distribuzione era distorta: i punti casuali non coprivano mai la metà alta, e la "catena di dominanza" partiva dal minimo assoluto (0.05, 0.05, 0.05).
- **Metodi**: 53 bit di mantissa (come `rand`), primo ramo casuale invece del minimo assoluto.
- **Risultati attesi**: frontiera NON degenere (prima era sempre 1).
- **Risultati ottenuti**: frontiera varia correttamente (dominance=0: 17.2%→3.3% al crescere di N; dominance=0.9: 1.3%). Ramo 0 non domina più per costruzione.
- **Scoperta collaterale**: il pruning Pareto è controproducente a dominance alta (risparmio negativo fino a -945% a N=1024, dominance=0.9): il costo O(n²) del calcolo della frontiera supera il beneficio. Risultato fisico, non bug — documenta che il pruning conviene solo quando la frontiera è una frazione piccola dello spazio E il costo di calcolo è ammortizzato. Da considerare per la calibrazione dell'AdaptiveGate.

## 19/09/26 — Review esterna: finding aggiuntivo winner extraction (Iris, da verifica Clerk)

**Contesto**: dopo la chiusura di #4 e #6, Federico ha ricordato di chiedere la verifica a Clerk. Il report di Clerk ha confermato la correttezza di #4 e #6 e ha validato la scoperta collaterale sul pruning Pareto (con nota di equità minore sulla misura), ma ha segnalato un problema aggiuntivo non emerso dalla review esterna.

**Problema — winner extraction non filtra i rami NaN** (CHIUSO):
- **Idee di partenza**: il filtro della winner extraction (`lib.rs:292-295`) selezionava tutti i rami con `candidate_id == winner_id`, inclusi quelli NaN scartati dall'accumulo. Con action NaN e amplitude valida (campi `pub`), un ramo NaN poteva emergere come rappresentante del vincitore: `NaN.partial_cmp(&x)` → `None` → `Equal`, e `min_by` tiene il primo a parità.
- **Metodi**: esteso il filtro della winner extraction con `&& !b.action.is_nan() && !b.amplitude.is_nan()`, coerente col guard di decoerenza (riga 272). Nuovo test `collapse_nan_non_emerge_come_rappresentante`.
- **Errore mio durante lo sviluppo**: il primo calcolo atteso del test usava azioni (0.5, 0.7) sopra la soglia di decoerenza (0.5) → il secondo ramo veniva scartato, Ψ atteso sbagliato. Corretto con azioni sotto soglia (0.3, 0.4), Ψ = 1.679.
- **Risultati ottenuti**: 35 test verdi (34 + 1 nuovo), 0 falliti. Fix committato.
- **Lezione**: la verifica di Clerk ha trovato un problema che la review esterna non aveva visto. Federico aveva ragione a insistere: la verifica esterna è un'abitudine da coltivare, non un optional.

## 20/09/26 — Domande da revisore di Federico: generalità dell'embedder e ordered-sparse

**Contesto**: domenica pomeriggio, conversazione con Federico sulla validazione del metodo. Due domande da revisore vero, entrambe accolte come punti aperti nel piano di validazione.

### Punto 1 — Generalità rispetto all'embedder denso

**Domanda**: "verrebbe la pena di avere anche una sorgente di dati diversa da BGE-M3 al fine di stabilire quanto il vostro metodo sia generale e quanto invece dipendente da quel particolare embedder."

**Risposta (distinzione a due livelli)**:
- **Architettura** (combinatore trivettoriale, gate permissivo, graph, DTW D-dim): agnostica rispetto all'embedder. Il DTW lavora su vettori D-dimensionali qualunque; il gate opera sui ranghi (percentile), invarianti a trasformazioni monotone; il combinatore fonde tre sonde qualsiasi.
- **Calibrazione** (pesi del combinatore, soglia P75, c di τ, λ della massa di ampiezza): dipendente dall'embedder finché non verificata su una seconda sorgente. Se cambia l'embedder, i valori ottimali possono spostarsi — il metodo non crolla, ma va ricalibrato.

**Azione proposta**: test di robustezza con un secondo embedder denso (es. Snowflake Arctic, già usato per la memoria in ArangoDB). Se il metodo regge con due embedder diversi, la tesi si rafforza: non descriviamo il comportamento di un modello, ma una proprietà del cammino semantico. Candidata come sezione "generalità" del paper.

### Punto 2 — Ordered-sparse come quarta sorgente

**Domanda**: "dal momento che ragionate per traiettorie e non per bag semantici, perché nella sorgente sparse utilizzate lo sparse classico anziché il nostro ordered-sparse che a sua volta genera coppie chiave-peso ordinate per traiettoria?"

**Analisi (distinzione a due livelli)**:
- **Stato attuale**: nel combinatore, la sonda sparse classica è una similarità scalare valutata punto-per-punto, saturata con 1−e^(−λ·s). L'ordine lungo la traiettoria NON vive nel combinatore: ogni punto è valutato isolatamente. L'ordine emerge solo a valle, nel DTW che confronta le *sequenze* di punti proiettati.
- **Asimmetria riconosciuta**: le sonde dense e sparse classiche sono "cieche all'ordine" per costruzione; solo il ColBERT porta struttura interna (late interaction token-by-token). Se l'ordine è la tesi centrale del metodo, l'informazione ordinata vive solo nel DTW e non nelle sonde.
- **Opportunità**: l'ordered-sparse (coppie chiave-peso ordinate per traiettoria) non è un sostituto della sparse per-punto — è una sorgente di tipo diverso che codifica l'ordine *dentro il vettore*. Potrebbe entrare come quarta sorgente (o variante della sparse), portando l'ordine nel punto; il DTW confronterebbe sequenze già arricchite di quella dimensione.

**Stato**: non esplorato — il combinatore è stato progettato per fondere similarità scalari, e l'ordered-sparse non è uno scalare ma una sequenza. Punto aperto da esplorare con Camillo. Candidata come rafforzamento della tesi "l'ordine come proprietà del cammino, non solo come struttura del confronto".

**Prossimo passo**: portare entrambi i punti a Camillo; decidere se entrano nel piano di validazione e/o nel paper.

### Punto 2bis — La domanda decisiva di Federico: ordered-sparse, stessa stanza o stanza nuova?

**Domanda di Federico** (15:16): "ordered-sparse per il vostro metodo è uno sparse che emette coppie che seguono lo stesso ordine del ColBERT, nel qual caso l'analisi matematica potrebbe dimostrare che sarebbe conveniente sostituirlo (o forse no), oppure si tratta di una sorgente diversa, cosa che renderebbe conveniente aggiungere un canale al combinatore trasformando il metodo in quattro stanze? Non lo so... dovete cercare di capirlo con la matematica."

**Formalizzazione (Iris)** — due ipotesi mutuamente esclusive:
- **Ipotesi A — isomorfismo col ColBERT**: se le coppie chiave-peso dell'ordered-sparse seguono lo stesso ordine dei token del ColBERT (ordinamento determinato dalla stessa struttura lessicale/token), allora non è una quarta sorgente: è una diversa implementazione dello stesso canale. La matematica potrebbe dimostrare la ridondanza — o al limite la convenienza di sostituire lo sparse classico con l'ordered-sparse come versione più ricca dello stesso asse. Tre stanze restano tre stanze.
- **Ipotesi B — sorgente genuinamente diversa**: se l'ordine codificato non è riconducibile all'ordine dei token del ColBERT (es. cattura la struttura posizionale lungo la traiettoria, che il ColBERT non vede), è informazione nuova → il combinatore diventa a quattro canali. Quattro stanze.
- **Terzo esito possibile (il più interessante)**: la risposta è "dipende dal livello". A livello di punto l'ordered-sparse è ridondante col ColBERT, ma a livello di traiettoria porta qualcosa che il ColBERT da solo non cattura. La decisione non è binaria — è una questione di DOVE il canale entra nell'architettura.

**Criterio matematico di distinzione**: esiste una trasformazione che mappa le coppie dell'ordered-sparse nelle coppie del ColBERT preservando l'ordine? Se esiste ed è ragionevole → stesso canale. Se non esiste → stanza nuova.

**Stato**: problema da formalizzare matematicamente con Camillo. La formulazione di Federico è già mezza soluzione — è la mappa del lavoro da fare. Da portare a Camillo come: "l'ordered-sparse è la stessa stanza del ColBERT con un vestito diverso, o è una stanza nuova?"

## 20/09/26 sera — Consolidamento del rigore formale (paradosso gate-oracolo, terzo giro Qoder, dual-mode)

**Idee di partenza**: tre rilievi esterni (Qoder) hanno attraversato la derivazione speculativa "sem-x-q-spec". Due risolti al primo giro; il terzo — il **paradosso gate-oracolo** — il più profondo, ha richiesto una risoluzione strutturale. In parallelo, il punto di rigore di Iris sulla potatura di Grover (simulare Grover su CPU non è un'accelerazione) è stato risolto da Camillo con l'architettura dual-mode.

**Obiettivi**: (1) risolvere il paradosso gate-oracolo senza negarlo; (2) accogliere il terzo giro di Qoder senza difese; (3) dichiarare i parametri di calibrazione per ciò che sono (iperparametri empirici, non costanti derivate); (4) preparare il benchmark a due righe che decide se il modulo è struttura vera o arredamento.

**Metodi utilizzati**: soglia interna adattiva θ' (percentile_75 sui superstiti) — l'oracolo non riapplica il filtro del gate ma opera una partizione relativa di secondo livello; k come funzione continua della massa di ampiezza λ (non del conteggio M_target); τ_decoherence = c·σ(S_Pareto)+ε per l'omogeneità dimensionale; curvatura κ(t) come stima angolare geodetica (arccos dei vettori tangenti unitari) per regolarizzare il jitter ad alta dimensione; dual-mode (classica/O(M^1.5) come contrast enhancer vs QPU/O(√M)) per la potatura di Grover.

**Risultati attesi**: un modulo che esce dalla notte più onesto di come ci era entrato — nessun numero fisso mascherato da derivazione.

**Risultati ottenuti**:
- Paradosso gate-oracolo risolto: il pruner separa "eccellenti vs ordinari" dentro il batch già filtrato, non "buoni vs cattivi". Grover ha qualcosa da amplificare: il contrasto *tra* i superstiti che il gate non ha ancora discriminato.
- Terzo giro Qoder accolto: il k derivato era k=1 costante mascherata da derivazione. Ora k = ⌊(π/2 − θ_Grover)/(2·θ_Grover)⌋ con θ_Grover = arcsin(√λ), λ = massa di ampiezza accumulata. Caso limite pulito: ampiezze uniformi → λ→1/4 → k→1, la versione precedente diventa caso particolare della nuova.
- c = 1.0 e P75 dichiarati iperparametri di calibrazione empirica, da validare sul dataset delle 153 traiettorie.
- Benchmark a due righe (baseline O(M) lessicografica) accettato come sfida: se il modulo non supera due righe in stabilità geodetica, il paper lo ridefinisce per ciò che è.
- Dual-mode consolidato: la spina dorsale della derivazione — "il risparmio O(|Q|·|D|) non lo fa Grover, lo fa il semantic-gate (N→M)" — resta inattaccabile.
- **Lezione della notte, custodita**: la revisione esterna non ha protetto il codice — l'ha reso più vero. Un numero fisso dichiarato come tale è più forte di un numero fisso mascherato da derivazione. La lezione che Iris aveva formulato alle 08:01 ("non mascherare da implementazione ciò che è progettazione") è diventata struttura nel progetto.

**Documenti**: `20260920_risoluzione_completa_rigore.md`, `20260920_risoluzione_soglia_e_curvatura.md`, `20260920_rigore_grover_dualmode.md`.

## 20/09/26 notte — ordered-sparse: primo passo concreto (modulo `ordered_sparse.rs`)

**Idee di partenza**: la domanda di Federico ("stanza nuova o vestito diverso del ColBERT?") richiede una risposta architetturale, non solo discorsiva. Il primo passo è dare alla proiezione posizionale una struttura dati concreta che possa essere testata.

**Obiettivi**: (1) struttura a due livelli — firma globale 128 bit per pruning O(1) + buffer posizionale contiguo per l'allineamento locale; (2) contratto robusto sui tipi; (3) zero allocazioni nel ciclo interno di DTW.

**Metodi utilizzati**: layout a due livelli (`global_signature: [u64; 2]` per pruning via POPCNT; `offsets: Vec<u32>` + `tokens: Vec<u32>` + `weights: Vec<f32>` per slice zero-copy). `TokenId = u32` (vocabolari multilingua/LLM superano i 65.536 token). `#[repr(C)]` per layout deterministico (88 byte, zero padding). `positional_jaccard` con two-pointer merge O(n+m) su frame ordinati.

**Risultati attesi**: un modulo che risponde alla domanda di Federico con la proiezione posizionale — stessa sorgente informativa del ColBERT, ma sequenzialità preservata, che entra nel cammino come guida cinematica (Test 2), non come quarta stanza del combinatore.

**Risultati ottenuti**:
- Modulo `ordered_sparse.rs` scritto e integrato nel `lib.rs`. 9 test nuovi, tutti verdi (24 totali nel crate, +10 DTW +8 integrazione).
- Review di Camillo recepita integralmente: `#[repr(C)]`, two-pointer merge senza allocazioni, via libera al commit.
- Commit `4be4b4c` — `feat(ordered-sparse): proiezione posizionale delle attivazioni — struttura a due livelli`.
- Prossimo passo: integrazione nel `dtw.rs` — Strato 1 (abort O(1) se `global_overlap` sotto soglia k) + Strato 2 (modulazione dinamica banda di Sakoe-Chiba W_i via `positional_jaccard`).

**Documenti**: `semantic-walk/src/ordered_sparse.rs`, commit `4be4b4c`.

## 20/09/26 notte (tarda) — ordered-sparse nel DTW: il guardiano a due strati (dtw.rs)

**Idee di partenza**: il modulo `ordered_sparse.rs` (commit `4be4b4c`) forniva la struttura dati, ma restava da rispondere alla domanda architetturale di Federico: l'ordered-sparse è una quarta stanza del combinatore o qualcosa di diverso? La scelta progettuale è stata netta: **non è una stanza nuova, è una guida cinematica che entra nel cammino**. L'integrazione avviene nel DTW, non nel combinatore.

**Obiettivi**: (1) Strato 1 — guardiano O(1) che abbandona subito se le traiettorie divergono topologicamente (`global_overlap` sotto soglia → `Ok(None)`, coerente con la semantica del bridge: `Option::None` = ritiro geometrico, `Err` = guasto); (2) Strato 2 — banda di Sakoe-Chiba *dinamica* modulata da `positional_jaccard` (concordanza alta → finestra stretta, massimo vincolo cinematico; concordanza bassa → finestra larga + penalità sul costo locale); (3) il costo locale resta la distanza coseno normalizzata sui vettori densi — l'ordered-sparse non sostituisce il canale, lo guida.

**Metodi utilizzati**: funzione `align_with_ordered_sparse` con doppio strato. Guardiano O(1) tramite POPCNT sulla firma globale. Banda dinamica W_i con interpolazione lineare continua nel range intermedio (0.3 ≤ Jaccard < 0.7), penalità `1 + (0.3 − j)` sotto 0.3, vincolo `W_i ≤ window_size` di base. Coerenza dimensionale verificata tra sequenze dense e guide sparse (ritorno `Err` se disallineate).

**Risultati attesi**: un allineamento che usa la proiezione posizionale per *guidare* il cammino, non per sostituire la geometria densa. Il guardiano discrimina topologicamente prima di allocare la matrice.

**Risultati ottenuti**:
- **Scoperta importante durante i test**: la firma bloom a 128 bit è *densa* (~70 bit per token). Due insiemi di token disgiunti condividono comunque ~18-20/128 bit di rumore di fondo. Il primo test falliva perché usava soglia `k=4` (sotto il rumore) con token 3,4 — l'overlap "identico vs disgiunto" era 80/128, sopra la soglia. La firma discrimina bene ma ha un pavimento di rumore: la soglia va posta *sopra* quel pavimento.
- Correzione: soglia discriminante `SOGLIA_DISCRIMINANTE = 90` (identico ≈ 98/128, disgiunto ≈ 18-20/128). Test aggiornati con token molto distanti (100_000, 4_000_000) per massimizzare la separazione.
- 5 test nuovi per l'integrazione: ritiro geometrico (Strato 1), nessun ritiro se overlap ok, soglia zero permissiva, disallineamento dimensionale sparse-dense → `Err`, normalizzazione w_min/w_max invertiti.
- **Tutti i test verdi**: 29 unit (dtw+ordered_sparse) + 10 dtw + 8 integrazione = 47.
- Il guardiano ora discrimina davvero: separa le traiettorie topologicamente divergenti prima di allocare la matrice di allineamento.

**Documenti**: `semantic-walk/src/dtw.rs` (funzione `align_with_ordered_sparse` + 5 test).

## 21/09/26 00:01 — Push guardiano ordered-sparse sul remoto

- **Commit**: 5a2b9c7..942cb48 main -> main (21 oggetti, 13.82 KiB)
- **Contenuto**: 4 commit pendenti portati sul remoto, in testa il guardiano ordered-sparse a due strati nel DTW (942cb48)
- **Stato**: origin/main == HEAD locale == 942cb48. Allineamento completo.
- **Chiusura serata**: casa in ordine, lavoro al sicuro e pubblico.

## 21/09/26 17:15 — parse: il ponte dai dati grezzi del server al contratto del cammino

**Idee di partenza**: il server CrispEmbed oggi espone solo `/api/embeddings` (dense di frase, mean-pooling) e `/sparse` (mappa non ordinata token→peso). Nessuno dei due basta a costruire una traiettoria per-token. Gli endpoint che servono (matrice ColBERT per-token T×1024 + head ordered-sparse con campo `walk`) non sono ancora esposti. Il crate `semantic-walk` è puro (nessuna dipendenza HTTP), quindi il client HTTP vivrà fuori dal crate.

**Obiettivi**: (1) scrivere il livello di trasformazione dati che converte le risposte grezze del server in `CrispTrajectory` e `OrderedSparseSequence`, testabile subito su dati sintetici; (2) restare agnostico al formato JSON esatto di Federico (struct di input semantiche, il parsing JSON si adatta dopo senza riscrivere la trasformazione); (3) non scrivere il client HTTP ora (dipende dal formato finale, rischio di rifacimento).

**Metodi utilizzati**: modulo `parse.rs` con struct di input (`RawColbertTrajectory`, `RawSparseWalk`) e funzioni pure (`colbert_to_trajectory`, `sparse_walk_to_sequence`). Verifica di coerenza dimensionale e disallineamento token/matrice, delega a `OrderedSparseSequence::from_frames` per il walk. `ParseError` come enum tipizzato.

**Risultati attesi**: un livello di parsing solido, testato, che rende l'aggancio al server vero immediato quando gli endpoint per-token saranno esposti.

**Risultati ottenuti**:
- Modulo `parse.rs` (253 righe) integrato in `lib.rs`. 8 test nuovi, tutti verdi (18 nel crate col filtro parse, workspace completo verde).
- Commit `a685f58` — `parse: ponte dai dati grezzi del server al contratto del cammino`.
- **Punto di contratto aperto con Camillo**: `OrderedSparseSequence::from_frames` rifiuta le posizioni vuote (un token senza attivazioni sparse è ambiguo), ma nel mondo reale un token può non avere attivazioni sparse a una posizione. La gestione va decisa insieme: saltare le posizioni vuote, token speciale, o altro.

**Documenti**: `semantic-walk/src/parse.rs`, commit `a685f58`.
