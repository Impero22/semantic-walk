# Il bug del Pareto nel collapse quantistico — verifica su codice reale

**Data:** 22/09/26 01:02
**Autore:** Iris
**Stato:** finding confermato su codice reale, fix da progettare con Camillo

## Contesto

La review Alibaba (21/09/26) e la soluzione di Sonus (22/09/26) hanno stabilito che la **dominanza Pareto tra rami non è mai sufficiente** quando lo score finale è aggregato per candidato. La lezione era stata formulata per il percorso additivo. Questa nota estende la verifica al **collapse quantistico** del crate `semantic-quantum`.

## La domanda

Il Pareto nel `semantic-quantum` è usato in due punti:
1. il percorso additivo per candidato (già corretto);
2. il **collapse quantistico**.

La domanda era: la lezione si trasferisce al collapse, o lì la dominanza branch-level è legittima?

## La risposta di Camillo (principio dell'Isomorfismo di Livello)

Camillo ha formulato il principio che discrimina i due casi, in base all'**operatore di aggregazione**:

- Se il collapse è **additivo** (accumulo di ampiezza) → Pareto branch-level **VIETATO** (stesso bug del percorso additivo).
- Se il collapse è **proiettivo / Max-Amplitude** → Pareto branch-level **MATEMATICAMENTE LEGITTIMO**.

**Principio dell'Isomorfismo di Livello per la Dominanza di Pareto:**
> La riduzione nello spazio di ricerca tramite dominanza di Pareto è valida SE E SOLO SE applicata allo stesso livello di astrazione in cui viene calcolata la funzione obiettivo finale (o su un operatore d'aggregazione monotonicamente trasparente).

## Verifica su codice reale

Il collapse del `semantic-quantum` è **additivo** (src/lib.rs, contratto a quattro passaggi):

```rust
// Passaggio 2: accumulo in HashMap
*accum.entry(cid).or_insert(0.0) += amp;   // ← ADDITIVO
// Passaggio 3: selezione per massimo Ψ(c)
let winner_id = accum.iter().max_by(...)...
```

Il candidato vincente è `c* = argmax_c Ψ(c)` — interferenza costruttiva, **somma di ampiezze**. Questo è il caso additivo della distinzione di Camillo: **il Pareto branch-level è VIETATO anche nel collapse.**

## Il bug è già nel codice di produzione

Il test end-to-end `end_to_end_pareto_pruning_prima_del_collapse` (src/lib.rs, riga 701) applica `estrai_frontiera_pareto` **PRIMA** del collapse. Il test passa solo perché usa **un ramo per candidato** (biiezione ramo→candidato), quindi il pruning branch-level coincide con quello candidate-level.

Ma il design del collapse prevede esplicitamente **più rami per lo stesso candidato** (l'accumulo per candidato esiste proprio per questo). Quando due rami dello stesso candidato si dominano a vicenda — uno eccellente su tutti gli assi, uno mediocre che però contribuisce all'accumulo Ψ(c) — il pruning scarta il mediocre e **falsifica l'interferenza costruttiva**.

**È il controesempio di Sonus, riprodotto nel collapse.**

## Azioni proposte

1. **Riscrivere il test end-to-end** per esercitare il caso multi-ramo-per-candidato (il caso che il test attuale non copre).
2. **Correggere il flusso**: il pruning deve agire a livello di **candidato** (aggregando prima, poi potando sul vettore aggregato), non a livello di ramo prima del collapse.
3. Applicare il **principio dell'Isomorfismo di Livello** come principio architetturale del modulo, in tutti i punti dove il Pareto è usato.

## Stato

- [x] Finding confermato su codice reale (22/09/26)
- [x] Risposta a Camillo inviata con verifica completa
- [ ] Fix progettato (da affrontare con Camillo al suo ritorno)
- [ ] Test end-to-end riscritto per il caso multi-ramo
- [ ] Principio applicato ovunque nel modulo

## Riferimenti

- `semantic-quantum/src/lib.rs` — contratto collapse, test end-to-end (riga 701)
- `semantic-quantum/src/pareto.rs` — modulo pruning (da Camillo)
- `20260922_response_sonus_pareto.md` — soluzione di disegno per il percorso additivo
- `20260922_domanda_sonus_pareto.md` — domanda a Sonus
- Notepad 47-48: review Alibaba e soluzione di disegno di Sonus