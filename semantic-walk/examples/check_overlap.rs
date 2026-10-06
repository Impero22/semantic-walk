// Diagnostica: misura la distribuzione di global_overlap sulle coppie del Dataset A
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_file = File::open(data_dir().join("dataset_a_index.json"))?;
    let index: DatasetIndex = serde_json::from_reader(BufReader::new(index_file))?;
    let npz_file = File::open(data_dir().join("dataset_a_cache.npz"))?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    let mut per_cat: HashMap<String, Vec<u32>> = HashMap::new();
    let mut all: Vec<u32> = Vec::new();
    for (pair_id, entry) in &index {
        let ids_a: Vec<i32> = read_npz_array(&mut zip, &format!("{}_a_ids", pair_id))?;
        let wa: Vec<f32> = read_npz_array(&mut zip, &format!("{}_a_weights", pair_id))?;
        let ids_b: Vec<i32> = read_npz_array(&mut zip, &format!("{}_b_ids", pair_id))?;
        let wb: Vec<f32> = read_npz_array(&mut zip, &format!("{}_b_weights", pair_id))?;
        let sa = build_sparse(&ids_a, &wa)?;
        let sb = build_sparse(&ids_b, &wb)?;
        let ov = sa.global_overlap(&sb);
        per_cat.entry(entry.category.clone()).or_default().push(ov);
        all.push(ov);
    }
    // report
    let mut cats: Vec<&String> = per_cat.keys().collect();
    cats.sort();
    for cat in &cats {
        let v = per_cat.get(*cat).unwrap();
        let min = v.iter().min().unwrap();
        let max = v.iter().max().unwrap();
        let mean = v.iter().map(|&x| x as f64).sum::<f64>() / v.len() as f64;
        println!("[{}] n={} min={} max={} mean={:.2}", cat, v.len(), min, max, mean);
    }
    let amin = all.iter().min().unwrap();
    let amax = all.iter().max().unwrap();
    println!("[TOTALE] n={} min={} max={}", all.len(), amin, amax);
    Ok(())
}
