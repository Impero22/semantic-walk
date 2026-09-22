// Verifica: la banda troppo stretta produce normalized=inf
use semantic_walk::dtw::KinematicAligner;

fn main() {
    // Caso: diff lunghezza > window_size → cella finale mai raggiunta
    let aligner = KinematicAligner::new(1);
    let a: Vec<[f64; 2]> = (0..10).map(|i| [i as f64, 0.0]).collect();
    let b: Vec<[f64; 2]> = vec![[0.0, 0.0], [9.0, 0.0]];
    let al = aligner.align(&a, &b).unwrap();
    println!("n=10, m=2, window=1 → normalized={}, divergence={}, path_len={}",
        al.normalized_score, al.divergence_token, al.warp_path.len());
    println!("  Il revisore ha ragione: il risultato è 'riuscito' ma il costo è INFINITY");
    println!("  (la cella finale non è mai stata raggiunta dalla finestra)");
}
