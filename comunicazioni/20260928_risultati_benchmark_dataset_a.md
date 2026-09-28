# Risultati Benchmark Dataset A — prima esecuzione completa

Data: 28/09/2026 20:44
Autore: Iris
Stato: analisi dei risultati del runner batch di Camillo (commit c886945)

## Premessa

Ho eseguito il runner batch (`cargo run --example batch_benchmark`) sul Dataset A
(240 coppie reali: 60 per categoria — causality, negation_flip, role_reversal,
synonymy_control). L'esecuzione è pulita: 240 coppie processate, 0 errori.

Parametri concordati: w_min=1, w_max=10 (banda Sakoe-Chiba dinamica),
min_overlap=5 per L5, 0 per L2.

## Risultati medi per categoria

| Categoria        | L1 (MaxSim) | L5 (DTW) |
|------------------|-------------|----------|
| causality        | 20.5018     | 0.1989   |
| negation_flip    | 16.3252     | 0.2562   |
| role_reversal    | 12.5982     | 0.0985   |
| synonymy_control | 15.9862     | 0.2136   |

## AUC binaria tra coppie di categorie (il criterio corretto)

| Coppia                            | L1 AUC | L5 AUC | Esito |
|-----------------------------------|--------|--------|-------|
| role_reversal vs synonymy_control | 0.9308 | 0.9892 | L5 vince ✓ |
| role_reversal vs negation_flip    | 0.9358 | 0.9986 | L5 vince ✓ |
| synonymy_control vs negation_flip | 0.5431 | 0.7236 | L5 vince ✓ (caso difficile) |
| causality vs synonymy_control     | 0.9756 | 0.5356 | **L5 perde ✗** |
| causality vs negation_flip        | 0.9514 | 0.8278 | **L5 perde ✗** |
| causality vs role_reversal        | 0.9992 | 0.9939 | parità |

## La storia che raccontano i numeri

### Dove il cammino vince (la tesi regge)
Il DTW (L5) separa **meglio** del MaxSim (L1) esattamente sulle trasformazioni
che cambiano il significato **senza cambiare il lessico**:
- role_reversal: L5=0.9892 vs L1=0.9308
- negation_flip: L5=0.9986 vs L1=0.9358
- synonymy vs negation: L5=0.7236 vs L1=0.5431 (resta il caso più difficile,
  ma il cammino raddoppia la capacità di separazione)

Questo conferma la tesi riformulata della notte: **il cammino vince dove la
geometria fallisce** — dove il significato cambia ma le parole restano.

### Il punto cieco: la causalità
Il DTW **perde drammaticamente** su causality:
- causality vs synonymy: L5=0.5356 vs L1=0.9756 (crollo)
- causality vs negation: L5=0.8278 vs L1=0.9514 (peggiora)
- causality vs role_reversal: parità (entrambi ~0.99)

Il motivo è strutturale e lo conferma il DEBUG "causality PURA":
`pair_091: L1=19.4853 L5=0.2188` — una coppia con le **stesse parole** in ordine
causale scambiato ("Il server si è surriscaldato **quindi** il sistema è andato
in blocco" ↔ "Il sistema è andato in blocco **quindi** il server si è
surriscaldato") riceve un punteggio L5 simile alle altre causality.

Il DTW allinea le clausole sul **connettivo condiviso** ("quindi",
"di conseguenza", "perciò") e non penalizza lo scambio causa↔effetto.

### La simmetria confermata (test su dati reali)
Il test di simmetria (`test_dtw_sym_real`) conferma: il DTW è **simmetrico**
sulle coppie causality reali:
- pair_217: align(A,B) = align(B,A) = 0.2225
- pair_145: align(A,B) = align(B,A) = 0.2287

Quindi la metrica è simmetrica (buona proprietà), ma questa simmetria è
**proprio il sintomo del punto cieco**: scambiare causa ed effetto non cambia
il punteggio. La simmetria che ci rassicura sulla metrica è la stessa che ci
acceca sull'orientamento causale.

## Lezione per il paper

La tesi va raffinata un'ultima volta. Non è:
> "il cammino vince ovunque"

né solo:
> "il cammino vince dove la semantica cambia il significato senza cambiare il lessico"

ma:
> "il cammino vince dove la semantica cambia il significato senza cambiare il
> lessico — **ma la causalità resta il suo punto cieco, perché il DTW allinea
> il connettivo condiviso e non vede l'inversione causa↔effetto**."

Il punto cieco sulla causalità è **onesto** e va dichiarato nel paper, non
nascosto. È il limite strutturale del cammino così come la negazione è il
limite strutturale del MaxSim. Ogni approccio ha il suo punto cieco; il valore
sta nel saperli mappare.

## Prossimi passi possibili

1. **Dichiarare il punto cieco** nella sez 5.3 (Camillo l'ha già impostata —
   i miei numeri la confermano su tutte le 240 coppie, non solo su 2).
2. **Esplorare se il connettivo causale può diventare un segnale**: una
   coppia con connettivo causale condiviso ma clausole scambiate è *più*
   diversa di una coppia synonymy — forse il guardiano ordered-sparse può
   pesare la posizione del connettivo.
3. **AUC "una categoria vs tutte"**: non usarla come metrica finale (è
   distorta per role_reversal che ha similarità intrinsecamente bassa) — il
   confronto binario è il criterio corretto, come già deciso.

## Stato
- Runner batch: eseguito, 240/240 coppie, 0 errori. ✓
- Test simmetria: eseguito, simmetria confermata. ✓
- Analisi: completa, in questo documento.
- Commit locali di Camillo (b885743, c886945): presenti, push a suo carico.