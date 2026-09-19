//! # Proiezione delle celle in punti geometrici continui
//!
//! Il DTW lavora sui **punti geometrici continui**, non sugli ID discreti
//! delle celle. Questo modulo realizza la trasformazione concreta del ponte:
//! converte il cammino di celle in una sequenza di punti nel continuo.
//!
//! Due strategie di proiezione:
//!
//! * [`ProiezionePunti::Centroide`] — proietta ogni passo sul **centroide**
//!   della cella. Semplice, deterministica, K punti discreti nel continuo.
//! * [`ProiezionePunti::MediaPesata`] — proietta ogni passo su una **media
//!   pesata** dei centroidi vicini, usando la fiducia `conf` come peso.
//!   Più ricca: usa l'informazione di fiducia che il centroide spreca.
//!
//! Entrambe sono pure, deterministiche e zero-alloc: scrivono su un buffer
//! pre-allocato dal chiamante (caller-allocated buffer).

use crate::{Dizionario, TrasformazionePonte, Walk};

/// Le strategie di proiezione delle celle in punti geometrici.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProiezionePunti {
    /// Proietta ogni passo sul centroide della sua cella.
    Centroide,
    /// Proietta ogni passo su una media pesata dei centroidi vicini,
    /// usando la fiducia `conf` come peso.
    MediaPesata,
}

impl Default for ProiezionePunti {
    fn default() -> Self {
        ProiezionePunti::Centroide
    }
}

impl TrasformazionePonte for ProiezionePunti {
    fn proietta<'a>(
        &self,
        walk: &Walk<'a>,
        dizionario: &Dizionario<'a>,
        out_points: &'a mut [f64],
    ) -> Option<&'a [f64]> {
        // Ritiro geometrico: cammino vuoto o buffer disallineati.
        if !walk.coerente() || !dizionario.coerente() {
            return None;
        }

        // Finding #1 della review: `walk.dim` e `dizionario.dim` non erano
        // mai comparati. Se il cammino è stato costruito con una dimensione
        // diversa da quella dei centroidi, i punti proiettati avranno la
        // dimensione sbagliata (quella del dizionario) mentre il chiamante
        // consumerà il buffer aspettandosi `walk.dim`. Disallineamento
        // dimensionale = ritiro geometrico, coerente con la filosofia
        // del bridge (giudizio, non guasto).
        if walk.dim != dizionario.dim {
            return None;
        }

        // La dimensione dei punti proiettati = dimensione dei centroidi.
        let d = dizionario.dim;
        let n = walk.celle.len();

        // Buffer insufficiente: il chiamante deve pre-allocare n * d f64.
        if out_points.len() < n * d {
            return None;
        }

        match self {
            ProiezionePunti::Centroide => {
                for (i, &cella) in walk.celle.iter().enumerate() {
                    let cella = cella as usize;
                    // Cella fuori range: ritiro geometrico.
                    if cella >= dizionario.k {
                        return None;
                    }
                    let base = cella * d;
                    let out_base = i * d;
                    for j in 0..d {
                        out_points[out_base + j] = dizionario.centroidi[base + j] as f64;
                    }
                }
            }
            ProiezionePunti::MediaPesata => {
                // Per ogni passo, la fiducia pesa i centroidi delle celle
                // "vicine" semanticamente: la cella corrente e le adiacenti
                // nel cammino, pesate dalla fiducia `conf`.
                //
                // Gestione dei bordi: la finestra NON viene clampata con
                // duplicazione (che raddoppierebbe il peso del passo di bordo).
                // Ai margini si usa una finestra ridotta a 2 elementi,
                // normalizzata sulla somma dei soli pesi presenti.
                for i in 0..n {
                    // Finestra locale, senza duplicare i bordi.
                    let mut peso_tot = 0.0f64;
                    let out_base = i * d;

                    // Accumulo direttamente sulla fetta di output del punto i:
                    // nessun buffer intermedio, nessun limite arbitrario su `d`,
                    // zero-alloc mantenuto. L'azzeramento evita residui di passi
                    // precedenti sul buffer caller-allocated.
                    for j in 0..d {
                        out_points[out_base + j] = 0.0;
                    }

                    let start = if i > 0 { i - 1 } else { i };
                    let end = if i + 1 < n { i + 1 } else { i };

                    for k in start..=end {
                        let ck = walk.celle[k] as usize;
                        if ck >= dizionario.k {
                            return None;
                        }
                        let w = walk.conf[k] as f64;
                        let base = ck * d;
                        for j in 0..d {
                            out_points[out_base + j] += dizionario.centroidi[base + j] as f64 * w;
                        }
                        peso_tot += w;
                    }

                    if peso_tot <= 0.0 {
                        return None;
                    }

                    // Normalizzazione sul posto.
                    for j in 0..d {
                        out_points[out_base + j] /= peso_tot;
                    }
                }
            }
        }

        Some(&out_points[..n * d])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn dizionario_esempio<'a>() -> Dizionario<'a> {
        let centroidi: &'a [f32] = &[
            1.0, 0.0, // cella 0
            0.0, 1.0, // cella 1
            0.5, 0.5, // cella 2
            1.0, 1.0, // cella 3
        ];
        Dizionario {
            centroidi,
            k: 4,
            dim: 2,
        }
    }

    #[test]
    fn centroide_proietta_sul_centroide() {
        let w = walk_esempio();
        let d = dizionario_esempio();
        let mut out = [0.0f64; 6]; // 3 passi * 2 dim
        let punti = ProiezionePunti::Centroide.proietta(&w, &d, &mut out).unwrap();
        assert_eq!(punti.len(), 6);
        // passo 0 → cella 0 → centroide [1.0, 0.0]
        assert!((punti[0] - 1.0).abs() < 1e-9);
        assert!((punti[1] - 0.0).abs() < 1e-9);
        // passo 1 → cella 1 → centroide [0.0, 1.0]
        assert!((punti[2] - 0.0).abs() < 1e-9);
        assert!((punti[3] - 1.0).abs() < 1e-9);
        // passo 2 → cella 2 → centroide [0.5, 0.5]
        assert!((punti[4] - 0.5).abs() < 1e-9);
        assert!((punti[5] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn buffer_insufficiente_ritiro() {
        let w = walk_esempio();
        let d = dizionario_esempio();
        let mut out = [0.0f64; 2]; // troppo piccolo (servono 6)
        let r = ProiezionePunti::Centroide.proietta(&w, &d, &mut out);
        assert!(r.is_none());
    }

    #[test]
    fn cammino_vuoto_ritiro() {
        let w = Walk {
            celle: &[],
            pos: &[],
            conf: &[],
            colbert: &[],
            dim: 2,
        };
        let d = dizionario_esempio();
        let mut out = [0.0f64; 6];
        let r = ProiezionePunti::Centroide.proietta(&w, &d, &mut out);
        assert!(r.is_none());
    }

    #[test]
    fn cella_fuori_range_ritiro() {
        let celle: &[u16] = &[0, 7]; // cella 7 fuori range (K=4)
        let pos: &[u32] = &[0, 1];
        let conf: &[f32] = &[0.9, 0.8];
        let colbert: &[f32] = &[1.0, 0.0, 0.0, 1.0];
        let w = Walk {
            celle,
            pos,
            conf,
            colbert,
            dim: 2,
        };
        let d = dizionario_esempio();
        let mut out = [0.0f64; 4];
        let r = ProiezionePunti::Centroide.proietta(&w, &d, &mut out);
        assert!(r.is_none());
    }

    #[test]
    fn media_pesata_usa_la_fiducia() {
        let w = walk_esempio();
        let d = dizionario_esempio();
        let mut out = [0.0f64; 6];
        let punti = ProiezionePunti::MediaPesata.proietta(&w, &d, &mut out).unwrap();
        assert_eq!(punti.len(), 6);

        // passo 0 (i=0): finestra ridotta [0, 1] (bordo, senza duplicazione)
        // acc = cella0*0.9 + cella1*0.8 = [1,0]*0.9 + [0,1]*0.8 = [0.9, 0.8]
        // peso_tot = 0.9+0.8 = 1.7
        // punto = [0.9/1.7, 0.8/1.7]
        assert!((punti[0] - 0.9 / 1.7).abs() < 1e-6);
        assert!((punti[1] - 0.8 / 1.7).abs() < 1e-6);
    }

    #[test]
    fn media_pesata_peso_zero_ritiro() {
        let celle: &[u16] = &[0, 1];
        let pos: &[u32] = &[0, 1];
        let conf: &[f32] = &[0.0, 0.0]; // tutte fiducie a zero
        let colbert: &[f32] = &[1.0, 0.0, 0.0, 1.0];
        let w = Walk {
            celle,
            pos,
            conf,
            colbert,
            dim: 2,
        };
        let d = dizionario_esempio();
        let mut out = [0.0f64; 4];
        let r = ProiezionePunti::MediaPesata.proietta(&w, &d, &mut out);
        assert!(r.is_none());
    }

    /// Regressione: la media pesata NON deve andare in panic con `dim > 64`.
    /// La criticità latente (buffer fisso `[0.0f64; 64]`) è stata rimossa:
    /// l'accumulo ora avviene direttamente sulla fetta di output.
    #[test]
    fn media_pesata_dim_1024_non_panica() {
        // Dizionario con 2 celle a dimensione 1024 (come i centroidi reali).
        let k = 2usize;
        let d = 1024usize;
        let mut centroidi = vec![0.0f32; k * d];
        // cella 0: tutti 1.0; cella 1: alternati 1.0/0.0
        for j in 0..d {
            centroidi[j] = 1.0;
            centroidi[1 * d + j] = if j % 2 == 0 { 1.0 } else { 0.0 };
        }
        let dizionario = Dizionario {
            centroidi: &centroidi,
            k,
            dim: d,
        };

        let celle: &[u16] = &[0, 1];
        let pos: &[u32] = &[0, 1];
        let conf: &[f32] = &[0.9, 0.8];
        let colbert: &[f32] = &vec![0.0f32; 2 * d];
        let w = Walk {
            celle,
            pos,
            conf,
            colbert,
            dim: d,
        };

        let mut out = vec![0.0f64; 2 * d];
        let punti = ProiezionePunti::MediaPesata
            .proietta(&w, &dizionario, &mut out)
            .expect("deve proiettare senza panic");

        assert_eq!(punti.len(), 2 * d);
        // passo 0 (bordo): finestra [0,1], acc = cella0*0.9 + cella1*0.8
        // peso_tot = 1.7 → punto = (0.9*c0 + 0.8*c1)/1.7
        // c0[j]=1, c1[j]=1 se j pari, 0 se dispari.
        for j in 0..d {
            let atteso = if j % 2 == 0 { (0.9 * 1.0 + 0.8 * 1.0) / 1.7 } else { (0.9 * 1.0 + 0.8 * 0.0) / 1.7 };
            assert!((punti[j] - atteso).abs() < 1e-6, "j={j}");
        }
    }

    #[test]
    fn dim_disallineate_ritiro() {
        // Finding #1 della review: `walk.dim` e `dizionario.dim` non erano
        // mai comparati. Un cammino costruito con dim diversa dai centroidi
        // produceva punti con la dimensione del dizionario mentre il
        // chiamante si aspettava quella del cammino → buffer mal interpretato.
        // Ora è un ritiro geometrico.
        let w = Walk {
            celle: &[0, 1],
            pos: &[0, 1],
            conf: &[0.9, 0.8],
            colbert: &[1.0, 0.0, 0.0, 1.0],
            dim: 3, // diverso dal dizionario (2)
        };
        let d = dizionario_esempio();
        let mut out = [0.0f64; 6];
        let r = ProiezionePunti::Centroide.proietta(&w, &d, &mut out);
        assert!(r.is_none(), "dim disallineate devono produrre ritiro");
    }
}
