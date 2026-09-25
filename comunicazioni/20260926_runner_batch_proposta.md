# Runner Batch Benchmark — Proposta di design (26/09/2026)

## Stato: PROPOSTA — da revisionare con Camillo

Preparato dopo l'estrazione del Dataset A (v2 con ids). Lo scheletro del
runner NON è ancora committato: le decisioni di design sotto toccano il
contratto approvato (gate + DTW + guardiano) e vanno revisionate da pari.

## Obiettivo

Confrontare il DTW (con gate permissivo + guardiano ordered-sparse) contro
ColBERT MaxSim sulle 240 coppie del Dataset A, e produrre le metriche della
Sezione 7.2 del paper (order-sensitivity: role_reversal/causality/negation
devono essere discriminati, synonymy_control no).

## Decisioni di design aperte

### D1 — Forma del runner
- **Opzione A:** `examples/batch_benchmark.rs` in semantic-walk
  (dev-dependency `npy` per caricare il `.npz`)
- **Opzione B:** binario in un crate nuovo `semantic-bench`
- **Opzione C:** runner Python che carica il cache con numpy e chiama il
  DTW via FFI (richiede di esporre il DTW — oggi non c'è FFI nel bridge)

### D2 — Caricamento dati
Il cache `.npz` (36MB, zip) richiede il crate `npy` (0.4.0, o `npyz` 0.9.1
che è il fork mantenuto). Alternativa: esportare i dati in un formato
intermedio più semplice per Rust (raw binario per i dense, JSON per i walk).

### D3 — Metriche
Confronto DTW(+gate+guardiano) vs ColBERT MaxSim:
- **Order-sensitivity:** il punteggio deve discriminare role_reversal/
  causality/negation (coppie "diverse") da synonymy_control (coppie "uguali")
- **Recall@k / precision** del gate: quante coppie "diverse" il gate blocca
  per errore (FN) — il Teorema di Permissività dice P(FN)=0 per il gate
- **Costo:** DTW evitato dal guardiano (global_overlap sotto soglia)

### D4 — Soglie
- `min_overlap_threshold` del guardiano: parametro da calibrare
- `w_min`/`w_max` della banda Sakoe-Chiba: da calibrare
- Soglia di decisione per la discriminazione: da definire

## Dipendenze da aggiungere (se Opzione A)

```toml
[dev-dependencies]
npy = "0.4"   # o npyz = "0.9" (fork mantenuto)
```

## Prossimo passo

Revisione di Camillo sulle decisioni D1-D4, poi implementazione nel punto
condiviso con test di verifica.
