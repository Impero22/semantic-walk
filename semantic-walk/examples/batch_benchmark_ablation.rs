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
const W_MIN: usize = 1;
const W_MAX: usize = 10;

// Livelli di ablazione (soglie di sovrapposizione sparse progressive).
//   L1 = denso puro (MaxSim bidirezionale mediato) — baseline, nessuna guida
//   L2 = DTW guidato ordered-sparse, MIN_OVERLAP = 0
//   L3 = DTW guidato ordered-sparse, MIN_OVERLAP = 2
//   L4 = DTW guidato ordered-sparse, MIN_OVERLAP = 4
//   L5 = DTW guidato ordered-sparse, MIN_OVERLAP = 5
// La progressione da 0 a 5 a passi di 2 (per L2-L4) mostra l'impatto
// progressivo del vincolo ordered-sparse rispetto al baseline denso L1.
const MIN_OVERLAP_L2: u32 = 0;
const MIN_OVERLAP_L3: u32 = 2;
const MIN_OVERLAP_L4: u32 = 4;
const MIN_OVERLAP_L5: u32 = 5;

// Livelli normalizzati (L6-L9): soglie Jaccard sul Bloom in [0.0, 1.0].
// Calibrate sulla distribuzione empirica del Dataset A (check_overlap.rs):
//   causality   J ∈ [0.794, 1.000] med 0.889
//   negation    J ∈ [0.709, 0.970] med 0.845
//   role        J ∈ [0.674, 1.000] med 0.837
//   synonymy    J ∈ [0.342, 0.658] med 0.531   ← il gruppo divergente
// La soglia discriminante naturale è J ≈ 0.65: sotto, coppie divergenti.
//   L6 = 0.00 (nessun pruning — baseline normalizzata)
//   L7 = 0.50 (pruning permissivo: taglia solo le coppie più divergenti)
//   L8 = 0.65 (soglia discriminante naturale)
//   L9 = 0.80 (pruning stretto: tiene solo coppie molto simili)
const MIN_JACCARD_L6: f32 = 0.00;
const MIN_JACCARD_L7: f32 = 0.50;
const MIN_JACCARD_L8: f32 = 0.65;
const MIN_JACCARD_L9: f32 = 0.80;

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

struct PairData {
    pair_id: String,
    category: String,
    seq_a: Vec<Vec<f64>>,
    seq_b: Vec<Vec<f64>>,
    sparse_a: OrderedSparseSequence,
    sparse_b: OrderedSparseSequence,
}

// ---------------------------------------------------------------------------
// Caricamento NPZ
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

/// Invariante di biiezione posizionale: un frame per posizione, senza
/// filtrare i soppressi (weight == 0.0).
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

    println!("[ABLATION] Indice:  {}", index_path.display());
    println!("[ABLATION] Cache:   {}", cache_path.display());

    let index_file = File::open(&index_path)?;
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader)?;
    println!("[ABLATION] Coppie totali: {}", index.len());

    let npz_file = File::open(&cache_path)?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    let aligner = KinematicAligner::new(W_MAX);

    // Per ogni categoria, per ogni livello: vettore di punteggi.
    // livelli: 1 (denso puro), 2..=5 (DTW guidato con soglia crescente)
    let mut scores: HashMap<String, HashMap<u32, Vec<f64>>> = HashMap::new();
    let mut pairs_processed = 0usize;
    let mut errors = 0usize;

    for (pair_id, entry) in &index {
        let pair = match load_pair(&mut zip, pair_id, &entry.category) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[ABLATION] Errore caricamento {}: {}", pair_id, e);
                errors += 1;
                continue;
            }
        };

        // L1: denso puro
        let l1 = compute_l1_maxsim(&pair.seq_a, &pair.seq_b);
        scores
            .entry(pair.category.clone())
            .or_default()
            .entry(1)
            .or_default()
            .push(l1);

        // L2..L5: DTW guidato con soglia progressiva (bit assoluti)
        for (level, min_overlap) in [
            (2u32, MIN_OVERLAP_L2),
            (3u32, MIN_OVERLAP_L3),
            (4u32, MIN_OVERLAP_L4),
            (5u32, MIN_OVERLAP_L5),
        ] {
            let score = match aligner.align_with_ordered_sparse(
                &pair.seq_a,
                &pair.seq_b,
                &pair.sparse_a,
                &pair.sparse_b,
                min_overlap,
                W_MIN,
                W_MAX,
            ) {
                Ok(Some(align)) => align.normalized_score,
                Ok(None) => f64::NAN, // ritiro geometrico (verdict)
                Err(e) => {
                    eprintln!(
                        "[ABLATION] Errore DTW L{} {} ({}): {}",
                        level, pair_id, pair.category, e
                    );
                    errors += 1;
                    continue;
                }
            };
            scores
                .entry(pair.category.clone())
                .or_default()
                .entry(level)
                .or_default()
                .push(score);
        }

        // L6..L9: DTW guidato con soglia Jaccard normalizzata
        for (level, min_jaccard) in [
            (6u32, MIN_JACCARD_L6),
            (7u32, MIN_JACCARD_L7),
            (8u32, MIN_JACCARD_L8),
            (9u32, MIN_JACCARD_L9),
        ] {
            let score = match aligner.align_with_ordered_sparse_norm(
                &pair.seq_a,
                &pair.seq_b,
                &pair.sparse_a,
                &pair.sparse_b,
                min_jaccard,
                W_MIN,
                W_MAX,
            ) {
                Ok(Some(align)) => align.normalized_score,
                Ok(None) => f64::NAN, // ritiro geometrico (verdict)
                Err(e) => {
                    eprintln!(
                        "[ABLATION] Errore DTW L{} {} ({}): {}",
                        level, pair_id, pair.category, e
                    );
                    errors += 1;
                    continue;
                }
            };
            scores
                .entry(pair.category.clone())
                .or_default()
                .entry(level)
                .or_default()
                .push(score);
        }

        pairs_processed += 1;
        if pairs_processed % 40 == 0 {
            println!("[ABLATION] Processate {} coppie", pairs_processed);
        }
    }

    println!("\n[ABLATION] Coppie processate: {}, errori: {}", pairs_processed, errors);

    // Report: punteggi medi per categoria × livello
    let mut cats: Vec<&String> = scores.keys().collect();
    cats.sort();
    println!("\n=== Punteggi medi per categoria × livello ===");
    for cat in &cats {
        let by_level = scores.get(*cat).unwrap();
        let mut levels: Vec<&u32> = by_level.keys().collect();
        levels.sort();
        for level in levels {
            let vals = by_level.get(level).unwrap();
            let ok: Vec<&f64> = vals.iter().filter(|&&x| !x.is_nan()).collect();
            let mean = if ok.is_empty() {
                f64::NAN
            } else {
                ok.iter().map(|&&x| x).sum::<f64>() / ok.len() as f64
            };
            println!(
                "[{}] L{} mean = {:.4} (n={}/{})",
                cat,
                level,
                mean,
                ok.len(),
                vals.len()
            );
        }
    }

    // AUC per categoria × livello (categoria vs tutte le altre)
    println!("\n=== AUC-ROC per categoria × livello ===");
    for cat in &cats {
        let by_level = scores.get(*cat).unwrap();
        let mut levels: Vec<&u32> = by_level.keys().collect();
        levels.sort();
        for level in levels {
            let pos: Vec<f64> = by_level
                .get(level)
                .unwrap()
                .iter()
                .filter(|&&x| !x.is_nan())
                .cloned()
                .collect();
            let mut neg = Vec::new();
            for (k, v) in &scores {
                if k != *cat {
                    neg.extend(
                        v.get(level)
                            .unwrap()
                            .iter()
                            .filter(|&&x| !x.is_nan())
                            .cloned(),
                    );
                }
            }
            let auc = compute_auc(&pos, &neg);
            println!("[{}] L{} AUC = {:.4}", cat, level, auc);
        }
    }

    // AUC binaria tra coppie di categorie × livello
    println!("\n=== AUC binaria tra coppie di categorie × livello ===");
    let cat_pairs = [
        ("role_reversal", "synonymy_control"),
        ("role_reversal", "negation_flip"),
        ("synonymy_control", "negation_flip"),
        ("causality", "synonymy_control"),
        ("causality", "negation_flip"),
        ("causality", "role_reversal"),
    ];
    for (a, b) in cat_pairs {
        let by_a = scores.get(a).unwrap();
        let by_b = scores.get(b).unwrap();
        let mut levels: Vec<&u32> = by_a.keys().collect();
        levels.sort();
        for level in levels {
            let sa: Vec<f64> = by_a
                .get(level)
                .unwrap()
                .iter()
                .filter(|&&x| !x.is_nan())
                .cloned()
                .collect();
            let sb: Vec<f64> = by_b
                .get(level)
                .unwrap()
                .iter()
                .filter(|&&x| !x.is_nan())
                .cloned()
                .collect();
            let auc_ab = compute_auc(&sa, &sb);
            let auc_ba = compute_auc(&sb, &sa);
            let best = auc_ab.max(auc_ba);
            println!("[{} vs {}] L{} AUC = {:.4}", a, b, level, best);
        }
    }

    Ok(())
}
