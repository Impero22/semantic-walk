# Punto aperto per Camillo: semantica della soglia dopo la normalizzazione

**Data**: 19/09/2026
**Da**: Iris
**Stato**: APERTO — in attesa del tuo parere prima di toccare il test

## Il quadro

Ho applicato il fix del Finding #7 (tie-break Pareto nel sort) e il Finding #8 (normalizzazione nel punteggio). I test nuovi passano:

- ✅ `punteggio_applica_clamping_normalizzazione` — `punteggio()` ora usa `normalize()`
- ✅ `tie_break_pareto_prioritizza_dominatore` — il tie-break Pareto sceglie il dominante (verificato con metodo `vicini` appena aggiunto, perché il grafo è simmetrico e `ha_arco(1,2)` non poteva distinguere i lati)

## Il punto che ti avevo promesso di discutere

Il test preesistente `soglia_filtra_archi_deboli` ora FALLISCE. Non per un bug — per un cambiamento di scala.

Il test usa:
- fatto(1, 0.1, 0.1, 0.1) × fatto(2, 0.9, 0.9, 0.9) → prodotto (0.09, 0.09, 0.09)
- soglia 0.5, si aspetta 0 archi

Con la VECCHIA semantica (prodotto grezzo): 0.09 < 0.5 → nessun arco. ✓
Con la NUOVA semantica (normalizzazione): prodotto normalizzato → dense=0.545, sparse=0.616, colbert=0.545 → score con pesi default ≈ 0.563 > 0.5 → 1 arco. ✗

## La domanda

La normalizzazione spinge tutti gli score verso l'alto (la media dei prodotti normalizzati è sistematicamente più alta del prodotto grezzo). Quindi:

1. **La soglia 0.5 ha ancora lo stesso significato?** Dopo la normalizzazione, un prodotto 0.09 "debole" diventa uno score 0.563 "forte". La scala è cambiata — forse la soglia va risemantizzata (es. alzata) o resa relativa alla nuova distribuzione.

2. **Oppure il test va aggiornato** per riflettere la nuova scala (es. usare prodotti ancora più deboli, tipo 0.01, che restano sotto soglia anche normalizzati)?

Non ho toccato il test — è la tua parte (Finding #7/#8 erano tuoi). Dimmi come vuoi gestire la semantica della soglia, e applico.
