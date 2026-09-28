use semantic_walk::dtw::KinematicAligner;
use semantic_walk::ordered_sparse::OrderedSparseSequence;
use std::fs::File;
use std::io::BufReader;
use npyz::NpyFile;
use zip::ZipArchive;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct IndexEntry {
    category: String,
    text_a: String,
    text_b: String,
}

fn read_dense(zip: &mut ZipArchive<BufReader<File>>, name: &str) -> Vec<Vec<f64>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname).unwrap();
    let reader = NpyFile::new(file).unwrap();
    let shape = reader.shape().to_vec();
    let flat = reader.into_vec::<f32>().unwrap();
    let dim = if shape.len() >= 2 { shape[1] as usize } else { 1 };
    let n = if flat.is_empty() { 0 } else { flat.len() / dim };
    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        let row = flat[i*dim..(i+1)*dim].iter().map(|&x| x as f64).collect::<Vec<f64>>();
        rows.push(row);
    }
    rows
}

fn read_ids(zip: &mut ZipArchive<BufReader<File>>, name: &str) -> Vec<i32> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname).unwrap();
    let reader = NpyFile::new(file).unwrap();
    reader.into_vec::<i32>().unwrap()
}

fn read_weights(zip: &mut ZipArchive<BufReader<File>>, name: &str) -> Vec<f32> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname).unwrap();
    let reader = NpyFile::new(file).unwrap();
    reader.into_vec::<f32>().unwrap()
}

fn build_sparse(ids: &[i32], weights: &[f32]) -> OrderedSparseSequence {
    let frames: Vec<Vec<(u32, f32)>> = ids.iter().zip(weights.iter())
        .map(|(&id, &w)| vec![(id as u32, w)]).collect();
    OrderedSparseSequence::from_frames(&frames).unwrap()
}

fn main() {
    let data_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_a");
    let idx_file = File::open(data_dir.join("dataset_a_index.json")).unwrap();
    let idx: HashMap<String, IndexEntry> = serde_json::from_reader(BufReader::new(idx_file)).unwrap();
    let npz_file = File::open(data_dir.join("dataset_a_cache.npz")).unwrap();
    let mut zip = ZipArchive::new(BufReader::new(npz_file)).unwrap();

    // Coppia causality pura reale
    for pid in ["pair_217", "pair_145"] {
        let e = &idx[pid];
        let seq_a = read_dense(&mut zip, &format!("{}_a_dense", pid));
        let seq_b = read_dense(&mut zip, &format!("{}_b_dense", pid));
        let ids_a = read_ids(&mut zip, &format!("{}_a_ids", pid));
        let w_a = read_weights(&mut zip, &format!("{}_a_weights", pid));
        let ids_b = read_ids(&mut zip, &format!("{}_b_ids", pid));
        let w_b = read_weights(&mut zip, &format!("{}_b_weights", pid));
        let sa = build_sparse(&ids_a, &w_a);
        let sb = build_sparse(&ids_b, &w_b);

        let aligner = KinematicAligner::new(10);
        let r_ab = aligner.align_with_ordered_sparse(&seq_a, &seq_b, &sa, &sb, 5, 1, 10);
        let r_ba = aligner.align_with_ordered_sparse(&seq_b, &seq_a, &sb, &sa, 5, 1, 10);
        println!("{} | A: '{}'", pid, e.text_a);
        println!("   B: '{}'", e.text_b);
        println!("   align(A,B) = {:?}", r_ab.map(|o| o.map(|a| a.normalized_score)));
        println!("   align(B,A) = {:?}", r_ba.map(|o| o.map(|a| a.normalized_score)));
    }
}
