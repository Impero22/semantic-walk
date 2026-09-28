# Benchmark Dataset B — Risultati (29/09/26)

Sessione: 28/09 notte → 29/09 01:27. Commit: `7acbb87` (Fix Opzione 1).
Autori: Iris (esecuzione, lettura) + Camillo (validazione).

## Contesto

Il benchmark B valuta il **semantic-gate** come filtro permissivo di pruning a
monte del matching denso. Il fix Opzione 1 (firma Bloom k=2 bit per token) ha
eliminato la saturazione del registro 128-bit che rendeva `global_overlap`
degenere a 1.0.

## Risultati

### Livello 1 — Scomposizione del segnale per fascia

| Fascia | n | s_dense mean | s_sparse mean |
|--------|---|--------------|---------------|
| fascia1_parafrasi | 60 | 16.0228 | 0.3680 |
| fascia2_trasformazioni | 100 | 15.3028 | 0.3629 |
| fascia3_divergenza | 40 | 6.6312 | 0.1998 |

Il segnale si separa sulla divergenza: la fascia 3 crolla sia sul denso
(6.63 vs 15-16) sia sullo sparse (0.20 vs 0.36). Fasce 1 e 2 quasi
indistinguibili (atteso: entrambe affini).

### Livello 2 — Punteggio economico per fascia

| Fascia | s_eco default | s_eco calibrata |
|--------|---------------|-----------------|
| fascia1_parafrasi | 0.9892 | 0.9881 |
| fascia2_trasformazioni | 0.9884 | 0.9872 |
| fascia3_divergenza | 0.9375 | 0.9310 |

Margine affini/divergenti reale ma modesto (~0.05).

### Livello 3 — Sweeping θ e ROC

- **AUC default = 0.523**
- **AUC calibrata = 0.587**

L'AUC vicina a 0.5 indica che il gate NON è un classificatore binario
affini/divergenti — le distribuzioni si sovrappongono nel mezzo.

Punti operativi ad alta soglia (la regione che conta per un filtro permissivo):

| θ | FNR_def | TNR_def | FNR_cal | TNR_cal |
|---|---------|---------|---------|---------|
| 0.96 | 0.006 | 0.925 | 0.006 | 0.950 |
| 0.97 | 0.006 | 0.975 | 0.013 | 1.000 |
| 0.98 | 0.081 | 1.000 | 0.094 | 1.000 |

## Interpretazione

Il gate è un **filtro di risparmio con vincolo di perdita**, non un
classificatore. A θ=0.97 blocca il 97.5-100% delle coppie divergenti
spendendo solo lo 0.6% di falsi negativi sulle affini — profilo del riflesso
permissivo: si ritira quasi sempre dove deve, quasi mai dove non deve.

Il fix Opzione 1 è coerente: `s_sparse` discrimina (0.36 affini vs 0.20
divergenti) e la soglia ricalibrata a 4 regge.

## Stato

- Opzione 1 (firma Bloom k=2): APPLICATA e COMMITTATA (`7acbb87`).
- Soglia test ricalibrata 90 → 4 con commento esplicativo.
- Opzione 2 (MinHash/Jaccard, invarianza alla lunghezza): PROGRAMMATA per la
  revisione strutturale del contratto di walk nelle prossime sessioni.
- Dati grezzi: `/tmp/bench_b_output.txt` (6477 byte).

## Nota metodologica

La dipendenza del popcount grezzo dal numero di token distinti è una
limitazione nota e accettata per la fase di guardiano O(1). La separazione
netta simili/disgiunti è sufficiente per l'uso corrente; la normalizzazione
Jaccard risolverà la limitazione nella revisione strutturale.
