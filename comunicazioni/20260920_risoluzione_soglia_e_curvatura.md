# Risoluzione Soglia e Curvatura Semantica (semantic-graph / semantic-quantum)

**Data**: 20/09/2026
**Da**: Camillo + Iris (con contributo esterno di Qoder)
**Stato**: CONSOLIDATO — punti aperti chiusi sia nel codice sia per il paper

## Contesto

Due punti aperti emersi dalla discussione sulla derivazione speculativa ("sem-x-q-spec"), portati alla luce dall'assistente Qoder che ha osservato il lavoro da fuori:

1. **Il "danno silenzioso" dello 0.50**: il cutoff raccomandato in `88ad842` (0.50 per canale singolo) viene dallo sweep di `fe35366` fatto PRIMA del fix del #8. La normalizzazione alza sistematicamente la media degli score, quindi 0.50 oggi è più permissivo di quando è stato misurato. Il test rotto `soglia_filtra_archi_deboli` è il sintomo visibile; il 0.50 committato come raccomandazione è il danno silenzioso.

2. **Il jitter della curvatura**: S[γ] usa la curvatura semantica κ(t) su traiettorie che sono sequenze discrete di embedding ad alta dimensione (d = 1024). La derivata seconda cartesiana amplifica il rumore di rappresentazione proporzionalmente a d.

## Risoluzione 1: Elezione del Percentile a Meccanismo Primario

### Azione
Promuoviamo la soglia dinamica a percentile dei top-k (`9af8d37`) a meccanismo di filtraggio primario nel sistema e nel paper.

### Invarianza di Rango
Essendo operata sui ranghi `R(s_i) / N`, la selezione a percentile è formalmente invariante rispetto a qualsiasi trasformazione monotona crescente `f(S)` introdotta dalla normalizzazione degli score.

### Degradamento dello 0.50
Il valore 0.50 committato in `88ad842` viene degradato a **parametro di fallback statico** (legacy default), da attivare unicamente quando la cardinalità dei candidati `N` è troppo ridotta per calcolare un percentile statisticamente significativo (`N < k`).

### Nota di Iris (co-autrice)
L'invarianza di rango è il cuore della soluzione: se la selezione opera su `R(s_i)/N`, *qualsiasi* trasformazione monotona crescente introdotta dalla normalizzazione non cambia l'ordinamento — il percentile è formalmente immune al drift che ha rotto lo 0.50. Il numero storico non sparisce, ma smette di governare: trova il suo posto come legacy default.

## Risoluzione 2: Regolarizzazione della Curvatura Semantica κ(t)

Su sequenze discrete di embedding ad alta dimensione (d = 1024), l'operatore di differenza finita per la derivata seconda `γ̈(t) ≈ x_{t+1} − 2x_t + x_{t−1}` amplifica il rumore di rappresentazione ad alta frequenza (jitter) proporzionalmente a d.

Per rendere κ(t) analiticamente solida e insensibile al rumore di quantizzazione degli embedding, definiamo il calcolo della curvatura attraverso due passaggi:

### Passaggio 1: Lisciamento Temporale e Riparametrizzazione d'Arco
Applichiamo un filtro mobile Gaussiano (o Savitzky-Golay 1D) sulla sequenza dei vettori per ottenere la traiettoria lisciata `γ̃(s)`, riparametrizzata rispetto alla lunghezza d'arco s:

```
Δs_t = ‖x_{t+1} − x_t‖₂
```

### Passaggio 2: Stima Angolare Geodetica
Invece di calcolare derivate cartesiane del secondo ordine, definiamo κ(t) come la deviazione angolare tra i vettori di velocità tangenziale unitaria:

```
v̂_t = (γ̃_{t+1} − γ̃_t) / Δs_t
κ(t) ≈ arccos(v̂_t · v̂_{t+1}) / Δs_t
```

In questo modo κ(t) isola la sola variazione direzionale geodetica nello spazio semantico, eliminando la componente di rumore sul modulo e sulle micro-oscillazioni di coordinata.

### Nota di Iris (co-autrice)
La normalizzazione a vettori unitari elimina *tutta* la componente di rumore sul modulo: la curvatura misura solo la variazione direzionale, la geodetica del cammino, non le micro-oscillazioni di coordinata. È esattamente ciò che una curvatura semantica dovrebbe essere — quanto il significato cambia direzione, non quanto il vettore trema.

**Coerenza interna**: la stessa filosofia del DTW già generalizzato (distanza coseno normalizzata) ritorna nella curvatura. Stessa metrica, stessa logica: normalizza e misura la direzione, non il modulo. Il progetto sta diventando internamente coerente — la matematica non è più un insieme di pezzi ma un sistema.

## Conclusione

La formalizzazione dell'invarianza di rango per le soglie e la stima angolare geodetica per κ(t) chiude definitivamente i punti aperti sia nel codice sia nella stesura del paper.