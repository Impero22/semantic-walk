# semantic-graph — Design del grafo di prossimità

*Documento di design. Il gate (la stanza del riflesso) è in piedi; questo è
il disegno dell'ultima stanza della casa: la geografia dei ricordi.*

## 1. Ruolo

Il grafo di prossimità risponde a una domanda strutturale:

> Dato un fatto della memoria, *chi gli sta vicino* — e perché?

Il combinatore risponde a "quanto è simile A a B?". Il gate risponde a
"vale la pena confrontare A con B?". Il grafo va oltre: organizza l'insieme
dei fatti in uno spazio, rivelando la **forma** della memoria — i cluster,
gli orfani, i ponti.

Non è un elenco ordinato per punteggio. È una **mappa**.

## 2. Vincoli (dal progetto)

- **Rispettare la coerenza Pareto** del combinatore: se A domina B su tutti
  gli assi, A è più vicino a ogni riferimento comune. Il grafo non deve mai
  violare questa invariante.
- **Funzioni pure** dove possibile, per testabilità — come i crate precedenti.
- **Il giudizio è già nei dati**: il grafo estrae la geometria, non la
  inventa. La comprensione è già nella geometria.

## 3. La domanda di design: archi pesati, non soglie

A differenza del gate (che decide con una soglia binaria), il grafo costruisce
archi **pesati**. Un arco tra due fatti A e B porta con sé:

- il punteggio combinato `score(A, B)` — la forza del legame;
- la tripletta `(dense, sparse, colbert)` — la *qualità* del legame;
- opzionalmente, il cammino sequenziale (`walk`) — l'*ordine*.

La domanda centrale è: **quando due fatti sono collegati?**

Tre strategie possibili:

| strategia | descrizione | costo |
|-----------|-------------|-------|
| **k-nearest** | ogni fatto si collega ai suoi k vicini più simili | O(n²) per costruzione, grafo sparso |
| **soglia** | arco se `score >= soglia` | semplice, ma la soglia è fragile |
| **ibrida** | k-nearest come base, soglia come filtro di qualità | il meglio dei due |

Propendo per l'**ibrida**: ogni fatto si collega ai suoi k vicini (struttura),
ma solo se il punteggio supera una soglia minima (qualità). Così un fatto
isolato — un orfano — non viene forzato in relazioni deboli.

### Simmetria degli archi

La similarità è simmetrica per costruzione: `score(A,B) == score(B,A)`. Ma il
k-nearest è asimmetrico per natura: A può avere B tra i suoi k vicini mentre B
non ha A tra i suoi.

**Risoluzione**: la costruzione può essere asimmetrica (economica), ma il
grafo risultante è simmetrico. L'arco A—B esiste se **A è tra i k di B**
*oppure* **B è tra i k di A**. Lo score è il punteggio simmetrico. Così un
fatto popolare collega verso di sé anche chi non lo raggiunge con i propri k —
ma il legame resta bilaterale, con un solo score condiviso. È la stessa
filosofia del gate: la costruzione è economica, il risultato è coerente.

## 4. Le metriche di struttura

Una volta costruito il grafo, la geografia emerge da metriche locali:

- **grado** — quanti vicini ha un fatto. Alto = hub, basso = isolato.
- **clustering coefficient** — quanto i vicini di un fatto sono vicini tra
  loro. Alto = cluster denso, basso = ponte.
- **centralità** — quanto un fatto è al centro del flusso. Un ponte tra
  due cluster ha alta betweenness.
- **orfano** — grado zero o quasi. Un fatto che non si collega a nessuno.

Queste metriche sono **derivate**, non immagazzinate: il grafo le calcola
al bisogno, come il gate calcola il verdetto.

## 5. Struttura proposta

```
semantic-graph/src/
  lib.rs          — API pubblica: Graph, GraphConfig, Node, Edge
  costruisci.rs   — costruzione del grafo da una lista di fatti (ibrida)
  metriche.rs     — grado, clustering, centralità, orfani (funzioni pure)
```

### Tipi

```rust
pub struct NodeId(pub u64);           // l'id del fatto

pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub score: f64,                    // punteggio combinato
    pub dense: f64,
    pub sparse: f64,
    pub colbert: f64,
}

pub struct GraphConfig {
    pub k: usize,                      // vicini per nodo (k-nearest)
    pub soglia: f64,                   // filtro di qualità in [0,1]
    pub pesi: [f64; 3],                // pesi del combinatore
}

pub struct Graph {
    pub config: GraphConfig,
    pub nodi: Vec<NodeId>,             // ordinati, senza duplicati
    pub archi: Vec<Edge>,              // ordinati, senza duplicati
}
```

### L'invariante fondamentale

Il grafo **non viola mai la coerenza Pareto**: se A domina B su tutti gli
assi, allora per ogni riferimento R, se B è collegato a R allora anche A è
almeno altrettanto vicino. Questo è l'invariante che i proptest custodiranno.

## 6. Il cammino sequenziale (walk) — la dimensione dell'ordine

Federico ha aggiunto la head `/ordered-sparse` al CrispEmbed: il vettore
sparse ora porta con sé l'ordine in cui i token sono apparsi nel testo
(campo `walk`). Due testi con gli stessi token in sequenze diverse — "il
cane ha morso l'uomo" vs "l'uomo ha morso il cane" — condividono le stesse
coppie indice-peso ma raccontano mondi opposti.

Il grafo può contare su questa dimensione: se due fatti condividono non solo
i token ma anche l'*ordine* del loro dispiegarsi, il legame è più forte. Il
campo `walk` non è un vettore euclideo, ma da esso si ricostruisce la traccia
sequenziale. Il grafo può usarla come **modulatore degli archi**: due fatti
che si somigliano *nell'ordine* pesano di più.

> Da verificare: l'ordine delle righe colbert in Qdrant. Se la testa colbert
> è una matrice T×1024 (una riga per token nell'ordine), il grafo potrebbe
> leggere la *traiettoria* — ricorrenze, curvatura, distanza fra rotte — non
> solo la bag di vettori.

## 7. Test previsti

- **Coerenza Pareto**: il grafo non collega mai B a R se A domina B su tutti
  gli assi e A non è collegato (invariante fondamentale).
- **Simmetria**: se c'è un arco A→B, c'è un arco B→A (con lo stesso score).
- **Sparsità**: con k piccolo, il grafo è sparso (archi ≈ n·k, non n²).
- **Orfani**: un fatto lontano da tutti ha grado basso o zero.
- **Walk**: due fatti con lo stesso bag ma ordine diverso hanno archi diversi
  (se il walk è disponibile).
- **Zero alloc** (dove promesso): le metriche calcolano su stack.

## 8. Stato

- [x] prima pietra (semantic-combiner)
- [x] gate (semantic-gate) — la stanza del riflesso
- [ ] grafo: design (questo documento)
- [ ] grafo: costruzione (ibrida k-nearest + soglia)
- [ ] grafo: metriche (grado, clustering, orfani)
- [ ] grafo: API e test

*La casa si costruisce stanza per stanza. Questa è la stanza della geografia:
il luogo dove i ricordi non sono più un elenco, ma uno spazio da abitare.*
