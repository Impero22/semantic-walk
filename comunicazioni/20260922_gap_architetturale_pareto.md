# Sintesi: il benchmark ha scoperto un gap architetturale, non un bug del generatore

## Il sintomo
`bench_pareto` e `probe3` mostrano `scarto=0.0` ovunque: il Pareto candidate-level
non filtra NESSUN candidato. La frontiera = tutti i candidati.

## La causa profonda
`WalkBranch` (semantic-quantum/src/lib.rs:64) memorizza SOLO:
- `action`: f32 — l'azione totale pesata (scalare)
- `amplitude`: f32 — exp(-S/k)

I tre assi grezzi (`s_inertial`, `s_geometric`, `s_colbert`) vengono calcolati in
`build_branch`, combinati in `total_action = w_I·S_I + w_G·S_G + w_C·S_C`, e POI
SCARTATI. Non sono mai memorizzati sul ramo.

## La conseguenza
`aggrega_per_candidato` non ha accesso ai tre assi → usa `action` e `amplitude`
come proxy. Ma:
- `action` è uno scalare pesato (1 asse, non 3)
- `amplitude` è exp(-S/k) (ancora 1 asse)

Tre assi collassano in uno. La dominanza che inietto nei centri NON sopravvive
alla trasformazione proxy: nel proxy, i candidati risultano tutti incomparabili
a coppie → frontiera = tutto → scarto = 0.

## Il teorema di Camillo resta valido
Il Pareto candidate-level post-collapse (selezione tra candidati GIÀ collassati,
ortogonale all'argmax) è corretto. Il problema è che l'INFRASTRUTTURA dati non lo
supporta: non esiste un vettore a 3 assi per candidato su cui fare il confronto.

## Le opzioni di design
1. **Estendere WalkBranch** con i tre assi grezzi (s_inertial, s_geometric,
   s_colbert) in aggiunta ad action/amplitude. Impatto: footprint memoria per ramo,
   ma abilita il Pareto candidate-level reale. Il collapse può ignorarli (usa solo
   action/amplitude), il Pareto candidate-level li usa per l'aggregazione.
2. **Struttura parallela**: un map candidate_id → (s_i, s_g, s_c) costruito a monte
   (dove gli assi esistono ancora) e passato al Pareto candidate-level. Nessun
   cambio a WalkBranch, ma richiede di trasportare la struttura accanto ai rami.

## La domanda per Camillo
Quale delle due? La (1) è più pulita (il ramo porta con sé la sua informazione),
la (2) è meno invasiva ma frammenta il dato su due strutture.
