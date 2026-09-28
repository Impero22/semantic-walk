#![allow(dead_code)]

//! # Benchmark Dataset B — Valutazione comparativa del semantic-gate
//!
//! Tre livelli richiesti dal progetto (documentati in tracciamento):
//!
//! **Livello 1 — Scomposizione del segnale.** Per ogni coppia si calcolano
//! le due sonde economiche grezze:
//!   * `s_dense` = MaxSim denso bidirezionale mediato (come L1 del bench A)
//!   * `s_sparse` = overlap di firma normalizzato (bit condivisi / 128)
//! Si traccia la distribuzione per fascia (parafrasi / trasformazioni /
//! divergenza).
//!
//! **Livello 2 — Valutazione comparativa del gate.** Due configurazioni:
//!   * `s_eco` default `[0.5, 0.5]` (colbert a 0, via `sonda_economica`)
//!   * `s_eco` calibrata `[0.215, 0.552, 0.233]` (colbert reale, via
//!     `combine` con `NormalizedAxes::normalize`)
//!
//! **Livello 3 — Sweeping θ e ROC.** θ ∈ [0,1] passo 0.01. Il gate blocca
//! quando `s_eco < θ` (non vale la pena spendere il colbert). Si misura:
//!   * FNR su Fascia 1+2 (parafrasi + trasformazioni): il gate blocca una
//!     coppia che era in realtà semanticamente affine → costo.
//!   * TNR su Fascia 3 (divergenza): il gate blocca una coppia che diverge
//!     → risparmio corretto.
//!
//! L'interpretazione del gate come *riflesso permissivo*: un `s_eco` basso
//! su una coppia divergente è un risparmio legittimo (non si spende il
//! matching completo su ciò che non merita); un `s_eco` basso su una coppia
//! affine è un falso negativo (il riflesso si ritira dove il giudizio
//! avrebbe dovuto passare).

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_combiner::{combine, NormalizedAxes, LAMBDA_CALIBRATO, PESI_CALIBRATI};
use semantic_gate::sonda_economica;
use semantic_walk::ordered_sparse::OrderedSparseSequence;

// ---------------------------------------------------------------------------
// Configurazione
// ---------------------------------------------------------------------------

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_b")
}

fn index_path() -> PathBuf {
    data_dir().join("dataset_b_index.json")
}

fn cache_path() -> PathBuf {
    data_dir().join("dataset_b_cache_fixed.npz")
}

/// Numero di bit della firma global (due u64).
const SIGNATURE_BITS: f64 = 128.0;

/// Peso del colbert nella sonda economica default: il gate non deve
/// indovinare un colbert non ancora calcolato, quindi lo pone a 0.
const PESI_DEFAULT: [f64; 2] = [0.5, 0.5];

// ---------------------------------------------------------------------------
// Strutture dati
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
struct IndexEntry {
    category: String,
    text_a: String,
    text_b: String,
}

type DatasetIndex = HashMap<String, IndexEntry>;

/// Una coppia caricata dal NPZ.
struct PairData {
    pair_id: String,
    category: String,
    seq_a: Vec<Vec<f64>>,
    seq_b: Vec<Vec<f64>>,
    sparse_a: OrderedSparseSequence,
    sparse_b: OrderedSparseSequence,
}

// ---------------------------------------------------------------------------
// Caricamento NPZ (stesso schema del benchmark A)
// ---------------------------------------------------------------------------

fn read_npz_array<T: npyz::Deserialize>(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    let data = reader.into_vec::<T>()?;
    Ok(data)
}

fn read_dense(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    let shape = reader.shape().to_vec();
    let flat = reader.into_vec::<f32>()?;

    let dim = if shape.len() >= 2 { shape[1] as usize } else { 1 };
    let n = if flat.is_empty() { 0 } else { flat.len() / dim };

    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        let row = flat[i * dim..(i + 1) * dim]
            .iter()
            .map(|&x| x as f64)
            .collect::<Vec<f64>>();
        rows.push(row);
    }
    Ok(rows)
}

fn build_sparse(ids: &[i64], weights: &[f32]) -> Result<OrderedSparseSequence, &'static str> {
    if ids.len() != weights.len() {
        return Err("ids e weights di lunghezza diversa");
    }
    let frames: Vec<Vec<(u32, f32)>> = ids
        .iter()
        .zip(weights.iter())
        .map(|(&id, &w)| vec![(id as u32, w)])
        .collect();
    OrderedSparseSequence::from_frames(&frames)
}

fn load_pair(
    zip: &mut ZipArchive<BufReader<File>>,
    pair_id: &str,
    category: &str,
) -> Result<PairData, Box<dyn std::error::Error>> {
    let seq_a = read_dense(zip, &format!("{}_a_dense", pair_id))?;
    let seq_b = read_dense(zip, &format!("{}_b_dense", pair_id))?;

    let ids_a: Vec<i64> = read_npz_array(zip, &format!("{}_a_ids", pair_id))?;
    let weights_a: Vec<f32> = read_npz_array(zip, &format!("{}_a_weights", pair_id))?;
    let ids_b: Vec<i64> = read_npz_array(zip, &format!("{}_b_ids", pair_id))?;
    let weights_b: Vec<f32> = read_npz_array(zip, &format!("{}_b_weights", pair_id))?;

    let sparse_a = build_sparse(&ids_a, &weights_a)?;
    let sparse_b = build_sparse(&ids_b, &weights_b)?;

    Ok(PairData {
        pair_id: pair_id.to_string(),
        category: category.to_string(),
        seq_a,
        seq_b,
        sparse_a,
        sparse_b,
    })
}

// ---------------------------------------------------------------------------
// Sonde economiche a livello di coppia
// ---------------------------------------------------------------------------

/// MaxSim denso bidirezionale mediato (identico a L1 del benchmark A).
fn compute_dense(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    let ta = a.len();
    let tb = b.len();
    if ta == 0 || tb == 0 {
        return 0.0;
    }

    let mut maxsim_a_b = 0.0f64;
    for i in 0..ta {
        let vec_a = &a[i];
        let mut max_sim = f64::NEG_INFINITY;
        for j in 0..tb {
            let vec_b = &b[j];
            let dot: f64 = vec_a.iter().zip(vec_b.iter()).map(|(x, y)| x * y).sum();
            if dot > max_sim {
                max_sim = dot;
            }
        }
        maxsim_a_b += max_sim;
    }

    let mut maxsim_b_a = 0.0f64;
    for j in 0..tb {
        let vec_b = &b[j];
        let mut max_sim = f64::NEG_INFINITY;
        for i in 0..ta {
            let vec_a = &a[i];
            let dot: f64 = vec_a.iter().zip(vec_b.iter()).map(|(x, y)| x * y).sum();
            if dot > max_sim {
                max_sim = dot;
            }
        }
        maxsim_b_a += max_sim;
    }

    (maxsim_a_b + maxsim_b_a) / 2.0
}

/// Similarità sparsa a livello di coppia: overlap di firma normalizzato.
///
/// `global_overlap` conta i bit condivisi delle due firme (0..=128).
/// Normalizzato per 128 dà una metrica sparsa in [0, 1] — la stessa che il
/// gate riceve come canale sparse grezzo.
fn compute_sparse(a: &OrderedSparseSequence, b: &OrderedSparseSequence) -> f64 {
    let overlap = a.global_overlap(b);
    (overlap as f64) / SIGNATURE_BITS
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[BENCH-B] Indice: {}", index_path().display());
    println!("[BENCH-B] Cache:  {}", cache_path().display());

    let index_file = File::open(&index_path())?;
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader)?;
    println!("[BENCH-B] Coppie totali: {}", index.len());

    let npz_file = File::open(&cache_path())?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    // Raccoglitori per fascia.
    // Per fascia salviamo le tuple (s_dense, s_sparse, s_eco_default, s_eco_calibrata).
    #[derive(Default)]
    struct Fascia {
        dense: Vec<f64>,
        sparse: Vec<f64>,
        eco_default: Vec<f64>,
        eco_calibrata: Vec<f64>,
    }
    let mut fasce: HashMap<String, Fascia> = HashMap::new();

    let mut pairs_processed = 0usize;
    let mut errors = 0usize;

    for (pair_id, entry) in &index {
        let pair = match load_pair(&mut zip, pair_id, &entry.category) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[BENCH-B] Errore caricamento {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };

        // --- Livello 1: sonde grezze ---
        let s_dense = compute_dense(&pair.seq_a, &pair.seq_b);
        let s_sparse = compute_sparse(&pair.sparse_a, &pair.sparse_b);

        // --- Livello 2: gate in due configurazioni ---
        // (a) default [0.5, 0.5], colbert a 0 — il riflesso economico puro.
        let eco_default = sonda_economica(
            s_dense,
            s_sparse,
            PESI_DEFAULT[0],
            PESI_DEFAULT[1],
        );
        // (b) calibrata [0.215, 0.552, 0.233], colbert reale = s_dense.
        //    Nota: nel gate il colbert è il matching completo; qui usiamo il
        //    MaxSim denso come sua approssimazione, perché il Dataset B non
        //    separa il matching colbert dal matching denso.
        let axes = NormalizedAxes::normalize(s_dense, s_sparse, s_dense, LAMBDA_CALIBRATO);
        let eco_calibrata = combine(&axes, PESI_CALIBRATI);

        let f = fasce.entry(pair.category.clone()).or_default();
        f.dense.push(s_dense);
        f.sparse.push(s_sparse);
        f.eco_default.push(eco_default);
        f.eco_calibrata.push(eco_calibrata);

        pairs_processed += 1;
    }

    println!("\n[BENCH-B] Coppie processate: {}, errori: {}", pairs_processed, errors);

    // -----------------------------------------------------------------------
    // Livello 1 — Distribuzione delle sonde per fascia
    // -----------------------------------------------------------------------
    println!("\n=== Livello 1: Scomposizione del segnale per fascia ===");
    let mut cats: Vec<&String> = fasce.keys().collect();
    cats.sort();
    for cat in &cats {
        let f = fasce.get(*cat).unwrap();
        let n = f.dense.len();
        let d_mean = f.dense.iter().sum::<f64>() / n as f64;
        let s_mean = f.sparse.iter().sum::<f64>() / n as f64;
        println!(
            "[{}] n={} | s_dense mean={:.4} | s_sparse mean={:.4}",
            cat, n, d_mean, s_mean
        );
    }

    // -----------------------------------------------------------------------
    // Livello 2 — Valutazione comparativa del gate
    // -----------------------------------------------------------------------
    println!("\n=== Livello 2: Punteggio economico per fascia ===");
    for cat in &cats {
        let f = fasce.get(*cat).unwrap();
        let n = f.eco_default.len();
        let d_mean = f.eco_default.iter().sum::<f64>() / n as f64;
        let c_mean = f.eco_calibrata.iter().sum::<f64>() / n as f64;
        println!(
            "[{}] n={} | s_eco default={:.4} | s_eco calibrata={:.4}",
            cat, n, d_mean, c_mean
        );
    }

    // -----------------------------------------------------------------------
    // Livello 3 — Sweeping θ e ROC
    // -----------------------------------------------------------------------
    // Fascia 1 (parafrasi) + Fascia 2 (trasformazioni) = semanticamente
    // affini → il gate deve PASSARE (s_eco >= θ). Un blocco qui è FNR.
    // Fascia 3 (divergenza) = dissimile → il gate può BLOCCARE (s_eco < θ).
    // Un blocco qui è TNR (risparmio corretto).
    let affini_default: Vec<f64> = ["fascia1_parafrasi", "fascia2_trasformazioni"]
        .iter()
        .filter_map(|k| fasce.get(*k))
        .flat_map(|f| f.eco_default.iter().cloned())
        .collect();
    let divergenti_default: Vec<f64> = fasce
        .get("fascia3_divergenza")
        .map(|f| f.eco_default.iter().cloned().collect())
        .unwrap_or_default();

    let affini_cal: Vec<f64> = ["fascia1_parafrasi", "fascia2_trasformazioni"]
        .iter()
        .filter_map(|k| fasce.get(*k))
        .flat_map(|f| f.eco_calibrata.iter().cloned())
        .collect();
    let divergenti_cal: Vec<f64> = fasce
        .get("fascia3_divergenza")
        .map(|f| f.eco_calibrata.iter().cloned().collect())
        .unwrap_or_default();

    println!("\n=== Livello 3: Sweeping θ (passo 0.01) ===");
    println!(
        "{:>5} | {:>10} {:>10} | {:>10} {:>10}", "θ", "FNR_def", "TNR_def", "FNR_cal", "TNR_cal"
    );

    let mut theta = 0.0f64;
    while theta <= 1.0 {
        let fnr_default = false_negative_rate(&affini_default, theta);
        let tnr_default = true_negative_rate(&divergenti_default, theta);
        let fnr_cal = false_negative_rate(&affini_cal, theta);
        let tnr_cal = true_negative_rate(&divergenti_cal, theta);
        println!(
            "{:>5.2} | {:>10.4} {:>10.4} | {:>10.4} {:>10.4}",
            theta, fnr_default, tnr_default, fnr_cal, tnr_cal
        );
        theta += 0.01;
    }

    Ok(())
}

/// Frazione di coppie affini (fascia 1+2) bloccate dal gate (s_eco < θ).
fn false_negative_rate(scores: &[f64], theta: f64) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let blocked = scores.iter().filter(|&&s| s < theta).count();
    blocked as f64 / scores.len() as f64
}

/// Frazione di coppie divergenti (fascia 3) bloccate dal gate (s_eco < θ).
fn true_negative_rate(scores: &[f64], theta: f64) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let blocked = scores.iter().filter(|&&s| s < theta).count();
    blocked as f64 / scores.len() as f64
}
