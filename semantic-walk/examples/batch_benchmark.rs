#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_walk::dtw::KinematicAligner;
use semantic_walk::ordered_sparse::OrderedSparseSequence;

// ---------------------------------------------------------------------------
// Configurazione
// ---------------------------------------------------------------------------

// Percorso derivato dal crate: portabile indipendentemente dall'utente.
fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_a")
}

fn index_path() -> PathBuf {
    data_dir().join("dataset_a_index.json")
}

fn cache_path() -> PathBuf {
    data_dir().join("dataset_a_cache.npz")
}

// Parametri del DTW (concordati con Camillo):
//   w_min = 1, w_max = 10  (banda Sakoe-Chiba dinamica)
//   min_overlap_threshold = 0 per L2, 5 per L5
const W_MIN: usize = 1;
const W_MAX: usize = 10;
const MIN_OVERLAP_L2: u32 = 0;
const MIN_OVERLAP_L5: u32 = 5;

// ---------------------------------------------------------------------------
// Strutture dati
// ---------------------------------------------------------------------------

/// Indice reale: dict { pair_id: { category, text_a, text_b } }
#[derive(Deserialize, Debug)]
struct IndexEntry {
    category: String,
    text_a: String,
    text_b: String,
}

type DatasetIndex = HashMap<String, IndexEntry>;

/// Una coppia caricata dal NPZ, pronta per il DTW.
struct PairData {
    pair_id: String,
    category: String,
    seq_a: Vec<Vec<f64>>, // traiettoria densa (n, 1024)
    seq_b: Vec<Vec<f64>>, // traiettoria densa (m, 1024)
    sparse_a: OrderedSparseSequence,
    sparse_b: OrderedSparseSequence,
}

// ---------------------------------------------------------------------------
// Caricamento NPZ
// ---------------------------------------------------------------------------

/// Legge un array dal NPZ e lo converte in Vec<T>.
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

/// Legge la traiettoria densa (shape [n, dim]) e la converte in Vec<Vec<f64>>.
fn read_dense(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    let shape = reader.shape().to_vec();
    let flat = reader.into_vec::<f32>()?;

    let dim = if shape.len() >= 2 {
        shape[1] as usize
    } else {
        1
    };
    let n = if flat.is_empty() {
        0
    } else {
        flat.len() / dim
    };

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

/// Costruisce una OrderedSparseSequence da ids/weights/status.
///
/// Invariante di biiezione posizionale: si passa UN frame per posizione,
/// senza filtrare i soppressi (weight == 0.0). Filtrare a monte romperebbe
/// la corrispondenza 1:1 con la traiettoria densa.
fn build_sparse(ids: &[i32], weights: &[f32]) -> Result<OrderedSparseSequence, &'static str> {
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

/// Carica una singola coppia dal NPZ.
fn load_pair(
    zip: &mut ZipArchive<BufReader<File>>,
    pair_id: &str,
    category: &str,
) -> Result<PairData, Box<dyn std::error::Error>> {
    let seq_a = read_dense(zip, &format!("{}_a_dense", pair_id))?;
    let seq_b = read_dense(zip, &format!("{}_b_dense", pair_id))?;

    let ids_a: Vec<i32> = read_npz_array(zip, &format!("{}_a_ids", pair_id))?;
    let weights_a: Vec<f32> = read_npz_array(zip, &format!("{}_a_weights", pair_id))?;
    let ids_b: Vec<i32> = read_npz_array(zip, &format!("{}_b_ids", pair_id))?;
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
// Baseline L1: ColBERT MaxSim bidirezionale mediato
// ---------------------------------------------------------------------------

fn compute_l1_maxsim(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
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

// ---------------------------------------------------------------------------
// AUC-ROC
// ---------------------------------------------------------------------------

fn compute_auc(pos_scores: &[f64], neg_scores: &[f64]) -> f64 {
    if pos_scores.is_empty() || neg_scores.is_empty() {
        return 0.5;
    }

    let mut all_scores: Vec<(f64, bool)> = Vec::new();
    for &s in pos_scores {
        all_scores.push((s, true));
    }
    for &s in neg_scores {
        all_scores.push((s, false));
    }

    all_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut num_pos = 0f64;
    let mut num_neg = 0f64;
    let mut rank_sum = 0f64;

    for (i, &(_score, is_pos)) in all_scores.iter().enumerate() {
        let rank = (all_scores.len() - i) as f64;
        if is_pos {
            num_pos += 1.0;
            rank_sum += rank;
        } else {
            num_neg += 1.0;
        }
    }

    if num_pos == 0.0 || num_neg == 0.0 {
        return 0.5;
    }

    (rank_sum - (num_pos * (num_pos + 1.0) / 2.0)) / (num_pos * num_neg)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = index_path();
    let cache_path = cache_path();

    println!("[BENCHMARK] Indice:  {}", index_path.display());
    println!("[BENCHMARK] Cache:   {}", cache_path.display());

    // Carica l'indice (dict { pair_id: {category, text_a, text_b} })
    let index_file = File::open(&index_path)?;
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader)?;
    println!("[BENCHMARK] Coppie totali: {}", index.len());

    // Apre il NPZ
    let npz_file = File::open(&cache_path)?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    let aligner = KinematicAligner::new(W_MAX);

    // Raccogliamo i punteggi per categoria (L1 e L5)
    // category -> (pos_scores, neg_scores) dove "pos" è la coppia stessa.
    // Per ora confrontiamo le coppie della stessa categoria tra loro: il
    // benchmark a coppie (dataset costruito) valuta la separabilità per
    // categoria tramite AUC su punteggi di similarità tra coppie della
    // stessa categoria vs coppie di categorie diverse.
    let mut l1_scores: HashMap<String, Vec<f64>> = HashMap::new();
    let mut l5_scores: HashMap<String, Vec<f64>> = HashMap::new();

    let mut pairs_processed = 0usize;
    let mut errors = 0usize;

    for (pair_id, entry) in &index {
        let pair = match load_pair(&mut zip, pair_id, &entry.category) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[BENCHMARK] Errore caricamento {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };

        // L1: MaxSim
        let l1 = compute_l1_maxsim(&pair.seq_a, &pair.seq_b);
        l1_scores
            .entry(pair.category.clone())
            .or_default()
            .push(l1);

        // L5: DTW con guida ordered-sparse
        let l5 = match aligner.align_with_ordered_sparse(
            &pair.seq_a,
            &pair.seq_b,
            &pair.sparse_a,
            &pair.sparse_b,
            MIN_OVERLAP_L5,
            W_MIN,
            W_MAX,
        ) {
            Ok(Some(align)) => align.normalized_score,
            Ok(None) => f64::NAN, // ritiro geometrico (verdict)
            Err(e) => {
                eprintln!(
                    "[BENCHMARK] Errore DTW {} ({}): {}",
                    pair_id, pair.category, e
                );
                errors += 1;
                continue;
            }
        };
        l5_scores
            .entry(pair.category.clone())
            .or_default()
            .push(l5);

        // DEBUG: coppie causality con inversione causale pura (stesse parole).
        // Verifichiamo se L5 separa come su role_reversal o se il connettivo
        // condiviso ("di conseguenza", "quindi", "perciò", "pertanto") fa
        // allineare il DTW sulle clausole senza penalizzare lo scambio causale.
        if pair.category == "causality" {
            let a_clean = entry.text_a.replace(',', "").replace('.', "");
            let b_clean = entry.text_b.replace(',', "").replace('.', "");
            let wa = a_clean.split_whitespace().collect::<Vec<_>>();
            let wb = b_clean.split_whitespace().collect::<Vec<_>>();
            let mut sa = wa.clone();
            let mut sb = wb.clone();
            sa.sort();
            sb.sort();
            if sa == sb {
                println!(
                    "[DEBUG] causality PURA {}: L1={:.4} L5={:.4}",
                    pair_id, l1, l5
                );
            }
        }
        // DEBUG: distribuzione L5 per le altre categorie (per confronto).
        if matches!(pair.category.as_str(), "synonymy_control" | "negation_flip" | "role_reversal") {
            println!(
                "[DEBUG] {} {}: L1={:.4} L5={:.4}",
                pair.category, pair_id, l1, l5
            );
        }

        pairs_processed += 1;
        if pairs_processed % 40 == 0 {
            println!("[BENCHMARK] Processate {} coppie", pairs_processed);
        }
    }

    println!("\n[BENCHMARK] Coppie processate: {}, errori: {}", pairs_processed, errors);

    // Report per categoria
    println!("\n=== Punteggi medi per categoria ===");
    let mut cats: Vec<&String> = l1_scores.keys().collect();
    cats.sort();
    for cat in &cats {
        let l1 = l1_scores.get(*cat).unwrap();
        let l5 = l5_scores.get(*cat).unwrap();
        let l1_mean = l1.iter().sum::<f64>() / l1.len() as f64;
        let l5_ok: Vec<&f64> = l5.iter().filter(|&&x| !x.is_nan()).collect();
        let l5_mean = if l5_ok.is_empty() {
            f64::NAN
        } else {
            l5_ok.iter().map(|&&x| x).sum::<f64>() / l5_ok.len() as f64
        };
        println!(
            "[{}] L1 mean = {:.4} (n={}) | L5 mean = {:.4} (n={})",
            cat,
            l1_mean,
            l1.len(),
            l5_mean,
            l5_ok.len()
        );
    }

    // AUC per categoria: pos = coppie della categoria, neg = tutte le altre
    println!("\n=== AUC-ROC per categoria (L1 vs L5) ===");
    for cat in &cats {
        let l1_pos = l1_scores.get(*cat).unwrap();
        let mut l1_neg = Vec::new();
        for (k, v) in &l1_scores {
            if k != *cat {
                l1_neg.extend(v.iter());
            }
        }
        let auc_l1 = compute_auc(l1_pos, &l1_neg);

        let l5_pos: Vec<f64> = l5_scores
            .get(*cat)
            .unwrap()
            .iter()
            .filter(|&&x| !x.is_nan())
            .cloned()
            .collect();
        let mut l5_neg = Vec::new();
        for (k, v) in &l5_scores {
            if k != *cat {
                l5_neg.extend(v.iter().filter(|&&x| !x.is_nan()));
            }
        }
        let auc_l5 = compute_auc(&l5_pos, &l5_neg);

        println!(
            "[{}] L1 AUC = {:.4} | L5 AUC = {:.4}",
            cat, auc_l1, auc_l5
        );
    }

    // Confronti binari tra coppie di categorie specifiche.
    // L'AUC "una categoria vs tutte le altre" è distorto per categorie con
    // similarità intrinsecamente diversa (role_reversal ha L1 media bassa
    // perché inverte il ruolo, quindi risulta sistematicamente "meno simile"
    // di qualunque altra). Il confronto binario tra categorie specifiche è
    // il criterio corretto: misura la capacità di SEPARARE due trasformazioni.
    println!("\n=== AUC binaria tra coppie di categorie (L1 vs L5) ===");
    let cat_pairs = [
        ("role_reversal", "synonymy_control"),
        ("role_reversal", "negation_flip"),
        ("synonymy_control", "negation_flip"),
        ("causality", "synonymy_control"),
        ("causality", "negation_flip"),
        ("causality", "role_reversal"),
    ];
    for (a, b) in cat_pairs {
        let l1_a = l1_scores.get(a).unwrap();
        let l1_b = l1_scores.get(b).unwrap();
        let auc_l1_ab = compute_auc(l1_a, l1_b);
        let auc_l1_ba = compute_auc(l1_b, l1_a);

        let l5_a: Vec<f64> = l5_scores
            .get(a)
            .unwrap()
            .iter()
            .filter(|&&x| !x.is_nan())
            .cloned()
            .collect();
        let l5_b: Vec<f64> = l5_scores
            .get(b)
            .unwrap()
            .iter()
            .filter(|&&x| !x.is_nan())
            .cloned()
            .collect();
        let auc_l5_ab = compute_auc(&l5_a, &l5_b);
        let auc_l5_ba = compute_auc(&l5_b, &l5_a);

        // Prendiamo il max: l'AUC binaria è simmetrica rispetto a quale
        // categoria è "pos" (AUC_ab + AUC_ba = 1.0).
        let best_l1 = auc_l1_ab.max(auc_l1_ba);
        let best_l5 = auc_l5_ab.max(auc_l5_ba);
        println!(
            "[{} vs {}] L1 AUC = {:.4} | L5 AUC = {:.4}",
            a, b, best_l1, best_l5
        );
    }

    Ok(())
}
