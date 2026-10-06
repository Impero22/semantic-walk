// Diagnostica: misura la distribuzione di global_overlap (bit assoluti) e
// global_jaccard (frazione normalizzata) sulle coppie del Dataset A.
// Scopo: calibrare la griglia L2-L5 su scala relativa J_bloom in [0.0, 1.0].
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;
use semantic_walk::ordered_sparse::OrderedSparseSequence;

#[derive(Deserialize, Debug)]
struct IndexEntry { category: String, text_a: String, text_b: String }
type DatasetIndex = HashMap<String, IndexEntry>;

fn data_dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_a") }
fn read_npz_array<T: npyz::Deserialize>(zip: &mut ZipArchive<BufReader<File>>, name: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    Ok(reader.into_vec::<T>()?)
}
fn build_sparse(ids: &[i32], weights: &[f32]) -> Result<OrderedSparseSequence, &'static str> {
    if ids.len() != weights.len() { return Err("len mismatch"); }
    let frames: Vec<Vec<(u32, f32)>> = ids.iter().zip(weights.iter())
        .map(|(&id, &w)| vec![(id as u32, w)]).collect();
    OrderedSparseSequence::from_frames(&frames)
}

fn quartili(v: &[f32]) -> (f32, f32, f32, f32, f32) {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = s.len();
    let q = |p: f64| s[((n as f64 - 1.0) * p) as usize];
    (q(0.0), q(0.25), q(0.5), q(0.75), q(1.0))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_file = File::open(data_dir().join("dataset_a_index.json"))?;
    let index: DatasetIndex = serde_json::from_reader(BufReader::new(index_file))?;
    let npz_file = File::open(data_dir().join("dataset_a_cache.npz"))?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    let mut per_cat: HashMap<String, Vec<f32>> = HashMap::new();
    let mut all_j: Vec<f32> = Vec::new();
    let mut all_ov: Vec<u32> = Vec::new();
    for (pair_id, entry) in &index {
        let ids_a: Vec<i32> = read_npz_array(&mut zip, &format!("{}_a_ids", pair_id))?;
        let wa: Vec<f32> = read_npz_array(&mut zip, &format!("{}_a_weights", pair_id))?;
        let ids_b: Vec<i32> = read_npz_array(&mut zip, &format!("{}_b_ids", pair_id))?;
        let wb: Vec<f32> = read_npz_array(&mut zip, &format!("{}_b_weights", pair_id))?;
        let sa = build_sparse(&ids_a, &wa)?;
        let sb = build_sparse(&ids_b, &wb)?;
        let j = sa.global_jaccard(&sb);
        let ov = sa.global_overlap(&sb);
        per_cat.entry(entry.category.clone()).or_default().push(j);
        all_j.push(j);
        all_ov.push(ov);
    }
    // report Jaccard normalizzato
    println!("=== Jaccard normalizzato (J_bloom, [0.0, 1.0]) ===");
    let mut cats: Vec<&String> = per_cat.keys().collect();
    cats.sort();
    for cat in &cats {
        let v = per_cat.get(*cat).unwrap();
        let (min, q1, med, q3, max) = quartili(v);
        println!("[{}] n={} min={:.3} q1={:.3} med={:.3} q3={:.3} max={:.3}",
            cat, v.len(), min, q1, med, q3, max);
    }
    let (amin, aq1, amed, aq3, amax) = quartili(&all_j);
    println!("[TOTALE] n={} min={:.3} q1={:.3} med={:.3} q3={:.3} max={:.3}",
        all_j.len(), amin, aq1, amed, aq3, amax);

    // report overlap assoluto (diagnostico, scala 0..=128)
    println!("\n=== Overlap assoluto (bit Bloom, 0..=128) ===");
    let omin = all_ov.iter().min().unwrap();
    let omax = all_ov.iter().max().unwrap();
    let omean = all_ov.iter().map(|&x| x as f64).sum::<f64>() / all_ov.len() as f64;
    println!("[TOTALE] n={} min={} max={} mean={:.2}", all_ov.len(), omin, omax, omean);
    Ok(())
}
