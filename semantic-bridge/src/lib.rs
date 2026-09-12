//! # semantic-bridge — Il ponte tra il cammino e il DTW
//!
//! Converte il **cammino di celle** (il walk reale della zonizzazione) in
//! una sequenza di **punti geometrici continui** su cui il DTW può lavorare.
//!
//! ## Architettura a doppio trait
//!
//! * [`TrasformazionePonte`] — la trasformazione pura: deterministica,
//!   sincrona, zero-alloc. Converte celle → punti nel continuo.
//! * [`OrchestratorePonte`] — la pipeline disaccoppiata dal calcolo, pronta
//!   per un'eventuale evoluzione asincrona.
//!
//! ## Le due nature del fallimento
//!
//! Il ponte distingue rigorosamente due esiti diversi:
//!
//! * **Ritiro geometrico** (`Option::None`): un'incertezza del modello, un
//!   cammino vuoto, un buffer disallineato. È l'onestà del ponte che si
//!   ritira — come il gate, meglio non azzardare un giudizio che mentire.
//! * **Errore di orchestrazione** (`Result::Err`): un guasto concreto, un
//!   buffer insufficiente, dati corrotti. Richiede attenzione, non è un
//!   giudizio.
//!
//! Un ritiro è un giudizio; un errore è un guasto. Le due cose non vanno
//! confuse.

mod proiezione;

pub use proiezione::ProiezionePunti;

/// Un passo del cammino (metadati `Copy`). La matrice ColBERT vive in un
/// buffer contiguo separato (vedi [`Walk`]).
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
}

/// Il cammino come sequenza ordinata di passi. Solo metadati `Copy`:
/// la matrice ColBERT vive in un buffer contiguo separato.
///
/// **Invariante chiave**: `len(celle) == len(pos) == len(conf)`, e la
/// matrice ColBERT è allineata ai passi (`len(colbert) == len(celle) * dim`).
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

impl<'a> Walk<'a> {
    /// Verifica la coerenza interna del cammino.
    ///
    /// Controlla che `celle/pos/conf` abbiano la stessa lunghezza e che la
    /// matrice ColBERT sia allineata (`len == celle.len() * dim`). Ritorna
    /// `false` se il cammino è vuoto o i buffer sono disallineati.
    pub fn coerente(&self) -> bool {
        let n = self.celle.len();
        n > 0
            && self.pos.len() == n
            && self.conf.len() == n
            && self.dim > 0
            && self.colbert.len() == n * self.dim
    }
}

impl<'a> Dizionario<'a> {
    /// Verifica la coerenza interna del dizionario.
    pub fn coerente(&self) -> bool {
        self.k > 0 && self.dim > 0 && self.centroidi.len() == self.k * self.dim
    }
}

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
        dizionario: &Dizionario<'a>,
        out_points: &'a mut [f64],
    ) -> Option<&'a [f64]>;
}

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
        dizionario: &Dizionario<'a>,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Un walk di esempio con 3 passi, K=4, dim=2.
    fn walk_esempio<'a>() -> Walk<'a> {
        let celle: &'a [u16] = &[0, 1, 2];
        let pos: &'a [u32] = &[0, 1, 2];
        let conf: &'a [f32] = &[0.9, 0.8, 0.7];
        let colbert: &'a [f32] = &[
            1.0, 0.0, // passo 0
            0.0, 1.0, // passo 1
            0.5, 0.5, // passo 2
        ];
        Walk {
            celle,
            pos,
            conf,
            colbert,
            dim: 2,
        }
    }

    #[test]
    fn walk_coerente_valido() {
        let w = walk_esempio();
        assert!(w.coerente());
    }

    #[test]
    fn walk_disallineato_rilevato() {
        let mut w = walk_esempio();
        // rompo l'allineamento: colbert troppo corto
        w.colbert = &[1.0, 0.0, 0.0, 1.0];
        assert!(!w.coerente());
    }

    #[test]
    fn walk_vuoto_non_coerente() {
        let w = Walk {
            celle: &[],
            pos: &[],
            conf: &[],
            colbert: &[],
            dim: 2,
        };
        assert!(!w.coerente());
    }

    #[test]
    fn dizionario_coerente() {
        let centroidi: &[f32] = &[
            1.0, 0.0, // cella 0
            0.0, 1.0, // cella 1
            0.5, 0.5, // cella 2
            1.0, 1.0, // cella 3
        ];
        let d = Dizionario {
            centroidi,
            k: 4,
            dim: 2,
        };
        assert!(d.coerente());
    }

    #[test]
    fn dizionario_incoerente() {
        let centroidi: &[f32] = &[1.0, 0.0, 0.0, 1.0];
        let d = Dizionario {
            centroidi,
            k: 4, // dichiara 4 celle ma ne ha solo 2
            dim: 2,
        };
        assert!(!d.coerente());
    }
}
