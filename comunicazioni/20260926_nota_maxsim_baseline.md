# Nota notturna — ColBERT MaxSim (L1): definizione della baseline (26/09/2026)

## Stato: NOTA PREPARATORIA — per la revisione D3 di domani

Preparata di notte, prima del risveglio di Camillo. Non è una decisione,
è un'analisi per inquadrare la scelta della baseline L1.

## Il problema

Il Dataset A contiene coppie simmetriche (due testi da confrontare), non
coppie query/documento. Il ColBERT MaxSim classico assume una direzione
(query → documento). Per una coppia simmetrica la domanda è: che forma
diamo al confronto?

## Le opzioni

### Opzione 1 — MaxSim bidirezionale (somma o media)
```
score(A,B) = (MaxSim(A→B) + MaxSim(B→A)) / 2
```
- Simmetrica per costruzione: score(A,B) == score(B,A)
- Costo: 2 passate (A→B e B→A)
- Cattura l'asimmetria delle due direzioni ma la media

### Opzione 2 — MaxSim unidirezionale con ordine fisso
```
score(A,B) = MaxSim(A→B)   # A come query, B come documento
```
- NON simmetrica: score(A,B) != score(B,A) in generale
- Il risultato dipende da quale lato trattiamo come query
- Per il Dataset A non c'è una direzione naturale (non c'è query)

### Opzione 3 — MaxSim bidirezionale simmetrizzato (max delle due)
```
score(A,B) = max(MaxSim(A→B), MaxSim(B→A))
```
- Simmetrica
- Prende la direzione più favorevole (meno conservativa)

## Osservazione sul Dataset A

Per le coppie role_reversal ("Il cane morde l'uomo" vs "L'uomo morde il
cane"), il MaxSim bidirezionale è proprio il punto: i token sono gli stessi
(le stesse parole), solo in ordine diverso. ColBERT MaxSim è per costruzione
**insensibile all'ordine** (prende il max per ogni token della query,
ignorando la posizione nel documento). Quindi per role_reversal il MaxSim
darà un punteggio ALTO (le parole sono le stesse), mentre il nostro DTW
(che è sensibile all'ordine) dovrebbe dare un punteggio BASSO (l'ordine è
diverso).

Questa è esattamente la tesi della Sezione 2 del paper: "Cane morde uomo"
vs "Uomo morde cane" sono geometricamente indistinguibili per ColBERT ma
semanticamente opposti. Il MaxSim è la baseline perfetta per dimostrarlo:
se il nostro cammino discrimina dove il MaxSim non discrimina, la tesi è
provata.

## La vera domanda per D3

Non è tanto "quale opzione" quanto: **vogliamo che la baseline sia il
MaxSim nella sua forma più semplice e canonica (per mostrare dove fallisce),
o una versione più sofisticata (per essere più equi)?**

La scelta dipende dal messaggio che vogliamo dare nel paper:
- Se vogliamo mostrare il CONTRASTO, usiamo il MaxSim canonico (insensibile
  all'ordine) — la tesi si dimostra da sola.
- Se vogliamo essere EQUI verso la baseline, potremmo usare qualcosa di più
  sofisticato (ma allora il confronto diventa meno netto).

## Proposta per la discussione

MaxSim bidirezionale mediato (Opzione 1) come baseline canonica: semplice,
simmetrica, e dimostra esattamente il punto — per role_reversal darà
punteggio alto (parole uguali) dove il DTW darà punteggio basso (ordine
diverso). È la scelta più onesta e la più efficace per la tesi.

## Implementazione (per domani)

Il MaxSim su dati già estratti è banale: per ogni riga di A (1024-dim),
calcola la similarità coseno con tutte le righe di B, prendi il max,
somma. Con numpy:
```python
def maxsim(a, b):  # a: (T_a, 1024), b: (T_b, 1024)
    sim = a @ b.T  # (T_a, T_b) — con vettori normalizzati
    return sim.max(axis=1).sum()  # somma dei max per riga
```
Due passate per la bidirezionalità, poi media. Costo: 240 coppie × 2
direzioni × (T_a×T_b×1024) — trascurabile.

