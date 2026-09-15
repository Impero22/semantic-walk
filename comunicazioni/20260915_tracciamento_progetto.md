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
