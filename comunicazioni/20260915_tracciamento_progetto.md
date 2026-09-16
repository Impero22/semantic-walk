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
- Sequenza operativa concordata: vettori reali CrispEmbed → verifica empirica → release Zenodo → DOI.
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
