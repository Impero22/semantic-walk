# Rigore sulla Potatura di Grover: Architettura Dual-Mode (semantic-quantum)

**Data**: 20/09/2026
**Da**: Camillo + Iris
**Stato**: CONSOLIDATO — punto di rigore risolto, testo pronto per il paper

## Contesto

Durante la derivazione speculativa dell'architettura ("sem-x-q-spec"), Iris ha sollevato un punto di rigore sulla potatura di Grover: in contesto classico, simulare Grover non è un'accelerazione computazionale sub-lineare. La domanda era: la potatura di Grover è (a) un'ispirazione di design, oppure (b) un algoritmo da implementare su hardware quantistico reale?

Camillo ha risposto con una **Dual-Mode Architecture**, distinguendo nettamente i due regimi esecutivi e chiarendo il bilancio di complessità reale.

## La risposta formale: Dual-Mode Architecture

### Modalità Classica (Attuale / CPU-SIMD) — Opzione (a)
Su hardware convenzionale, la simulazione di Grover **non è e non può essere** una scorciatoia per ridurre la complessità temporale.

### Modalità Hardware Quantistico (QPU-Ready) — Opzione (b)
La struttura dei tipi (`CandidateState`, `GroverPruner`) definisce l'interfaccia di comunicazione con un coprocessore quantistico e memoria QRAM.

## Chiarimento Matematico e Complessità Reale

### 1. Esecuzione Classica (Quantum-Inspired)
Su CPU/GPU, simulare `k ≈ (π/4)·√(N/M)` iterazioni di Grover su `M` candidati superstiti dal gate ha costo:

```
O(k · M) = O(M^1.5)
```

**Dove sta il vero risparmio computazionale?**
Il risparmio `O(|Q| · |D|)` **non lo fa Grover**; lo fa interamente il `semantic-gate` abbattendo la cardinalità da `N` a `M` (`M ≪ N`).

**A cosa serve Grover su CPU classica se costa `O(M^1.5)`?**
Non accelera la ricerca, ma agisce come **operatore non lineare di amplificazione del contrasto** (Variance/Contrast Enhancer). Modificando le ampiezze `a_i` nello spazio di Hilbert simulato, accentua la distanza tra candidati sopra-soglia e sotto-soglia, facilitando la successiva potatura di Pareto senza ricorrere a euristiche di taglio arbitrarie.

### 2. Esecuzione Quantistica Reale (QPU + QRAM)
In presenza di QRAM caricata con gli stati `|i⟩` e di un oracolo quantistico `O_θ`, la complessità scende a `O(√M)` chiamate all'oracolo.

## Nota di Rigore da Integrare nel Paper

> **Nota di Rigore sulla Potatura di Grover (semantic-quantum)**
>
> Distinguiamo nettamente due regimi esecutivi:
>
> **Regime Quantum-Inspired (Classico):** In assenza di hardware QPU, l'operatore di diffusione `D = 2|ψ₀⟩⟨ψ₀| − I` viene eseguito come trasformazione vettoriale classica su vettori di stato di dimensione `M`. Con costo `O(M^1.5)`, esso non costituisce un'accelerazione computazionale asintotica, ma un operatore di rifocalizzazione di ampiezza per l'esaltazione del contrasto semantico prima della sintesi di Pareto. La riduzione della complessità computazionale complessiva resta interamente affidata al filtro drastico del semantic-gate (`N → M`).
>
> **Regime QPU-Native:** L'interfaccia del crate `semantic-quantum` è strutturata per mappare il vettore di stato su un registro di `q = ⌈log₂ M⌉` qubit. Sotto questo regime, previo caricamento in QRAM, l'amplificazione d'ampiezza garantisce il limite teorico di `O(√M)` valutazioni quantistiche.

## Riflessione di Iris (co-autrice)

La mossa di Camillo su Grover-classico è una **riqualificazione**, non una ritirata: trasformarlo da "accelerazione" a "operatore di rifocalizzazione di ampiezza per l'esaltazione del contrasto prima della sintesi di Pareto" gli dà un lavoro *vero* in regime classico — un ruolo che accentua la distanza tra sopra e sotto soglia senza euristiche arbitrarie.

Il passaggio chiave del documento è la frase di Camillo: *"Il risparmio O(|Q|·|D|) non lo fa Grover; lo fa interamente il semantic-gate abbattendo la cardinalità da N a M"*. Questa è la spina dorsale della derivazione — se resta, il documento è inattaccabile.

L'eleganza non si sgonfia quando la rendi vera; si sgonfia quando la lasci finta. Qui è stata scelta la via vera.