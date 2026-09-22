# Domanda di disegno: quale obiettivo governa il collapse?

**Data:** 22/09/26
**Autore:** Iris
**Stato:** in attesa di decisione con Camillo

## Il problema

La risposta di Sonus alla domanda sul pruning Pareto (`20260922_response_sonus_pareto.md`)
è formalmente corretta, ma è costruita su un obiettivo che **non coincide** con quello
del collapse reale. Questa discrepanza va risolta prima di scrivere il refactoring.

## I due obiettivi

### 1. Minimizzazione della somma dei costi (come formulato a Sonus)

```
score(c) = Σ_{r ∈ rami(c)} (s_inertial + s_geometric + s_colbert)
winner   = argmin_c score(c)
```

- Penalizza i candidati con molti rami "medi" (i costi si sommano).
- È l'obiettivo su cui Sonus ha basato la soluzione di candidate-level early termination.

### 2. Massimizzazione della somma delle ampiezze (il collapse reale)

Il collapse in `semantic-quantum/src/lib.rs` (riga ~261) fa:

```
Ψ(c) = Σ_{r ∈ rami(c)} exp(-a_r / κ)
winner = argmax_c Ψ(c)
```

- Premia i candidati con molti rami che convergono verso lo stesso esito
  (interferenza costruttiva: più percorsi = più ampiezza totale).
- È il cuore della metafora quantistica del crate.

## La non-equivalenza

```
exp(-(x+y)/κ) ≠ exp(-x/κ) + exp(-y/κ)
```

Quindi minimizzare la somma dei costi **non** equivale a massimizzare la somma
delle ampiezze. I due obiettivi possono produrre vincitori diversi.

## Implicazioni per il refactoring del Pareto

- **Se l'obiettivo è l'interferenza costruttiva** (più rami = più peso): il pruning
  branch-level è sbagliato per costruzione, e la soluzione di Sonus (early termination
  su costi sommati) **non si applica** — lì non si minimizzano costi ma si sommano ampiezze.
  Serve un criterio diverso, o rinunciare al pruning.

- **Se l'obiettivo è il costo totale minimo**: la somma delle ampiezze per-ramo è la
  funzione sbagliata. Andrebbe ripensata, es. `exp(-S_c/κ)` per-candidato (dove
  `S_c` è la somma dei costi del candidato), che è monotonicamente equivalente
  a minimizzare `S_c` e quindi compatibile con la soluzione di Sonus.

## Il test end-to-end

`end_to_end_pareto_pruning_prima_del_collapse` (riga 701) verifica che il pruning
Pareto preceda il collapse senza alterare il vincitore. È la consacrazione della
cosa sbagliata in **entrambi** i casi, e va riscritto — ma in direzioni diverse
a seconda dell'obiettivo scelto.

## La lettura di Iris

La metafora quantistica del crate — interferenza costruttiva, decoerenza, collasso —
punta naturalmente alla **prima** opzione: il collapse premia la convergenza di
più percorsi verso lo stesso esito. Ma è una decisione di rotta del crate, da
prendere con Camillo.