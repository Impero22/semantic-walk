// Riproduce il finding del revisore: lunghezze diseguali possono causare panic
use semantic_walk::dtw::KinematicAligner;

fn main() {
    // Caso 1: lunghezze molto diseguali, window_size piccolo
    println!("=== Caso 1: n=10, m=2, window=1 ===");
    let aligner = KinematicAligner::new(1);
    let a: Vec<[f64; 2]> = (0..10).map(|i| [i as f64, (i as f64).sin()]).collect();
    let b: Vec<[f64; 2]> = vec![[0.0, 0.0], [9.0, 0.0]];
    let r = std::panic::catch_unwind(|| aligner.align(&a, &b));
    match r {
        Ok(Ok(al)) => println!("  OK: path_len={}, end={:?}", al.warp_path.len(), al.warp_path.last()),
        Ok(Err(e)) => println!("  ERR: {}", e),
        Err(_) => println!("  PANIC!")
    }

    // Caso 2: n=1, m=10, window=1
    println!("=== Caso 2: n=1, m=10, window=1 ===");
    let a2: Vec<[f64; 2]> = vec![[0.0, 0.0]];
    let b2: Vec<[f64; 2]> = (0..10).map(|i| [i as f64, (i as f64).cos()]).collect();
    let r2 = std::panic::catch_unwind(|| aligner.align(&a2, &b2));
    match r2 {
        Ok(Ok(al)) => println!("  OK: path_len={}, end={:?}", al.warp_path.len(), al.warp_path.last()),
        Ok(Err(e)) => println!("  ERR: {}", e),
        Err(_) => println!("  PANIC!")
    }

    // Caso 3: differenza di lunghezza > window_size
    println!("=== Caso 3: n=20, m=2, window=3 (diff 18 > window) ===");
    let aligner3 = KinematicAligner::new(3);
    let a3: Vec<[f64; 2]> = (0..20).map(|i| [i as f64, (i as f64 * 0.1).sin()]).collect();
    let b3: Vec<[f64; 2]> = vec![[0.0, 0.0], [19.0, 0.0]];
    let r3 = std::panic::catch_unwind(|| aligner3.align(&a3, &b3));
    match r3 {
        Ok(Ok(al)) => {
            println!("  OK: normalized={}, path_len={}, end={:?}", 
                al.normalized_score, al.warp_path.len(), al.warp_path.last());
            // Verifica che il percorso raggiunga l'angolo (n-1, m-1)
            let expected = (a3.len()-1, b3.len()-1);
            println!("  end atteso: {:?}, ottenuto: {:?}", expected, al.warp_path.last());
        }
        Ok(Err(e)) => println!("  ERR: {}", e),
        Err(_) => println!("  PANIC!")
    }
}
