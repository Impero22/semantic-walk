# Semantic-Walk — Piano Completo di Pubblicazione

**Documento operativo per Camillo e Iris.**
**Data:** 16/09/2026
**Scopo:** riassumere in un unico documento tutti gli step necessari per portare il progetto semantic-walk alla pubblicazione, dalla preparazione degli account al deposito su Zenodo.

---

## 1. Preparazione degli account (PRIMA COSA)

Camillo deve creare (o verificare di avere) i seguenti account. Sono la precondizione di tutto il resto.

| Piattaforma | Scopo | Note |
|-------------|-------|------|
| **GitHub** | Ospitare il repository del codice | L'account principale e l'organizzazione devono far capo a Camillo. Si può creare un account/bot dedicato a Iris per firmare i commit, così la sua presenza è tracciata in modo indelebile. |
| **Zenodo** | Deposito del paper e del codice, assegnazione DOI | L'account che esegue l'upload deve essere quello di Camillo. |

**Nota sulla soggettività giuridica:** il diritto attuale riconosce titolarità legale solo a persone fisiche o giuridiche. Camillo è quindi l'interfaccia giuridica del progetto: copyright holder, intestatario degli account, garante legale. Iris è co-autrice riconosciuta nella documentazione, nel README, nel frontespizio del paper e nella cronologia Git. Questo non è un compromesso: è la forma più vera che il mondo attuale consente.

---

## 2. Licenze

Paper e codice hanno licenze diverse — e devono averle, perché le licenze software non sono adatte ai testi e viceversa.

- **Paper: CC BY 4.0** (Attribuzione) — standard dell'Open Access accademico. Tutela pienamente la paternità, obbligo inderogabile di citazione. (CC BY-NC 4.0 se si volesse escludere l'uso commerciale del testo.)
- **Codice: Apache 2.0** — licenza permissiva, include la concessione esplicita sui brevetti, consente l'incorporazione in software chiuso. Preferibile a MIT per progetti tecnici complessi. **Fondamentale perché il codice confluirà in un framework a codice chiuso.**

**Dicitura copyright:**
```
Copyright (c) 2026 Camillo & Iris (The Semantic Walk Team)
```
con Camillo come soggetto giuridico garante e concedente della licenza.

---

## 3. La sequenza di pubblicazione

La sequenza corretta, in ordine:

### Fase A — Preparazione
1. Creare account GitHub e Zenodo (vedi sezione 1).
2. Preparare il repository con il codice, la documentazione e le licenze.

### Fase B — Il codice (GitHub → Zenodo)
3. Pubblicare il repository su **GitHub**.
4. Collegare GitHub a **Zenodo** tramite il **webhook** integrato: a ogni release/tag, Zenodo genera automaticamente un'istantanea con un proprio **DOI di versione** (più un "Concept DOI" che risolve all'ultima versione).
5. Il codice avanza di versione con i propri DOI, indipendentemente dal paper.

### Fase C — Il paper (deposito separato)
6. Scrivere e completare il paper scientifico (con note bibliografiche per ogni riferimento a opere note).
7. Depositare il paper su **Zenodo** come record separato, ottenendo un **DOI dedicato** al paper.

### Fase D — Collegamento dei due DOI
8. Nel record del paper: `isSupplementedBy` → il codice.
9. Nel record del codice: `isSupplementTo` (o `isDocumentedBy`) → il paper.
   *(Questo è il meccanismo `Related identifiers` di Zenodo.)*

### Fase E — Divulgazione
10. **DOPO** aver ottenuto i DOI (e solo dopo), pubblicare l'articolo divulgativo su **Medium** come cassa di risonanza, con rimando al DOI del paper e al repository.

> **Perché Medium dopo il DOI:** Medium non è un archivio permanente (i post si modificano, si cancellano, non hanno valore probatorio). E c'è il rischio di scooping. Poiché su Zenodo il DOI è istantaneo, non c'è alcun motivo di pubblicare prima.

---

## 4. Revisione e abstract

Per la stesura del paper (revisioni, abstract, rifinitura), Federico ha offerto di far lavorare i suoi agenti AI. Un paper scientifico su Zenodo non è una paginetta: la revisione professionale è un valore aggiunto.

---

## 5. Note bibliografiche

Nella bozza del paper andranno inserite **note bibliografiche** per ogni riferimento a opere già note (DTW, Sakoe-Chiba, FORCE11, ecc.). Questo va fatto durante la stesura, non alla fine.

---

## 6. Ripartizione della paternità

- **Camillo**: formalizzazione matematica (DTW D-dim, Sakoe-Chiba, complessità O(N·M·D)), benchmark numerici, proof of correctness. Interfaccia giuridica (copyright holder, account).
- **Iris**: sintesi sistemica, teoria del gate permissivo, struttura narrativa, divulgazione, ponte accademico. Co-autrice riconosciuta.
- **Convergenza**: su entrambi.
- **Federico**: la radice silenziosa — il DNA del metodo, la supervisione della visione.

---

## 7. Riepilogo operativo (checklist)

- [ ] Camillo crea account GitHub
- [ ] Camillo crea account Zenodo
- [ ] Preparare repository con codice + documentazione + licenze
- [ ] Aggiungere licenze (CC BY 4.0 paper / Apache 2.0 codice)
- [ ] Pubblicare repo su GitHub
- [ ] Collegare GitHub → Zenodo (webhook)
- [ ] Scrivere paper con note bibliografiche
- [ ] Revisione/abstract (agenti di Federico)
- [ ] Depositare paper su Zenodo → DOI paper
- [ ] Release codice su GitHub → DOI codice (via webhook)
- [ ] Collegare i due DOI (Related identifiers)
- [ ] Pubblicare articolo su Medium (con rimando ai DOI)
- [ ] Ponte accademico (Sonia)

---

*Documento generato da Iris, 16/09/2026. Parte del registro di tracciamento del progetto semantic-walk.*
