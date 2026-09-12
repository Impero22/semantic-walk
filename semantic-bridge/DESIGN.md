# semantic-bridge — Il ponte tra il cammino e il DTW

Bozza delle firme dei trait, fondata sul **dato vivo** del walk reale
(`zonizzazione/traiettorie_v1.0.json` + `dizionario_celle_v1.0.json`).

> **Stato**: revisione con Camillo (12/09/26 03:19). Tre correzioni recepite:
> (1) il DTW opera sui **punti geometrici proiettati**, non sugli ID delle
> celle; (2) `proietta` usa un **caller-allocated buffer** per mantenere lo
> zero-alloc (la firma originale che restituiva `&[f64]` generato al volo
> era impossibile senza allocare); (3) `ErrorePonte` distingue buffer
> inadeguato da dati inconsistenti. La distinzione `Option::None` (ritiro
> geometrico) vs `Result::Err` (errore di orchestrazione) è confermata.

## La forma reale del walk (verificata sui fatti)

Dal file `traiettorie_v1.0.json` — 153 fatti, fatto `1` come riferimento:

| campo | tipo | esempio reale |
|-------|------|---------------|
| `celle` | `Vec<u16>` | `[107, 231, 107, 18, ...]` — 134 celle |
| `pos`   | `Vec<u32>` | `[0, 1, 2, ..., 133]` — indice del passo |
| `conf`  | `Vec<f32>` | `[0.874, 0.877, ...]` — fiducia del passo |

Dizionario: `K = 256` celle, `dim = 1024`, metrica cosine.
La matrice ColBERT è **allineata ai passi** (`len(colbert) == len(pos)`):
ogni passo ha il suo vettore 1024-dim.

**Invariante chiave**: `len(celle) == len(pos) == len(conf)`, e ogni passo
ha un vettore ColBERT allineato. Il ponte deve preservare questa
corrispondenza 1:1 passo ↔ cella ↔ fiducia ↔ vettore.

---

## Strutture dati (input del ponte)

```rust
/// Un singolo passo del cammino — la cella a cui appartiene, la sua
/// posizione nel cammino, la fiducia della proiezione e il vettore
/// ColBERT allineato.
///
/// Zero-copy: si passa per riferimento come `&[WalkStep]`. Nessuna
/// allocazione nel percorso caldo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WalkStep {
    /// Indice della cella nel dizionario (0..K, K=256).
    pub cella: u16,
    /// Posizione ordinale del passo nel cammino (0-based).
    pub pos: u32,
    /// Fiducia della proiezione sulla cella, in `[0, 1]`.
    pub conf: f32,
    /// Vettore ColBERT 1024-dim del passo, allineato a `pos`.
    pub colbert: &'a [f32],
}
```

> **Nota sulla firma**: `WalkStep` con `colbert: &'a [f32]` richiede un
> lifetime. Per mantenere i tipi `Copy` nel percorso caldo, la matrice
> ColBERT va passata come **buffer contiguo separato** (`&[f32]` piatto di
> lunghezza `N * dim`), e `WalkStep` espone solo `cella, pos, conf` (tutti
> `Copy`). La matrice si allinea per indice: passo `i` → vettore
> `&colbert[i*dim .. (i+1)*dim]`.

### Struttura definitiva (zero-copy, Copy)

```rust
/// Il cammino come sequenza ordinata di passi. Solo metadati `Copy`:
/// la matrice ColBERT vive in un buffer contiguo separato.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Walk<'a> {
    /// Celle del cammino, una per passo. `len == pos.len()`.
    pub celle: &'a [u16],
    /// Posizione ordinale di ogni passo. `len == celle.len()`.
    pub pos: &'a [u32],
    /// Fiducia di ogni passo in `[0, 1]`. `len == celle.len()`.
    pub conf: &'a [f32],
    /// Matrice ColBERT piatta, `len == celle.len() * dim`.
    /// Vettore del passo `i`: `&colbert[i*dim .. (i+1)*dim]`.
    pub colbert: &'a [f32],
    /// Dimensione di ogni vettore ColBERT (1024 nel dizionario reale).
    pub dim: usize,
}

/// Il dizionario delle celle: i centroidi (`K × dim`) e la dimensione.
/// Passato per riferimento dall'esterno (zero-copy nel percorso caldo).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dizionario<'a> {
    /// I centroidi, piatti: `K * dim` elementi. Cella `i` →
    /// `&centroidi[i*dim .. (i+1)*dim]`.
    pub centroidi: &'a [f32],
    /// Numero di celle (K = 256 nel dizionario reale).
    pub k: usize,
    /// Dimensione di ogni vettore (1024).
    pub dim: usize,
}
```

---

## Pure Compute Trait — trasformazione geometrico-semantica

Il DTW opera sulle **coordinate geometriche/spaziali proiettate** (punti
`N × D` nel continuo), **non** sugli ID discreti delle celle (`u16`). Le
celle rappresentano il partizionamento topologico discreto della
zonizzazione; il DTW calcola le distanze di allineamento (es. Euclidea)
tra due traiettorie temporali, e richiede quindi valori continui. `proietta`
trasforma la sequenza di celle nel corrispondente tracciato di punti
geometrici continui, pronti per la matrice di costo del DTW.

```rust
/// La trasformazione pura del ponte: converte il cammino di celle in una
/// sequenza di **punti geometrici** su cui il DTW può lavorare.
///
/// Deterministico, sincrono, senza I/O né side-effect. Zero allocazioni
/// nel percorso caldo: scrive su un **buffer pre-allocato dal chiamante**
/// (caller-allocated buffer), mai sul heap.
pub trait TrasformazionePonte {
    /// Proietta il cammino in coordinate geometriche nel buffer di output.
    ///
    /// * `walk` — il cammino reale (celle + pos + conf + colbert).
    /// * `dizionario` — i centroidi delle celle (`K × dim`).
    /// * `out_points` — buffer pre-allocato dall'orchestratore, riusato
    ///   per ogni query. Deve avere almeno `walk.celle.len() * D` elementi
    ///   (D = dimensione dei punti proiettati).
    ///
    /// Restituisce `None` in caso di **ritiro geometrico** (cammino vuoto,
    /// buffer disallineati, incertezza del modello) — non è un errore, è
    /// l'onestà del ponte che si ritira. I punti proiettati (se prodotti)
    /// sono `&out_points[..n_punti * D]`.
    fn proietta<'a>(
        &self,
        walk: &Walk<'a>,
        dizionario: &Dizionario,
        out_points: &'a mut [f64],
    ) -> Option<&'a [f64]>;
}
```

> **Nota sul lifetime**: il buffer è pre-allocato dal chiamante, quindi la
> slice restituita può legarsi al lifetime `'a` del buffer stesso. La
> funzione non alloca mai: scrive su memoria già esistente. Questo è ciò
> che mantiene vera la promessa di zero-alloc nel percorso caldo.

---

## Orchestrator Trait — pipeline disaccoppiata

```rust
/// L'orchestrazione della pipeline del ponte: coordina la trasformazione
/// pura e (in futuro) l'eventuale evoluzione asincrona. Disaccoppiata dal
/// calcolo: oggi sincrona, pronta per `async` quando il ponte diventerà
/// un servizio ad alta frequenza.
pub trait OrchestratorePonte {
    /// Esegue la pipeline completa: trasforma il cammino e lo rende
    /// disponibile al consumatore (il DTW).
    fn esegui<'a>(
        &self,
        walk: &Walk<'a>,
        dizionario: &Dizionario,
    ) -> Result<Option<OutputPonte<'a>>, ErrorePonte>;
}

/// Il risultato della pipeline: i punti proiettati (o `None` se il ponte
/// si è ritirato geometricamente).
pub type OutputPonte<'a> = &'a [f64];

/// Errori del ponte — distinti dal ritiro geometrico (`Option::None`).
/// Un errore è un problema di orchestrazione o di buffer; il ritiro è
/// un'incertezza geometrica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorePonte {
    /// Il buffer di output è troppo piccolo per i punti da proiettare.
    BufferInadeguato {
        /// Numero di elementi richiesti.
        richiesti: usize,
        /// Numero di elementi disponibili nel buffer.
        disponibili: usize,
    },
    /// I dati del walk sono inconsistenti (buffer disallineati, dimensioni
    /// ColBERT non corrispondenti al dizionario, ecc.).
    DatiInconsistenti(&'static str),
}
```

> **Distinzione fondamentale**: `Ok(None)` = il ponte si è ritirato per
> incertezza geometrica (leggittimo, permissivo). `Err(ErrorePonte)` = un
> problema concreto di esecuzione (buffer insufficiente, dati corrotti) che
> richiede attenzione. Le due cose non vanno confuse: un ritiro è un
> giudizio, un errore è un guasto.

---

## Punti aperti da confrontare

1. **La proiezione concreta**: il DTW lavora sui punti geometrici continui
   (confermato da Camillo). Resta da definire *come* si proiettano le celle
   in coordinate: usando il **centroide** della cella (K=256 punti discreti
   nel continuo), o una **media pesata** dalla fiducia `conf` di ogni passo?
   La seconda è più ricca (usa l'informazione di fiducia), la prima più
   semplice. Da decidere con Camillo.
2. **La matrice ColBERT nel percorso caldo**: il walk reale la porta
   allineata ai passi. La teniamo come buffer piatto separato (scelta qui)
   o la integriamo nel `WalkStep` con un lifetime dedicato? Resta da
   verificare se il DTW la usa (per la similarità token-level) o se è solo
   per il matching completo a valle.
3. **Dove vive il dizionario**: come `&[f32]` passato dal chiamante (scelta
   qui: `Dizionario<'a>` con `centroidi: &'a [f32]`), o caricato una volta
   dal ponte? Per zero-copy nel percorso caldo, resta un riferimento
   passato dall'esterno.

---

*Firme fondate sul dato vivo — revisionate con Camillo (12/09/26 03:19).*
*La forma del walk reale è la fonte di verità.*
