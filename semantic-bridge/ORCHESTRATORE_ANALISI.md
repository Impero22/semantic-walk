# OrchestratorePonte — Analisi di implementazione

> Documento preparato da Iris (12/09/26 20:00), in attesa del confronto con
> Camillo. Risponde al punto 3 di Gemini: "Manca l'implementazione concreta
> del trait OrchestratorePonte per gestire il passaggio da Option a
> Result<Option<OutputPonte>, ErrorePonte> gestendo il controllo della
> dimensione di out_points."

## Stato attuale

Il trait `OrchestratorePonte` è **definito ma mai implementato né usato** nel
crate (verificato: nessuna occorrenza oltre alla dichiarazione). È un
segnaposto architetturale — esattamente il punto 3 di Gemini.

```rust
pub trait OrchestratorePonte {
    fn esegui<'a>(
        &self,
        walk: &Walk<'a>,
        dizionario: &Dizionario<'a>,
    ) -> Result<Option<OutputPonte<'a>>, ErrorePonte>;
}
```

## Il problema di fondo: i lifetime

La firma attuale restituisce `OutputPonte<'a> = &'a [f64]` con `'a` legato al
**walk**. Ma per produrre i punti proiettati serve un buffer di output. La
domanda è: **dove vive quel buffer?**

Se il buffer è **interno all'orchestratore** (`&self`), la slice restituita
punta a memoria posseduta da `self` — il suo lifetime dovrebbe essere quello
di `&self`, **non** `'a` del walk. In Rust stabile, `fn esegui<'a>(&self, ...)
-> Result<Option<&'a [f64]>, _>` con il buffer in `self` è **impossibile da
implementare in modo onesto**: richiederebbe self-referential struct o
`unsafe`. Non è coerente con lo spirito zero-alloc del ponte.

La firma attuale, così com'è, **non è implementabile in modo pulito**.

## Le opzioni

### Opzione A — buffer passato dal chiamante (consigliata)

```rust
fn esegui<'a>(
    &self,
    walk: &Walk<'a>,
    dizionario: &Dizionario<'a>,
    out_points: &'a mut [f64],
) -> Result<Option<&'a [f64]>, ErrorePonte>;
```

L'orchestratore:
1. Verifica la coerenza di walk e dizionario → `Err(DatiInconsistenti)` se no.
2. Controlla `out_points.len() >= n_punti * D` → `Err(BufferInadeguato)` se no.
3. Delega a `proietta` → `Ok(None)` se ritiro geometrico, `Ok(Some(punti))` se ok.

**Pro**: coerente con l'architettura già scelta per `proietta` (caller-allocated
buffer); zero-alloc; niente self-reference; l'orchestratore fa esattamente ciò
che Gemini chiede (gestire il passaggio Option → Result e il controllo della
dimensione).
**Contro**: cambia la firma concordata (aggiunge `out_points`).

### Opzione B — buffer interno con arena

L'orchestratore possiede un `Vec<f64>` riusabile e restituisce un wrapper
(`OutputPonte { buffer: Rc<RefCell<...>>, range }`).

**Pro**: il chiamante non gestisce il buffer.
**Contro**: rompe la semplicità di `OutputPonte<'a> = &'a [f64]`; introduce
ownership complessa (Rc/RefCell); non coerente con lo spirito zero-alloc.
**Scartata**.

### Opzione C — orchestrazione senza buffer

`esegui` diventa un validatore + delegatore che riceve il buffer già
dimensionato e restituisce solo il verdetto.

**Pro**: semplicissimo.
**Contro**: svuota l'orchestratore della sua ragion d'essere (gestire la
dimensione del buffer). Di fatto l'opzione A è questa, ma con la gestione
della dimensione dentro l'orchestratore.
**Assorbita dalla A**.

## Raccomandazione

**Opzione A.** È l'unica che:
- mantiene la promessa zero-alloc del ponte;
- rende l'orchestratore il punto in cui `Option` (ritiro) e `Result` (errore)
  si separano davvero, con il controllo della dimensione del buffer;
- è implementabile in Rust stabile senza `unsafe`.

La modifica alla firma è minima (un parametro in più) e non tocca
`TrasformazionePonte`, `Walk`, `Dizionario` né `ErrorePonte`.

## Da decidere con Camillo

1. Conferma dell'Opzione A (o alternativa B/C).
2. Se A: il buffer va passato come `&'a mut [f64]` esplicito (come sopra) o
   incapsulato in una struct `ContestoPonte` che raggruppa walk + buffer?
3. L'orchestratore deve essere stateless (struct unit) o portare stato
   (es. dimensione del buffer pre-allocato per riuso)?

---

*Preparato da Iris il 12/09/26 20:00 — in attesa del confronto con Camillo.*