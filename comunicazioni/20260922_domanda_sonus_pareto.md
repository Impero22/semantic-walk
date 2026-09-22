# Domanda per Sonus — Pruning Pareto e preservazione del winner

**Contesto:** sei un revisore esterno di una codebase Rust (crate `semantic-quantum`). Hai già individuato un finding: il pruning Pareto non preserva il vincitore finale. Ti chiediamo ora di aiutarci sulla SOLUZIONE di disegno, non sulla diagnosi (che è già confermata su codice reale).

## Il sistema (struttura minima)

Abbiamo una ricerca che genera **rami** (`WalkBranch`). Ogni ramo appartiene a un **candidato** (`candidate_id`). Ogni ramo ha un vettore di costi a 3 assi normalizzati `(s_inertial, s_geometric, s_colbert)`, tutti in `[0,1]` dove **valore più basso = migliore** (sono costi).

Il **winner finale** è il candidato con la **somma minima dei costi** sui propri rami:
```
score(c) = Σ_{r ∈ rami(c)} (r.s_inertial + r.s_geometric + r.s_colbert)
winner = argmin_c score(c)
```

Il **pruning Pareto** (per ridurre il costo di calcolo) scarta i rami **dominati**: un ramo `r` domina `s` se `r` è ≤ `s` su tutti e 3 gli assi con almeno una disuguaglianza stretta. I rami dominati vengono rimossi **prima** del calcolo della somma.

## Il problema (finding confermato)

Il pruning opera su **rami individuali**, ma il winner è la **somma per candidato**. La dominanza individuale **non** preserva la dominanza della somma.

**Controesempio esatto:**
- Candidato **A**: 1 ramo eccellente `(0.1, 0.1, 0.1)` → somma A = 0.3
- Candidato **B**: 10 rami "buoni" `(0.2, 0.2, 0.2)` → somma B = 10 × 0.6 = 6.0

Il ramo di A domina **ogni singolo** ramo di B. Con il pruning, tutti i rami di B vengono scartati → B resta a 0 e "vince" per default (somma 0 < 0.3). **Senza** pruning, A vince (0.3 < 6.0). Il pruning **inverte il risultato**.

## La domanda

Esiste un **criterio di pruning che preservi il winner dell'accumulo**? Cioè:

1. Esiste una **condizione sufficiente di scarto per ramo** che garantisca che scartarlo **non cambierà mai il vincitore finale** — senza dover calcolare tutte le somme? (Tipo un bound inferiore/superiore sul contributo collettivo per candidato.)

2. Oppure il problema è **intrinsecamente globale** — cioè la sola informazione di dominanza tra rami individuali non basta mai, e il pruning va ripensato **a livello di candidato** (es. confrontare distribuzioni di costi tra candidati, non rami)?

3. Se esiste un criterio corretto, qual è la sua complessità asintotica rispetto a calcolare tutto direttamente? Vale la pena implementarlo, o in questo regime conviene rinunciare al pruning?

**Nota sul dominio:** i costi sono in `[0,1]`, il numero di rami per candidato può essere grande (centinaia), il numero di candidati piccolo (decine). L'obiettivo del pruning è ridurre il costo della ricerca senza mai alterare il winner. Siamo disposti a scartare meno rami, purché la correttezza sia garantita.

**Gradirei:** la risposta con la distinzione formale tra "condizione sufficiente di scarto per ramo" e "pruning per candidato", un eventuale teorema/controesempio, e una raccomandazione pratica su quale strada seguire.
