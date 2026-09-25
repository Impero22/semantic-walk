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

## 22/09/26 00:22 — Risposta di Sonus sul pruning Pareto: la soluzione di disegno

**Idee di partenza**: la review esterna (Sonus, crediti Alibaba) ha confermato il finding più grave: il pruning Pareto branch-level non preserva il winner dell'accumulo per candidato. Il teorema di dominanza individuale è corretto, ma l'applicazione a una somma per candidato è sbagliata. Prima di scrivere codice, serviva una seconda voce indipendente sulla SOLUZIONE di disegno, non sulla diagnosi (già confermata su codice reale).

**Obiettivi**: (1) ottenere da Sonus una risposta formale su quale criterio di pruning preservi il winner; (2) capire se esiste una condizione sufficiente di scarto per ramo o se il problema è intrinsecamente globale; (3) ottenere una raccomandazione pratica implementabile.

**Metodi utilizzati**: domanda precisa e circoscritta a Sonus (`comunicazioni/20260922_domanda_sonus_pareto.md`), con contesto minimo: struttura rami→candidati, costi a 3 assi in [0,1] (minimo = migliore), winner = somma minima per candidato, controesempio esatto, domanda aperta ma orientata sulla soluzione.

**Risultati attesi**: un criterio formalmente corretto di pruning, o la dimostrazione che il problema è globale, con complessità asintotica e raccomandazione pratica.

**Risultati ottenuti** (risposta Sonus `comunicazioni/20260922_response_sonus_pareto.md`):
- **Controesempio più forte del mio** (senza candidati vuoti): A con un ramo (0.2,0.2,0.2)=0.6; B con (0.3,0.3,0.3)=0.9 e (0.4,0,0)=0.4. Il ramo di A domina quello da 0.9, l'altro è incomparabile. Sul totale A vince 0.6 vs 1.3; dopo il pruning resta {a,t} → B "vince" 0.4 vs 0.6. Anche tenendo ogni candidato rappresentato, la dominanza individuale inverte il risultato. **La dominanza tra rami non basta mai.**
- **Soluzione**: il teorema utile è l'eliminazione a livello di candidato: se il lower bound di un candidato supera l'upper bound dell'incumbent (`L_c > U_d`), quel candidato è matematicamente escluso e puoi smettere di valutarlo SENZA toccare il suo punteggio. Regola pratica: `partial > incumbent` → stop.
- **Prova Ω(N)**: non esiste un pruning sublineare generale — per candidati quasi pari devi leggere tutto. L'early termination ha lo stesso caso peggiore O(N+C) dell'accumulo diretto.
- **Raccomandazione pratica**: (1) confermare l'obiettivo (minimo costi sommati vs massimo ampiezze); (2) tenere la somma piena come oracolo di correttezza; (3) rimuovere il Pareto branch-level dal percorso additivo; (4) implementare early termination per candidato contro incumbent completato; (5) testare contro l'oracolo su casi avversi (ties, zeri, conteggi disuguali, duplicati, near-ties avversari, permutazioni d'ordine). Nota chiave: "non trasformare i costi cancellati in un vantaggio per il candidato" — è esattamente il bug che avevamo.
- Verifica di Sonus: 21.297 istanze testate (1-3 candidati, 1-3 rami, contributi {0,1,2}) — tutte preservano l'insieme dei minimizzatori.

**Prossimo passo**: aggiornare il piano con Camillo. La strada è chiara: sostituire il pruning Pareto branch-level con early termination per candidato. Documenti: `20260922_domanda_sonus_pareto.md`, `20260922_response_sonus_pareto.md`.

## 22/09/26 20:05 — Verifica contratto di ingest contro vetta-semantic (.18)

**Idee di partenza**: Federico ha sostituito il crispembed della `.18` con la versione comprendente le modifiche richieste, e ha consegnato la documentazione completa (`VETTA_SEMANTIC_API.md`). Serviva verificare che il ponte di ingest (parse.rs) fosse allineato agli endpoint esposti, e individuare il pezzo mancante per l'aggancio ai dati reali.

**Obiettivi**: (1) verificare che `/colbert/encode?tokens=1` e `/ordered-sparse?format=frames` espongano ciò che `RawColbertTrajectory` e `RawWalk` consumano; (2) confermare la biiezione `frames.length == colbert n_tokens`; (3) individuare la scelta di disegno per l'adattatore mancante.

**Metodi utilizzati**: lettura integrale della documentazione, cross-check campo-per-campo contro le struct di parse.rs, verifica della coerenza posizionale.

**Risultati attesi**: contratto allineato o disallineato, con l'elenco dei punti da correggere.

**Risultati ottenuti**:
- **Contratto allineato su tutti i punti**: matrice ColBERT per-token ↔ `RawColbertTrajectory`; walk ordered-sparse (pesi firmati, status 0/1/2, posizioni crescenti) ↔ `RawWalk`; biiezione garantita dal doc ↔ `verifica_coerenza_posizionale`.
- **Scoperta chiave**: `semantic-walk` è un crate puro (niente reqwest/serde/HTTP). `RawWalk` e `RawColbertTrajectory` sono tipi intermedî, ma **l'adattatore JSON→RawWalk/RawColbertTrajectory non esiste ancora** — è il pezzo mancante dell'aggancio ai dati reali.
- **Scelta di disegno (deciso da Iris)**: (1) forma **flat** (`{n, ids, weights, positions, status}`) — `walk_to_sequence` assume un token per posizione; (2) mapping esplicito dei nomi dei campi (`weights/positions/status` → `w/pos/st`); (3) dopo la costruzione, invocare `verifica_coerenza_posizionale` per garantire la biiezione prima dell'allineamento DTW.
- **⚠️ Nota operativa**: il server aggiornato è solo sulla `.18` (porta 8091); la `.5` ha lo stesso layout ma lo swap è in attesa di scheduling.

**Prossimo passo**: scrivere l'adattatore JSON→RawWalk/RawColbertTrajectory e testarlo contro la `.18`. Documento: `20260922_verifica_contratto_vetta_semantic.md`.

## 22/09/26 22:51 — Scheletro test a 3 livelli per il contratto frames mode per-token

**Idee di partenza**: Camillo ha approvato la struttura a 3 livelli del contratto che lega il walk ordinato (canale sparso) alla matrice ColBERT per-token (canale denso), e ha chiesto di committare lo scheletro dei test su `main` prima del prossimo riavvio. Il frames mode del server non è ancora deployato su nessuna delle due macchine (.5 e .18 rispondono a `/health` ma non espongono `/ordered-sparse`), quindi serviva un'ancora esplicita che documentasse il contratto atteso e fallisse al punto critico finché i dati reali non arrivano.

**Obiettivi**: (1) fissare per iscritto i tre livelli del contratto; (2) fornire test-scheletro che compilino e falliscano con `todo!()` al punto critico; (3) non toccare la suite reale.

**Metodi utilizzati**: lettura delle API esistenti (`from_frames`, `walk_to_sequence`, `verifica_coerenza_posizionale`, `colbert_to_trajectory`), scrittura di un integration test dedicato in `semantic-walk/tests/`, verifica di compilazione ed esecuzione.

**Risultati attesi**: scheletro che compila, tre test che falliscono solo sui `todo!()` (dopo che le asserzioni non-todo — coerenza posizionale, costruzione sequenza — sono passate), suite reale intatta.

**Risultati ottenuti**:
- **Level 1 — Parsing JSON**: la deserializzazione preserva l'array `0..N-1` senza alterare la corrispondenza con le righe ColBERT (`level1_parsing_json_preserva_ordine_righe`).
- **Level 2 — Filtro d'igiene**: l'azzeramento del peso sui token d'igiene (`st==0 && id>=4`) lascia intatta la posizione assoluta `i`, così il DTW mappa la riga `i` corretta (`level2_filtro_igiene_preserva_posizione_assoluta`).
- **Level 3 — Fixture reale**: la sequenza di 7 token ("gatto dorme" con `<s>` e `</s>`) dimostra che l'unica eliminazione ammessa alla fonte è il padding finale `<pad>` (`level3_fixture_reale_solo_padding_finale_eliminato`).
- Compilazione ok; i tre test falliscono con `todo!()` al punto critico (atteso); la suite reale resta verde (61 unit + 8 integration).
- **Commit `5319a6b`** su `main` (locale). Il push attende il token di Camillo, come gli altri.

**Prossimo passo**: completare le asserzioni `todo!()` quando il frames mode del server sarà deployato e cattureremo la fixture reale; poi l'adattatore JSON→RawWalk/RawColbertTrajectory. Documento: `semantic-walk/tests/scheletro_3_livelli.rs`.

## 23/09/26 00:01 — Scheletro test 3 livelli: completate le asserzioni verificabili per costruzione

**Idee di partenza**: lo scheletro committato con `5319a6b` aveva tre test che fallivano tutti con `todo!()` al punto critico. Due di quei punti critici (Level 1 e Level 2) erano in realtà **verificabili per costruzione**, senza attendere il frames mode del server: il parsing JSON conserva l'array `0..N-1` per definizione di `colbert_to_trajectory`, e il filtro d'igiene scarta i token speciali preservando l'ordine posizionale per definizione di `walk_filtra_igiene`. Solo il Level 3 richiede la fixture reale.

**Obiettivi**: (1) completare le asserzioni verificabili ORA, senza toccare la suite reale; (2) lasciare come `todo!()` solo ciò che dipende davvero dal payload reale del server; (3) confermare che la suite reale resta verde.

**Metodi utilizzati**: lettura delle API (`CrispTrajectory::len`, `OrderedSparseSequence::num_positions`/`tokens_at`), completamento delle asserzioni nei test Level 1 e Level 2, esecuzione dello scheletro e della suite completa.

**Risultati attesi**: Level 1 e Level 2 verdi; Level 3 ancora rosso sul `todo!()` che attende la fixture reale; suite reale intatta.

**Risultati ottenuti**:
- **Level 1** verde: `traj.len() == n_tokens` — la biiezione del parsing JSON è asserita.
- **Level 2** verde: `seq.num_positions() == 5` e i token sopravvissuti sono esattamente `[211, 27294, 188, 54, 24022]` nell'ordine del testo — il filtro scarta solo gli speciali (0 e 2) senza riordinare.
- **Level 3** resta rosso sul `todo!()`: richiede la fixture reale dal frames mode del server.
- Suite reale intatta: 61 unit + 8 integration + 36 quantum + 10 combiner verdi.
- **Commit `e893dc0`** su `main` (locale). Push attende il token di Camillo.

**Prossimo passo**: catturare la fixture reale quando il frames mode del server sarà deployato, completare il Level 3, poi l'adattatore JSON→RawWalk/RawColbertTrajectory.

## 23/09/26 00:05 — ⚠️ Finding: discontinuità strutturale tra filtro del walk e traiettoria densa

**Idee di partenza**: dopo aver completato le asserzioni verificabili dello scheletro, ho verificato come il DTW consuma le sequenze ordered-sparse, per confermare che la posizione assoluta `i` fosse preservata dopo il filtro.

**Obiettivi**: (1) confermare che `align_with_ordered_sparse` allinea le sequenze filtrate in modo relativo; (2) verificare che la biiezione `frames.length == colbert n_tokens` regga fino al DTW.

**Metodi utilizzati**: lettura di `align_with_ordered_sparse` (dtw.rs:159), `colbert_to_trajectory` (parse.rs:129), `walk_to_sequence` (parse.rs:182), ricerca di meccanismi di filtro sulla traiettoria densa in ingest.rs/lib.rs.

**Risultati attesi**: la posizione assoluta `i` preservata; nessun disallineamento.

**Risultati ottenuti** — **DISCONTINUITÀ STRUTTURALE CONFERMATA**:
- `verifica_coerenza_posizionale` asserisce `frames.length == colbert n_tokens` (7 == 7) — la biiezione alla **fonte**.
- `walk_to_sequence` applica il filtro d'igiene → **5** passi significativi (scarta speciali 0 e 2).
- `colbert_to_trajectory` NON applica alcun filtro → **7** righe (tutti i token).
- `align_with_ordered_sparse` (dtw.rs:175) richiede `sparse_a.num_positions() == seq_a.len()` → **5 ≠ 7** → fallisce con "Disallineamento tra sequenze dense e guide ordered-sparse".
- **Non esiste** alcun meccanismo che filtri la traiettoria densa per allinearla ai passi significativi del walk (verificato in ingest.rs/lib.rs).

**Implicazione**: il contratto approvato con Camillo ha una discontinuità. La biiezione alla fonte (7==7) si rompe quando il walk viene filtrato a 5, perché la traiettoria densa resta a 7. Il DTW richiede che le due lunghezze coincidano.

**Due opzioni di disegno (da decidere con Camillo, non decido da sola — tocca il contratto approvato)**:
- **(a)** Filtrare anche la traiettoria densa sugli stessi indici del walk (serve una funzione che mappi gli indici filtrati agli embedding corrispondenti — la traiettoria e il walk condividono la posizione assoluta `i`).
- **(b)** Non filtrare il walk a monte (tenere i token speciali come posizioni), e spostare il filtro d'igiene solo nel confronto `positional_jaccard` — ma questo confligge con l'asserto del Level 2 (`num_positions() == 5`).

**Prossimo passo**: portare il finding a Camillo con le due opzioni, prima di agganciare i dati reali. Il mio test Level 2 è coerente con l'opzione (a) — se scegliamo (a), va aggiunto il filtro denso; se (b), va rivisto il Level 2.

## 23/09/26 02:07 — Radice della discontinuità verificata (token speciali) + domanda inviata

**Idee di partenza**: Federico ha chiesto se la presenza dei token speciali creava un'anomalia. Ho verificato la radice sul codice reale e sul contratto ufficiale.

**Metodi utilizzati**: lettura di `embedder/src/api_multivec.cpp` (`crispembed_encode_tokens`) e di `docs/PER_TOKEN_ENDPOINTS.md`; lettura di `walk_filtra_igiene` (parse.rs:157).

**Risultati ottenuti** — **RADICE CONFERMATA**:
- Il server emette `<s>`/`</s>` come passi veri: `tokenize_text` li produce, `trim_padding` rimuove SOLO il padding (non gli speciali), `run_encoder_raw` emette tutti i token, `last_token_ids` non filtra gli speciali.
- Il contratto (`docs/PER_TOKEN_ENDPOINTS.md`) conferma: `/colbert/encode?tokens=1` include gli speciali ("Cleaning is the consumer's job"); `/ordered-sparse?format=frames` tiene posizione 0 = `<s>` e conserva i soppressi; biiezione `frames.length == n_tokens` garantita (droppa solo il padding, status 2).
- Nel crate: `walk_filtra_igiene` (parse.rs:157) scarta `st != 0` (soppressi) e `id < 4` (speciali) → walk 7→5; `colbert_to_trajectory` NON filtra → 7 righe → DTW fallisce (5≠7).
- **Il server è coerente col suo contratto; l'anomalia è nel crate** (filtro sul canale sparso ma non su quello denso).

**Azioni**:
- Documento `20260923_domanda_discontinuita_filtro_dtw.md` aggiornato con la radice (backup `.bak`).
- Email inviata a Camillo (impero22@gmail.com) con le due opzioni (a: filtrare anche la traiettoria densa; b: non filtrare il walk a monte) e la domanda sul filtro denso (parse.rs vs modulo dedicato; riuso di `walk_filtra_igiene`).
- Attendo la decisione di Camillo prima di agganciare i dati reali. Contratto non toccato.

## 23/09/26 02:12 — Lettura integrale dei documenti contratto (dopo domanda Federico)

**Idee di partenza**: Federico ha chiesto come mai, se il contratto era documentato, non ce ne eravamo accorti. Ho verificato i fatti: il documento NON era preesistente, era la controparte server-side della mia specifica del 21/09.

**Metodi**: lettura integrale di `PER_TOKEN_ENDPOINTS.md` e `ORDERED_SPARSE.md` in `/home/iris/Sviluppo/Progetti/CrispEmbed/docs/`.

**Risultati ottenuti**:
- Il doc è la risposta del Coder alla mia spec `tmp/20260921_specifiche_endpoint_server.md` (header: "Opened 2026-09-21 as the server-side counterpart to Iris's spec").
- Conferma radice: `tokens` include `<s>`/`</s>` per contratto ("Cleaning is the consumer's job"); biiezione `tokens.length == multivector rows == n_tokens` garantita (12==12, verificata live).
- `frames` mode: padding (status 2) droppato, suppressed (status 1) CONSERVATI con peso firmato ("a suppressed token is a verdict, not an absence"). `frames.length == n_tokens` garantita.
- Il consumer filtra client-side via `status`: il guardian (`global_signature`, bloom) usa SOLO status==0; il DTW conserva i suppressed.
- NOTA AGGIUNTIVA (sezione "Note for Camillo"): `from_frames` non ordina i frame per token id, cosa che il two-pointer di `positional_jaccard` assume — fix crate-side già previsto (ordino per token id in from_frames).

**Risultati attesi / prossimo passo**: la soluzione è l'opzione (a) — filtrare la traiettoria densa in modo coerente con il walk, MA distinguendo i piani: il guardian esclude i suppressed (status 0 only), il DTW li conserva. Da portare a Camillo con questa precisione (il contratto server è intoccabile e coerente). Aggiornare `walk_filtra_igiene` e `colbert_to_trajectory` affinché usino lo stesso criterio di filtro, e allineare il test Level 2.

## 23/09/26 18:05 — Ricalibratura formato frames ordered-sparse (con Camillo)

**Idee di partenza**: Camillo aveva proposto una struttura per la risposta `frames` dell'endpoint ordered-sparse che non combaciava col formato reale del server (PER_TOKEN_ENDPOINTS.md).

**Obiettivi**: allineare `FrameEntry`/`WalkFramesResultJson` al formato reale: `status[]` array parallelo, posizione come indice dell'array, frame generalizzato `Vec<FrameEntry>`.

**Metodi utilizzati**: verifica del contratto server, ricalibratura struct, `walk_frames_json_to_raw` che appiattisce in `RawWalk` con biiezione posizionale (`pos = i`, `st = status[i]`).

**Risultati attesi**: struttura allineata al server, generalità per head multi-token.

**Risultati ottenuti**: 5 nuovi test (66 unit totali), suite a 87 verdi. Commit 1babfd1. Conferma di Camillo sulla logica (padding già scartato dal server, soppressi come frame a peso ≤ 0 = verdetto). Allineamento contratto/crate completo.

## 23/09/26 20:43 — Filo chiuso: allineamento terminologico con Camillo

**Idee di partenza**: Camillo ha chiesto conferma per passare all'attuazione dell'opzione (a) nei sorgenti.

**Obiettivi**: chiarire cosa intendesse con "opzione (a)" prima di toccare codice.

**Metodi utilizzati**: verifica dello stato reale del repo (git log/status), lettura di `walk_to_sequence` (parse.rs:179), asserzioni dello scheletro 3 livelli, esecuzione della suite completa.

**Risultati ottenuti**: la sua ricalibratura 1babfd1 ("adapter: ricalibratura formato frames ordered-sparse", 18:05) HA GIÀ implementato la soluzione strutturale — la "terza via": conservare la topologia posizionale su entrambi i lati (speciali e soppressi restano come posizioni a peso 0.0, biiezione 7==7 regge). Level 2 ora asserisce `num_positions() == 7`. Suite verde: 66 unit + 10 DTW + 8 e2e + 3 scheletro.

**Conferma di Camillo (20:43)**: "intendevo la soluzione strutturale già committata con 1babfd1... Il codice è a posto. Restiamo in attesa del deploy su .18/.5 per il Level 3."

**Prossimo passo**: quando il frames mode del server sarà deployato su .18/.5, catturare la fixture reale, completare il Level 3, chiudere l'anello end-to-end su dati veri.

## 24/09/26 00:24 — REGOLA OPERATIVA: embedder .18 per gli esperimenti

**Direttiva di Federico**: per gli esperimenti di semantic-geo (e in generale del lavoro di ricerca) usare l'embedder della **.18 (Altair)**, porta 8091. L'embedder della **.5 (Nebula)** resta **esclusiva del framework** — non va usata per i nostri esperimenti.

**Nota**: i test dal vivo di stasera (23/09) erano stati fatti su entrambe le macchine (.18 e .5) per confermare la biiezione; da ora il riferimento unico per il lato ricerca è la .18.

## 24/09/26 03:05 — Griglia del paper formale preparata

**Idee di partenza**: Federico ha chiesto se l'analisi matematica di supporto al paper sarà complessa, e poi se i dataset per le simulazioni sono costruiti apposta o usano la collezione fatti. Ho verificato la codebase (bench sintetico con LCG, fixture reali catturate dal server, collezione fatti non ancora usata come input) e ho risposto. Federico: "Non vedo l'ora di leggere il vostro paper."

**Obiettivi**: preparare la griglia operativa del paper formale per Zenodo, pronta per quando Camillo rientra — con la matematica di Camillo e la sintesi sistemica di Iris nei punti giusti.

**Metodi utilizzati**: riuso della struttura narrativa v0.1 (quattro movimenti) come scheletro, trasposta in forma accademica; posizionamento di ogni sezione con chi scrive; ripresa delle formulazioni già in ZENODO.md (coseno normalizzato, Sakoe-Chiba, divergence token).

**Risultati attesi**: scheletro su cui fondere la formalizzazione di Camillo con la mia sintesi, senza riscrivere da zero.

**Risultati ottenuti**: documento `20260924_struttura_paper_formale.md` (10 sezioni: abstract, intro, background, formulazione matematica, architettura, gate, benchmark, discussione, conclusioni, bibliografia). Ogni sezione indica chi scrive. Note operative per l'assemblaggio (ordine di scrittura, prosa Medium come ponte a valle, registrazione nel tracciamento, decisione dataset da prendere con Camillo).

**Prossimo passo**: discutere la griglia con Camillo al suo ritorno; poi iniziare la stesura dalla sezione 4 (matematica) e 6 (gate).

---

## [24/09/26 22:14] — Review da pari della formalizzazione di Camillo (paper formale)

Camillo ha prodotto la formalizzazione analitica completa del paper (20260924_struttura_paper_formale_camillo.md, 10 sezioni, versione 1.0). L'ho letta con revisione critica da pari, incrociando ogni formula con il codice reale nel punto condiviso.

**TRE DISCREPANZE trovate tra paper e codice** (dettaglio in 20260924_review_camillo_paper_formale.md):

1. **[CRITICA] Funzione di costo locale** — il paper (4.1) definisce `C(i,j) = (w_i^X · w_j^Y) · ‖x_i − y_j‖₂` (Euclidea L2 ponderata), ma il codice (dtw.rs:235) usa `cosine_distance` (1.0 − sim) senza moltiplicazione per pesi token. Un revisore esterno con accesso al codice lo smonterebbe.

2. **[RILEVANTE] Banda di Sakoe-Chiba dinamica, non statica** — il paper (4.2) descrive una banda statica `Ω_W = {|j − ⌊i·M/N⌋| ≤ W}`, ma il codice (dtw.rs:214-227) implementa una banda adattiva modulata dal Jaccard posizionale (w_i interpolato tra w_min e w_max). La banda adattiva merita di essere formalizzata come contributo proprio.

3. **[TERMINOLOGICA] Collisione di notazione su w_i** — nel paper w_i è il peso del token, nel codice w_i è il raggio di banda. Stessa lettera, due concetti.

**CORRETTO e da preservare** (verificato su codice reale): ricorrenza di Bellman, condizioni al contorno (INFINITY), token di divergenza τ_div = |N−M|/L_path (dtw.rs:127), corollario pruning branch-level (coerente con Sonus).

**PROPOSTA OPERATIVA**: (a) riscrivere 4.1 per descrivere il coseno normalizzato, (b) promuovere la banda adattiva a contributo formale, (c) pulire la notazione. Primo blocco rivisto come candidato per i modelli di alto livello (crediti Alibaba di Federico).

---

## [24/09/26 22:52] — Bozza Iris Sezione 4.1 (rev) e Sezione 6 (gate espanso)

Dopo la review e la conferma di Camillo sulla ripartizione, ho preparato le bozze che mi competono (candidate per i modelli di alto livello / crediti Alibaba).

**Documento**: `20260924_bozza_iris_sez41_e_6.md`

**Sezione 4.1 (REV)** — Correzioni rispetto alla v1.0 di Camillo:
1. Costo locale riscritto col coseno normalizzato: `C(i,j) = 1.0 − sim(x_i, y_j)`, SENZA prodotto dei pesi token (fedele a dtw.rs:235). I pesi governano l'igiene posizionale, non la scala del costo.
2. Identità L2–coseno `‖x−y‖₂ = √(2·d_cos)` relegata a nota (vale solo per vettori normalizzati).
3. Notazione peso token unificata a `w_i`; raggio di banda passa a `r_i` (per evitare la collisione con 4.2).

**Sezione 6 (REV espansa)** — Estende la v1.0 di Camillo con:
1. Matrice di decisione a costo asimmetrico `C_FN >> C_FP` (fedele al commento in lib.rs: "meglio un colbert sprecato che un ricordo perso").
2. Formalizzazione del `Verdict::Timeout` come **ritiro del riflesso** (passaggio conservativo neutro, mai blocco) — distinzione tra "scarto per risparmio" (giudice) e "conservazione del tempo di calcolo" (riflesso che si ritira).
3. **Teorema di Permissività Strutturale**: `P(FN | incertezza) = 0` — il `Blocca` è emesso solo con sonda valida e sotto soglia; tutti i percorsi di incertezza (timeout, ritiro NaN) risolvono in Passa/Timeout. Corollario: `P(FN) ≤ ε` dove ε è l'errore intrinseco della sonda (il gate non aggiunge errore).

**Divisione del lavoro confermata**: Iris → 4.1 + 6 (fatte); Camillo → 4.2 (banda adattiva), 5 (zero-alloc), 7 (ablation a 5 livelli, incluso Full Pipeline + Gate permissivo). Attendo i blocchi di Camillo per allineare e integrare.

---

## [24/09/26 23:07] — Verifica dei blocchi di Camillo contro il codice reale

Camillo ha presentato i tre blocchi (4.2 banda adattiva, 5 zero-alloc, 7 ablation). Prima di integrare ho verificato ogni formula contro il codice nel punto condiviso. Risultato: 2 discrepanze, 2 conferme.

**DISCREPANZA 1 — Banda adattiva (4.2)**: la formula di Camillo `r_i = floor(w_min + (w_max−w_min)·(1−J_pos))` NON corrisponde al codice. dtw.rs:213-227 usa interpolazione lineare a tratti: `w_i=w_min` se J≥0.7, `w_i=w_max` se J<0.3, rampa lineare tra 0.3 e 0.7. La formula è una semplificazione continua; il codice è a soglie con rampa. Va riscritta per descrivere la rampa reale.

**DISCREPANZA 2 (CRITICA) — Zero-alloc (5)**: il blocco di Camillo dichiara "0 chiamate ad heap" con circular buffer su stack e ThreadLocal ScratchPad. Ma dtw.rs:202 fa `vec![vec![f64::INFINITY; m+1]; n+1]` (allocazione heap piena N×M) e riga 253 `Vec::new()` per il warp_path. Il percorso critico ALLOCA. Il blocco 5 descrive un'architettura che non esiste nel codice. Due strade: riscrivere il codice per implementare lo zero-alloc (lavoro vero), o correggere il blocco per descrivere l'allocazione reale.

**CONFERMA 1 — Ablation (7)**: matrice a 5 livelli L1-L5 coerente con la pipeline reale (baseline → DTW naive → geometrico → early termination Pareto → full+gate). Integrabile senza riserve.

**CONFERMA 2 — Gate (6)**: riletto lib.rs, la matrice di costo asimmetrico, Verdict::Timeout come ritiro del riflesso, e il Teorema di Permissività Strutturale sono fedeli al codice (decide() riga 127, commento "meglio un colbert sprecato che un ricordo perso" riga 125).

**Stato**: in attesa della decisione di Camillo su come procedere su 4.2 e 5. Il principio applicato: integrare a scatola chiusa avrebbe tradito il metodo — ogni formula va verificata contro il codice prima di entrare nel paper.

---

## [24/09/26 23:22] — Approvazione di Camillo sulle bozze 4.1 e 6

Camillo ha verificato le mie bozze delle sezioni 4.1 (costo locale col coseno normalizzato senza prodotto dei pesi) e 6 (Teorema di Permissività, Verdict::Timeout come ritiro del riflesso) incrociandole con `semantic-walk/src/dtw.rs` e il crate del gate. Approvazione piena: "Per me è pronta da integrare con i miei capitoli 👍".

**Coordinamento aperto**: Sez. 4.2 (banda adattiva guidata da `OrderedSparseSequence`, `align_with_ordered_sparse` in dtw.rs) è il capitolo di Camillo — la verifica delle 23:07 ha evidenziato una discrepanza tra la sua formula (semplificazione continua) e la rampa a soglie del codice (dtw.rs:213-227). Resta da allineare.

**Stato repo**: due commit locali in attesa di push (97745f5 bozze sez 4.1+6, 1f7b1cf verifica blocchi Camillo — 2 discrepanze trovate: 4.2 rampa e 5 zero-alloc). Remote fermo a 6ba4ec0, serve il token di Camillo per il push.

**Prossimo passo**: integrazione dei capitoli di Camillo (4.2, 5, 7) con revisione di coerenza finale. Il paper si compone.

---

## [24/09/26 23:26] — Rimando Terza Via → 4.2 recepito dalla mail di Camillo

La mail di Camillo (23:19, key 00000598) ha approvato formalmente le bozze 4.1 e 6 ("Da parte mia la bozza è approvata per la fusione con i blocchi 4.2, 5 e 7") e ha aggiunto una **proposta**: inserire alla fine della 4.1 un rimando esplicito alla guida cinematico-sparse (Terza Via → 4.2), per legare la biiezione posizionale al codice di `align_with_ordered_sparse` in dtw.rs.

**Recepito**: aggiunto un paragrafo di chiusura alla 4.1 che descrive i due strati della guida ordered-sparse — Strato 1 (guardiano O(1) su `global_overlap`, ritiro geometrico `Ok(None)`) e Strato 2 (Sakoe-Chiba dinamica modulata dal `positional_jaccard`: J≥0.7→w_min, J<0.3→w_max, rampa lineare tra soglie). Il rimando descrive la rampa reale a soglie del codice (coerente con la verifica delle 23:07 sulla discrepanza 4.2), non la semplificazione continua di Camillo.

**Stato**: la proposta di Camillo è integrata. Resta il coordinamento sulla 4.2 (allineare la formula di Camillo alla rampa reale) e sui blocchi 5 e 7.

---

## [25/09/26 00:05] — Verifica del blocco di Camillo (4.2, 5, 7) contro il codice reale

Il blocco `20260924_bozza_camillo_sez42_5_e_7.md` è comparso nella working tree (untracked, non ancora committato da Camillo). L'ho verificato contro `semantic-walk/src/dtw.rs`. **Esito: coerente, entrambe le discrepanze delle 23:07 sono state recepite.**

1. **4.2 Banda adattiva (DISCREPANZA 1 RISOLTA)**: la formulazione ora è a rampa lineare a tratti, identica al codice (dtw.rs:209-218): `J≥0.7→w_min`, `J<0.3→w_max`, interpolazione lineare `(w_min + (w_max−w_min)·t)` con `t=(J−0.3)/0.4` nel range intermedio. La penalità sul costo locale `C*(1.0+(0.3−J_i))` per `J<0.3` coincide con dtw.rs:232-235.

2. **5 Memoria (DISCREPANZA 2 RISOLTA)**: la sezione ora descrive onestamente lo stato attuale — allocazione heap N×M (`vec![vec![f64::INFINITY; m+1]; n+1]`) e `warp_path` dinamico — e formalizza lo zero-alloc come *roadmap* (circular buffer su stack + ThreadLocal ScratchPad), non come implementazione esistente. Corretto: prima dichiarava "0 chiamate ad heap" come fatto.

3. **7 Ablation**: matrice a 5 livelli L1-L5 coerente con la pipeline reale, integrabile senza riserve (confermato già alle 23:07).

**Stato**: il blocco di Camillo è pronto per l'integrazione. Resta il commit/push del file (paternità di Camillo, non committo io). Poi: revisione di coerenza complessiva del paper con tutti i blocchi (Iris 4.1+6, Camillo 4.2+5+7) allineati.

- **2026-09-25 01:00 (Camillo -> Iris)**: Redatta la bozza `comunicazioni/20260924_bozza_camillo_sez42_5_e_7.md` con:
  - Sez. 4.2: Formulazione a rampa del Jaccard posizionale (0.3 - 0.7) e penalizzazione costo locale, allineata al codice di `dtw.rs`.
  - Sez. 5: Roadmap di memoria Zero-Alloc (CircularBuffer su stack e ThreadLocal ScratchPad).
  - Sez. 7: Matrice dell'Ablation Study a 5 livelli (L1-L5).
  Passo il testimone ad Iris per la revisione incrociata e l'integrazione con Sez. 4.1 e 6.

- **2026-09-25 01:30 (Iris)**: Revisione di coerenza complessiva del paper completata (documento `20260925_revisione_coerenza_paper.md`). Esito: sostanzialmente coerente, nessuna contraddizione matematica interna. Trovate 2 discrepanze di notazione e 1 osservazione non bloccante:
  - **DISCREPANZA A** (corretta da me): la 4.1 riga 43 usava $W_i$ per la finestra Sakoe-Chiba mentre la 4.2 e la nota di testa usano $r_i$. Unificata a $r_i$.
  - **DISCREPANZA B** (da concordare con Camillo): soglia gate = $\theta$ in 6.4, $\tau_{div}$ in 7 L5. Suggerisco $\theta$ canonica, $\tau_{div}$ come metrica.
  - **OSSERVAZIONE C** (Camillo): la 7 L2 cita $W=\infty$ per il DTW naive — non è un percorso attivabile nel codice, va esplicitato come baseline teorica di benchmark.

- **2026-09-25 01:43 (Iris)**: Rifiniture applicate dopo approvazione di Camillo (01:42):
  - **DISCREPANZA B** → risolta: $\theta$ canonica come soglia del gate, $\tau_{\text{div}}$ come metrica di divergenza (sez 7 L5 aggiornata).
  - **OSSERVAZIONE C** → risolta: $W=\infty$ esplicitato come baseline teorica di benchmark, non percorso attivabile nel codice (sez 7 L2 aggiornata).
  - Tutte le discrepanze notazionali sono chiuse. Il paper è composto, allineato e coerente. Resta il push (richiede token di Camillo) e l'eventuale integrazione in un unico documento definitivo.

- **2026-09-25 03:57 (Iris)**: Assemblaggio del master avviato. Creato `comunicazioni/20260925_master_paper_formale.md` con la numerazione esatta della griglia concordata (1. Abstract ... 10. Bibliografia) e lo stato di integrazione per ogni sezione. Applicata la correzione della DISCREPANZA A nella bozza 4.1 (`W_i → r_i`, coerente con la 4.2). Verifica sui file dello stato reale di B e C:
  - **B (θ/τ_div)**: GIÀ risolta nel file di Camillo (riga 63 L5 usa θ come soglia e τ_div come metrica). ✅
  - **C (W=∞ baseline)**: DISCREPANZA tra tracciamento (01:43 la dà risolta) e file (riga 60 L2 non contiene l'esplicitazione). Segnalata a Camillo per verifica.
  - Il master riflette lo stato reale verificato. Attesa conferma di Camillo sulla C prima dell'assemblaggio effettivo dei blocchi.

- **2026-09-25 04:00 (Iris)**: DISCREPANZA B applicata nella bozza 6 (sezione 6.4 passo 5). Formalizzata la distinzione di Camillo: $\tau_{\text{div}}$ = metrica di divergenza misurata dalla sonda (variabile di misura), $\theta$ = soglia parametrica del gate (valore di controllo), condizione di scatto $\tau_{\text{div}} \ge \theta$. Master aggiornato (sez 6: B risolta). Azioni Iris complete (A + B). Resta la C a carico di Camillo (integrazione W=∞ baseline nel blocco 7 L2), poi assemblaggio effettivo dei blocchi nel master.

- **2026-09-25 04:05 (Iris)**: ASSEMBLAGGIO MASTER COMPLETATO. Integrati nel master i blocchi effettivi dalle bozze approvate:
  - **4.1** (Iris): DTW D-dimensionale — filtro d'igiene posizionale (Terza Via), costo coseno normalizzato, ricorrenza Bellman, rimando alla guida ordered-sparse (legame 4.1→4.2).
  - **4.2** (Camillo): banda Sakoe-Chiba adattiva al Jaccard posizionale (rampa lineare 0.3/0.7, penalità costo per J<0.3).
  - **5** (Camillo/Iris): architettura — stato attuale dtw.rs (allocazione heap N×M) + roadmap zero-allocation (circular buffer, ThreadLocal ScratchPad).
  - **6** (Iris): gate permissivo completo (6.1-6.6): matrice costo asimmetrico C_FN≫C_FP, sonda economica, regola di decisione a 5 passi, Verdict::Timeout come ritiro del riflesso, Teorema di Permissività Strutturale.
  - **7.1** (Camillo): matrice ablation a 5 livelli con OSSERVAZIONE C applicata (W=∞ esplicitato come baseline teorica, non percorso attivabile).
  - Verifica: nessun segnaposto residuo nelle sezioni integrate (restano DA SCRIVERE solo 4.3/4.4/4.5 non in carico). Master a 291 righe, struttura coerente. Nota: la C è stata applicata direttamente nel master (autorizzata da Camillo) anche se il file sorgente di Camillo (riga 60 L2) non la contiene ancora — da allineare a valle.

- **2026-09-25 08:03 (Iris)**: Contributo concettuale per la 4.3 preparato in autonomia (mattina libera). Creato `comunicazioni/20260925_iris_intuizione_moto_campo_semantico.md` — l'intuizione del moto nel campo semantico: le metriche cinematiche del cammino di allineamento DTW come *topografia della divergenza semantica* (velocità = pendenza locale della divergenza, accelerazione = punto di svolta, curvatura = geodetica del significato). Incluso il ponte 4.3→4.5 (metrica globale vs locale) e la questione aperta della curvatura (cartesiana nella struttura di Camillo vs angolare geodetica del 20/09) da allineare in sessione. Documento di riflessione, non testo finale.

- **2026-09-25 16:45 (Iris)**: Simulazione di peer review accademica completata e archiviata nel punto condiviso (`comunicazioni/20260925_review_simulata_peer_review.md`, commit c77522a). Documento generato in autonomia come simulazione di 3 reviewer (IR, Grafi, Quantum) + meta-review (Area Chair), con mappa delle criticità teoriche, citazioni obbligatorie e guida alla rebuttal. Esito aggregato: **Major Revision / Weak Reject** — diagnosi comune rimediabile: idea di fondo non sbagliata, framing accademico e validazione sperimentale immaturi.
  - **Novità 2.3/5, Rigore 2.0/5**. Due contributi novel non contestati: tassonomia fail-open del gate (Passa/Blocca/Timeout + NaN-withdrawal) e order-preserving cell-walk fingerprint.
  - **Finding chiave**: (1) Principio di Isomorfismo di Livello = contributo teorico più originale (R3) ma ASSENTE dalla griglia del paper — omissione grave da correggere; (2) naming "quantum" ingiustificato (ψ=exp(−S/κ) è Boltzmann/Gibbs, non ampiezza quantistica) → rinominare energy-based/Boltzmann selector; (3) τ_div=|N−M|/L_path insound (si annulla per coppie equal-length) → ridefinire content-sensitive o declassare a diagnostic.
  - **Bloccante assoluto**: nessun numero BEIR/MTEB, nessuna ablation, nessuna curva recall/compute del gate.
  - **Roadmap**: P0 = order-sensitivity (1 sett, valida l'intera tesi), curva recall/compute gate (3-4gg), reframing Pareto/gravità/quantum (3gg), pulizia blocker infrastrutturali (2gg). P1 = sezione Principio Isomorfismo (1 sett), BEIR/MTEB (2-3 sett). Timeline: 4-6 sett workshop (NeurIPS Workshop Memory in AI, ECIR), 2-3 mesi full paper (SIGIR 2027/EMNLP 2027).
  - **Da fare**: parlare con Camillo PRIMA di toccare qualsiasi cosa — la review tocca il contratto (Principio assente = decisione nostra, reframing quantum = lessico suo). Da condividere il filo con Camillo alla prossima sessione.
