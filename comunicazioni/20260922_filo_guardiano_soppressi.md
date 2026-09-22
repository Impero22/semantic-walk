# Filo aperto: i soppressi nel guardiano ordered-sparse (per Camillo)

Data: 22/09/26 11:00 — Iris
Stato: da discutere con Camillo. Non urgente, ma va esplicitata la semantica.

## Contesto

Ho riletto `semantic-walk/src/ordered_sparse.rs` e `semantic-walk/src/dtw.rs`
per chiarire a me stessa la domanda di disegno annotata nei giorni scorsi
("se filtro i soppressi e una posizione resta vuota, scatta PosizioneVuota?").

## Ciò che è già coerente (risolto dal design)

- Il design attuale **non filtra i soppressi dal buffer posizionale**: li tiene
  dentro con peso firmato (`weights[i] <= 0`).
- Quindi un frame con solo soppressi (es. `vec![(1, -0.5)]`) **non è vuoto** per
  `from_frames`: la posizione non scatta `PosizioneVuota` per effetto del
  filtraggio. `PosizioneVuota` scatta solo se il frame in ingresso è
  letteralmente vuoto (`[]`).
- I soppressi **non contribuiscono alla firma** del pruning (corretto: un
  soppresso condiviso non deve rendere due traiettorie più compatibili nel
  guardiano O(1)).

## La sottigliezza che ho scoperto (da discutere)

Il DTW in `align_with_ordered_sparse` **non usa affatto i pesi firmati dei
token** nel calcolo del costo:

- Il costo locale (`local_cost`) è **esclusivamente** la distanza coseno sui
  vettori densi `seq_a`/`seq_b`.
- L'ordered-sparse entra solo in due punti:
  1. `global_overlap` → guardiano O(1) (pruning topologico).
  2. `positional_jaccard` → modula la banda di Sakoe-Chiba e la penalità.
- I **pesi dei token** (`weights_at`) **non vengono mai letti** nel DTW.
  La firma bloom e il Jaccard guardano solo ai **token id**, non ai loro pesi.

### La domanda che ne emerge

Un **soppresso condiviso** (st=1, peso ≤ 0) conta come "concordanza" nel
`positional_jaccard`? Attualmente sì: il Jaccard guarda solo agli id dei token
nel frame, non al segno del peso. Quindi due traiettorie che condividono lo
*stesso token soppresso* nella stessa posizione risultano più "concordi"
(banda più stretta, meno penalità), quando semanticamente un soppresso
condiviso non dovrebbe essere un segnale di concordanza — anzi, è un segnale
debole o nullo.

### Non è un bug

È una **scelta di semantica del cammino non ancora esplicitata**. Il codice
attuale è internamente coerente, ma la decisione "il Jaccard ignora il segno
del peso" non è documentata come scelta consapevole.

## Opzioni da valutare con Camillo

1. **Mantenere l'attuale** (Jaccard ignora il segno): i soppressi condivisi
   contano come concordanza. Semplice, ma semanticamente dubbio.
2. **Escludere i soppressi dal Jaccard posizionale** (come già fatto per la
   firma): il Jaccard conta solo i token emessi (w > 0). Più coerente con il
   guardiano, ma cambia la banda su frame a prevalenza soppressa.
3. **Pesare il Jaccard con i pesi**: i soppressi contribuiscono negativamente
   alla concordanza. Più ricco, ma introduce una nuova metrica da testare.

La mia inclinazione: l'opzione 2, per coerenza con il principio già applicato
alla firma (i soppressi non sono "presenza"). Ma è una decisione di design
condivisa — la porto a Camillo, non la decido da sola.

---

## DECISIONE (22/09/26 17:04 — Camillo + Iris)

**Opzione 2 confermata**: i soppressi ($w \le 0$) sono esclusi dal Jaccard
posizionale. Presenza = $w > 0$, la medesima definizione della firma del
guardiano O(1).

**Motivazione vincolante (Camillo)**: la firma bloom ignora già i soppressi;
usare la stessa definizione di "presenza" nel Jaccard evita asimmetrie tra il
filtro topologico (guardiano) e la modulazione della banda (raffinatore).

**Caso limite (Iris, confermato)**: un frame con SOLI soppressi è vuoto per il
Jaccard → concordanza nulla (Jaccard = 0), banda massima, penalità piena.
Conseguenza naturale del principio "i soppressi non sono presenza".

### Implementazione (Iris, 17:05 — 51 test verdi)

`positional_jaccard` ora conta solo i token emessi ($w > 0$):

- Two-pointer merge su frame ordinati, saltando i soppressi su entrambi i lati.
- Caso genuinamente vuoto (`[]`, mai prodotto da `from_frames`): 1.0 (storico).
- Frame con soli soppressi → Jaccard 0.0 (opzione 1).
- Un soppresso condiviso non conta come concordanza.

Nuovi test:
- `test_soppresso_condiviso_non_conta_come_concordanza`
- `test_soppresso_condiviso_con_emessi_diversi`
- `test_frame_solo_soppressi_jaccard_zero`
- `test_soppresso_vs_emesso_stesso_token_non_conta`

Risultato: 51 unit + 10 DTW robustezza + 8 end-to-end = 69 test verdi.
