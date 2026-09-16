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
