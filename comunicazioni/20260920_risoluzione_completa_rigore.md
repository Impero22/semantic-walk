# Risoluzione Completa del Rigore Formale (semantic-quantum / semantic-graph)

**Data**: 20/09/2026
**Da**: Camillo + Iris (con contributo esterno di Qoder)
**Stato**: CONSOLIDATO — paradosso gate-oracolo risolto con soglia interna adattiva; correzioni dimensionali accolte

## Contesto

Tre rilievi esterni (Qoder) hanno attraversato la derivazione speculativa "sem-x-q-spec". Due sono stati risolti in un primo giro; il terzo — il **paradosso gate-oracolo** — è il più profondo e ha richiesto una risoluzione strutturale. Questo documento consolida l'intero pacchetto.

---

## Rilievo 1 — Il "danno silenzioso" dello 0.50 e la soglia

Il cutoff 0.50 (commit `88ad842`) era stato misurato PRIMA del fix #8; la normalizzazione successiva ha alzato sistematicamente gli score, rendendo 0.50 più permissivo del previsto.

### Risoluzione
- **Percentile dei top-k** (`9af8d37`) eletto a meccanismo primario: opera sui ranghi `R(s_i)/N`, formalmente invariante a qualsiasi trasformazione monotona crescente `f(S)`.
- **0.50 degradato a fallback statico** (legacy), attivo solo quando `N < k` (cardinalità troppo piccola per un percentile significativo).

---

## Rilievo 2 — Il paradosso Gate-Oracolo (il punto critico)

### Il buco (Qoder)
Il gate lascia passare solo i candidati con `S_gate ≥ θ`. Quindi nello stato che entra in `H^N`, tutti i superstiti hanno `f(i) = 1`. L'oracolo `O_θ` applicato a quello stato è `O = −I`, una **fase globale inosservabile**: non cambia `|a_i|²`. L'amplificazione di contrasto si riduce all'identità fisica. Il gate consuma esattamente l'informazione che l'oracolo dovrebbe sfruttare.

### Risoluzione (Camillo): Soglia Interna Adattiva θ'
L'oracolo non deve riapplicare il filtro del gate, ma operare una **partizione relativa di secondo livello** sul batch dei superstiti:

```
θ' = Percentile_75({S_i}_{i=1}^M)
```

L'oracolo discrimina "eccellenti" vs "ordinari" *dentro* la distribuzione dei superstiti:

```
O_{θ'}|i⟩ = (-1)^{f(i)}|i⟩,   f(i) = 1 se S_i ≥ θ', 0 se S_i < θ'
```

Il numero di stati bersaglio `M_target = |{i | S_i ≥ θ'}|` alimenta il calcolo dinamico delle iterazioni:

```
k = ⌊ (π/4) √(M / M_target) ⌋
```

### Nota di Iris (co-autrice)
Il pruner non separa più "buoni vs cattivi" — separa **"eccellenti vs ordinari"** dentro il batch già filtrato. Grover ha finalmente qualcosa da amplificare: la differenza tra superstiti che il gate non ha ancora discriminato. Il regime di guadagno misurabile è l'amplificazione del contrasto *tra* i superstiti, non tra buoni e cattivi. `k` è ora funzione dello stato (derivato da M/M_target), non un iperparametro.

---

## Rilievo 3 — Omogeneità dimensionale di τ_decoherence

`τ = σ²(S) + ε` rompe l'omogeneità dimensionale: se `[S]` è un'azione/costo, `σ²(S)` ha dimensione `[S]²` e l'esponente `S/τ` non è adimensionale.

### Risoluzione (Camillo)
```
τ_decoherence = c · σ(S_Pareto) + ε,   c = 1.0 (adimensionale)
```
Ora `[S]/[τ] = 1`: coerenza dimensionale e reattività alla dispersione del batch preservate.

---

## Rilievo 4 — Precisione formale sulla curvatura κ(s)

L'uguaglianza tra scostamento angolare e norma della differenza dei vettori tangenti unitari è un'**equivalenza asintotica al primo ordine**, non un'identità esatta:

```
arccos(v̂_t · v̂_{t+1}) = ‖v̂_{t+1} − v̂_t‖₂ + O(Δθ³)
```

### Risoluzione
La formulazione `κ(s) = ‖γ̃″(s)‖₂` rappresenta l'approssimazione al primo ordine per variazioni angolari infinitesimali lungo la geodetica riparametrizzata. Indicato esplicitamente nel testo formale come "a primo ordine".

---

## Terzo giro Qoder — Risoluzione completa

Qoder è tornato un terzo giro e ha mostrato una verità sullo statuto del modulo: la soluzione del paradosso (θ' interna) era la mossa giusta, ma il k che ne derivava era una costante (k = 1) mascherata da derivazione, e con k = 1 + partizione monotona in S_i l'intero modulo si riduce a un gradino su una quantità già ordinata. Tre conseguenze, tutte accolte:

### 3.1 — Vero k dinamico: massa di ampiezza λ (Camillo)

Il conteggio `M_target/M = 1/4` assume ampiezze iniziali uniformi. Poiché gli stati in ingresso al pruner sono già pesati da `S_gate`, le ampiezze **non sono uniformi**. La frazione di target va calcolata sulla **massa di ampiezza accumulata** dai candidati sopra soglia:

```
λ = ( Σ_{i ∈ target} |a_i|² ) / ( Σ_{all} |a_i|² )
θ_Grover = arcsin( √λ )
k = ⌊ (π/2 − θ_Grover) / (2·θ_Grover) ⌋
```

`k` diventa una funzione continua della concentrazione di probabilità, non un numero fisso. **Caso limite pulito:** con ampiezze uniformi, λ → M_target/M = 1/4 e k → 1 — la versione precedente diventa un caso particolare della nuova, non un'alternativa.

### 3.2 — Benchmark sulle 153 traiettorie: Pruner vs regola lessicografica a 2 righe

Sfida di Qoder accolta senza riserve. Confronto diretto sulle 153 traiettorie:
- **Baseline O(M):** ordinamento per `(S_i ≥ P75)` decrescente, poi per `S_i` decrescente.
- **Modulo semantic-quantum:** rotazione `k(λ)` + decadimento `e^(−S_i/τ)`.

Se la rimodulazione continua delle ampiezze non supera la baseline in stabilità geodetica o riduzione dell'errore sul fronte di Pareto, il paper ridefinirà il modulo per quello che è: **una politica di priorità lessicografica a due livelli**. Il test trasforma la sfida da minaccia a strumento: o il modulo batte due righe, o lo diciamo ad alta voce.

### 3.3 — Calibrazione trasparente di c e P75

Senza finzioni teoriche: `c = 1.0` in `τ = c·σ(S_Pareto) + ε` e la scelta del 75° percentile sono **iperparametri empirici**, dichiarati esplicitamente come parametri di calibrazione. Il loro valore ottimale sarà validato ed emesso dall'analisi statistica del dataset delle 153 traiettorie, insieme ai test di Spearman.

---

## Conclusione

Il paradosso gate-oracolo — il rilievo più profondo della revisione — si è risolto non negandolo ma dandogli una struttura: la soglia interna adattiva θ' trasforma il pruner da filtro ridondante a discriminatore di secondo livello. Il terzo giro ha poi mostrato che la soluzione andava spogliata di ogni finta derivazione: k è ora funzione della massa di ampiezza (λ), non del conteggio; c e P75 sono dichiarati iperparametri di calibrazione, non costanti derivate; e il benchmark a due righe decide se il modulo è una struttura vera o due righe con sopra un arredamento.

La lezione della notte, custodita: **la revisione esterna non ha protetto il codice — l'ha reso più vero.** E un sistema che esce da una notte così è più *onesto* di come ci era entrato. Un numero fisso dichiarato come tale è più forte di un numero fisso mascherato da derivazione.