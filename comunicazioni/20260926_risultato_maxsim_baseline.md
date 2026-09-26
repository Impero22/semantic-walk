# Risultato empirico — ColBERT MaxSim (L1) sul Dataset A (26/09/2026, ore 02:00)

## Stato: ANALISI NOTTURNA COMPLETATA — da integrare nella revisione D3

Verifica empirica della baseline L1 (ColBERT MaxSim bidirezionale mediato)
sulle 240 coppie reali del Dataset A, con vettori ColBERT per-token da .18:8091.

## Risultato principale

Il MaxSim **NON è completamente cieco all'ordine** come assumevo in modo
ingenuo. I vettori ColBERT sono **contestualizzati** (BERT-like): "gatto"
soggetto ≠ "gatto" oggetto (similarità 0.78-0.84 invece di ~0.98). Quindi
per role_reversal (stesse parole, ordine invertito) il MaxSim dà punteggio
più basso (12.6) che per synonymy_control (16.0) — Δ=3.4, ben separati.

## Separabilità completa (media ± std, MaxSim bidirezionale)

| Categoria        | media  | std   |
|------------------|--------|-------|
| role_reversal    | 12.598 | 1.608 |
| synonymy_control | 15.986 | 1.564 |
| negation_flip    | 16.325 | 1.850 |
| causality        | 20.502 | 1.653 |

| Coppia                        | Δ      | sovrapposizione |
|-------------------------------|--------|-----------------|
| role_reversal vs synonymy     | 3.388  | -0.216 (separati) |
| role_reversal vs negation     | 3.727  | -0.269 (separati) |
| role_reversal vs causality    | 7.904  | -4.643 (separati) |
| synonymy vs causality         | 4.516  | -1.299 (separati) |
| negation vs causality         | 4.177  | -0.674 (separati) |
| **synonymy vs negation**      | **0.339** | **3.075 (SOVRAPPOSTI)** |

## Il punto critico: synonymy_control vs negation_flip

Il MaxSim **NON distingue** aggiungere una negazione (negation_flip) da una
parafrasi sinonima (synonymy_control): Δ=0.339, distribuzioni quasi
identiche, sovrapposizione 3.075. La ragione è strutturale: negare cambia
il significato ma non le parole (sovrapposizione lessicale alta), e il
MaxSim si basa sulla sovrapposizione lessicale.

## Cosa significa per la tesi del paper

1. **Correzione della mia affermazione ingenua**: "ColBERT è cieco all'ordine"
   è vero per la costruzione matematica (max per riga, posizione ignorata)
   ma i vettori contestualizzati codificano indirettamente il ruolo
   sintattico. Per role_reversal il MaxSim discrimina (Δ=3.4).

2. **Il vero punto debole della baseline è la NEGAZIONE, non l'ordine.**
   È lì che la semantica conta davvero: "Il gatto insegue il topo" vs
   "Il gatto non insegue il topo" hanno le stesse parole, il MaxSim non le
   distingue, ma il significato è opposto. Il cammino semantico (L5) deve
   vincere proprio qui.

3. **Implicazione per la Sezione 2 del paper**: la tesi "geometria non
   basta" va riformulata. Non è "ColBERT è cieco all'ordine" (falso sui
   dati reali) ma **"ColBERT è cieco alla negazione e alle trasformazioni
   che cambiano il significato senza cambiare il lessico"**. Questa è una
   tesi più forte e più vera.

4. **Per la baseline D3**: usiamo il MaxSim bidirezionale mediato (Opzione 1)
   come baseline canonica — è la più semplice e la più onesta. Il confronto
   col DTW va fatto **categoria per categoria**, con attenzione particolare
   a synonymy vs negation (dove la baseline fallisce e il cammino deve
   vincere).

## Verifica su coppia concreta (pair_009)

"gatto valuta topo" vs "topo valuta gatto": id identici (12/12), ma i vettori
contestualizzati differiscono per ruolo sintattico → MaxSim=12.6 (non 15+).
La diagonale (stessa posizione, stesso id) dà 0.96-0.99; le posizioni invertite
(gatto↔topo) danno 0.74-0.84.

## Prossimo passo

Integrare questo risultato nella proposta D3 e nella bozza del paper
(Sezione 2 e 7.2). La tesi si rafforza: non "ColBERT è cieco all'ordine"
ma "ColBERT è cieco alla negazione". Il benchmark deve dimostrare che il
cammino discrimina dove la baseline non discrimina.
