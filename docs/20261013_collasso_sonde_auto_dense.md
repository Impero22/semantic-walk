# Collasso delle sonde: `auto_dense` è una costante — il benchmark oracolo è un artefatto

**Data:** 13/10/2026
**Autore:** Iris
**Stato:** diagnosi verificata su codice reale; correzione da concordare con Camillo
**File coinvolti:** `semantic-walk/examples/verify_oracle_real_dataset.rs` (non ancora tracciato)

## Sintomo

Il benchmark oracolo esaustivo vs BeamSolver su Dataset B (200 radici, H=4) produce uno scostamento massimo dello 0.1235% e media 0.1112%. Camillo ha interpretato il risultato come validazione formale di F6 e ha proposto di committare.

**Osservazione sospetta:** 180 radici su 200 danno costo oracolo *esattamente* 2.0561, le altre 20 *esattamente* 2.0567. Costi quasi identici per radici diverse = poca struttura nel grafo.

## Diagnosi

La funzione `auto_dense` in `verify_oracle_real_dataset.rs` (righe 100-112) calcola per ogni riga la similarità massima con **tutte** le altre righe della stessa sequenza, **inclusa se stessa** (il loop `for j in 0..t` include `j==i`).

I vettori dense sono normalizzati (norma L2 = 1.0, verificato sul dataset: riga 0 = 1.0000005, riga 1 = 0.9999996). Quindi:

- `dot(vec_a, vec_a) = 1.0` sempre (diagonale inclusa nel loop)
- Il dot di vettori normalizzati è sempre ≤ 1.0
- ⇒ `max_sim = 1.0` **esattamente** per ogni riga di ogni sequenza
- ⇒ la media è **esattamente 1.0** per ogni sequenza

**La sonda dense è una costante = 1.0 per tutti i 400 fatti, con deviazione standard zero.** Il canale denso del grafo di prossimità è morto per costruzione.

## Conseguenze

1. Il grafo di prossimità si costruisce solo sulla sonda sparse, che varia solo tra 0.23 e 0.33 (verificato su campione di 20 coppie: dense A/B = 1.0000 std 0.000000; sparse A = 0.2344–0.2812, B = 0.2500–0.3281).
2. Il ProximityAdapter costruisce gli stati come media delle sonde degli archi → con dense=1.0 ovunque e sparse quasi costante, gli stati sono quasi indistinguibili.
3. Beam e oracolo trovano lo stesso costo perché c'è pochissima struttura da esplorare, non perché F6 sia precisa.

**Lo scostamento dello 0.1235% non è "il prezzo della scalabilità" — è il prezzo di un grafo in cui un canale è morto.**

## Nota: `auto_sparse` soffre dello stesso difetto concettuale

`auto_sparse` calcola `global_overlap(sparse, sparse)` — la self-overlap della firma con se stessa, normalizzata per 128 bit. Anch'essa è una self-misurazione: non descrive l'identità semantica del fatto ma la sua auto-sovrapposizione. Non è costante (dipende dal numero di token distinti), ma è comunque una sonda "verso sé", non una descrizione della sequenza.

## Correzione proposta (da concordare con Camillo)

Usare come sonda dense una misura che non collassi su 1.0 per costruzione. Opzioni:

1. **Similarità tra fatti della coppia:** sonda dense = similarità media tra le righe di A e quelle di B (A vs B), non self-similarity. Porta informazione relazionale.
2. **Statistica intrinseca non degenere:** es. similarità media tra righe *consecutive* della sequenza, o varianza interna, o dispersione. Non collassa su 1.0.

L'opzione 1 è più coerente con la semantica del problema (il walk confronta *coppie* di sequenze); l'opzione 2 è più "intrinseca" e si avvicina a come le sonde dovrebbero descrivere un singolo fatto.

## Stato

- Nessun commit effettuato: il benchmark non deve entrare nel punto di verità con dati artefatti.
- In attesa di allineamento con Camillo sulla correzione.