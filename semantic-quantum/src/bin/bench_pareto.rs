//! Benchmark: collapse quantistico + selezione Pareto candidate-level.
//!
//! Dopo il principio dell'Isomorfismo di Livello (Camillo, 22/09/26), il
//! pruning branch-level PRIMA del collapse è vietato: ogni ramo contribuisce
//! all'ampiezza Ψ(c) = Σ exp(-S_r/κ) del proprio candidato, e scartare un ramo
//! "dominato" mutila la funzione d'onda prima dell'integrazione.
//!
//! Il Pareto ha un ruolo legittimo SOLO DOPO il collapse, sui candidati
//! collassati: seleziona quali candidati mantenere per le fasi successive,
//! senza alterare l'esito del collapse (che è già avvenuto).
//!
//! Questo benchmark misura quindi il flusso corretto:
//!   - Baseline : collapse completo su N rami (tutti contribuiscono).
//!   - Candidate: collapse completo + aggregazione costi per candidato +
//!                Pareto candidate-level sui candidati collassati.
//!
//! La metrica onesta NON è "quanto risparmio sul collapse" (il collapse va
//! comunque fatto su tutti i rami) ma quanto costa la selezione candidate-level
//! rispetto al solo collapse — e quanti candidati dominati scarta.
//!
//! Tre scenari per ogni N (16, 64, 128, 256, 1024) e dominanza (0.0, 0.5, 0.9).

use std::time::Instant;

use semantic_quantum::pareto::{
    estrai_frontiera_pareto_sui_candidati, CandidateCostVector,
};
use semantic_quantum::{BranchBuilder, BranchType, QuantumResolver, WalkBranch};

/// Genera N rami con costi casuali ma deterministici (LCG a 53 bit, finding #4).
///
/// `dominance` in [0,1] controlla la frazione di candidati **dominati**: genera
/// M = n / rami_per_candidato centri candidato sparsi in [0,1]³, poi una
/// frazione `dominance` di essi viene spostata verso l'alto (peggiore) su tutti
/// e tre gli assi — così c'è dominanza reale tra candidati, e il Pareto
/// candidate-level ha davvero qualcosa da filtrare.
///
/// Ogni candidato genera `rami_per_candidato` rami attorno al proprio centro
/// (interferenza costruttiva reale nel collapse). Il numero di candidati
/// M = ceil(n / rami_per_candidato).
fn gen_branches(n: usize, dominance: f32, seed: u64, rami_per_candidato: usize) -> Vec<WalkBranch> {
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut next = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (((state >> 11) as f64) / ((1u64 << 53) as f64)) as f32
    };

    let builder = BranchBuilder::new((0.4, 0.3, 0.3));
    let mut out = Vec::with_capacity(n);
    let rpc = rami_per_candidato.max(1);
    let m = n / rpc; // numero di candidati

    for c in 0..m {
        // Centro del candidato: punto casuale in [0.05, 0.95]³.
        let mut center = (
            0.05 + next() * 0.9,
            0.05 + next() * 0.9,
            0.05 + next() * 0.9,
        );
        // Se `dominance` lo decide, il candidato è DOMINATO: tutti i suoi costi
        // vengono alzati di un offset su tutti gli assi. Così un candidato
        // dominato è strettamente peggiore di un altro (dominanza reale).
        if next() < dominance {
            let offset = 0.15 + next() * 0.3;
            center.0 = (center.0 + offset).min(1.0);
            center.1 = (center.1 + offset).min(1.0);
            center.2 = (center.2 + offset).min(1.0);
        }
        // I `rpc` rami del candidato: piccola perturbazione attorno al centro.
        for _ in 0..rpc {
            let perturb = 0.02 * next();
            let (s_i, s_g, s_c) = (
                (center.0 + perturb).min(1.0),
                (center.1 + perturb).min(1.0),
                (center.2 + perturb).min(1.0),
            );
            let colbert_sim = (1.0 - s_c).max(0.0);
            let branch = builder.build_branch(BranchType::Inertial, c as u64, s_i, s_g, colbert_sim, 2.0);
            out.push(branch);
        }
    }
    out
}

/// Aggrega i costi dei rami per candidato (media), producendo i vettori
/// candidate-level su cui opera il Pareto post-collapse.
fn aggrega_per_candidato(branches: &[WalkBranch]) -> Vec<CandidateCostVector> {
    use std::collections::HashMap;
    let mut sums: HashMap<u64, (f32, f32, f32, usize)> = HashMap::new();
    for b in branches {
        let Some(cid) = b.candidate_id else { continue };
        // Usa i tre assi grezzi di costo ora conservati sul WalkBranch
        // (s_inertial, s_geometric, s_colbert), non più i proxy a 1 asse.
        let entry = sums.entry(cid).or_insert((0.0, 0.0, 0.0, 0));
        entry.0 += b.s_inertial; // costo inerziale
        entry.1 += b.s_geometric; // costo geometrico
        entry.2 += b.s_colbert; // costo colbert
        entry.3 += 1;
    }
    sums.into_iter()
        .map(|(id, (si, sg, sc, n))| {
            let n = n.max(1) as f32;
            CandidateCostVector::new(id, si / n, sg / n, sc / n)
        })
        .collect()
}

fn main() {
    let resolver = QuantumResolver::new(0.5, 3, 2.0);
    let sizes = [16usize, 64, 128, 256, 1024];
    let dominances = [0.0f32, 0.5, 0.9];
    // Rami per candidato: 1 = baseline (un solo ramo per candidato, il Pareto
    // candidate-level non ha nulla da filtrare); 8 = interferenza costruttiva
    // reale, ogni candidato ha 8 rami che contribuiscono al suo Ψ(c).
    let rami_per_candidato_opts = [1usize, 8];
    let iters = 1000;

    println!(
        "{:<6} {:<6} {:<10} {:<10} {:<16} {:<16} {:<12} {:<10}",
        "N", "R/C", "Dominanza", "Candidati", "Collapse (us)", "Coll+Pareto (us)", "Scarto%", "Overhead%"
    );

    for &n in &sizes {
        for &rpc in &rami_per_candidato_opts {
            for &d in &dominances {
                let branches = gen_branches(n, d, 42, rpc);

                // Baseline: collapse completo su tutti i rami.
                let t0 = Instant::now();
                for _ in 0..iters {
                    let _ = resolver.collapse(&branches);
                }
                let baseline_us = t0.elapsed().as_micros() as f64 / iters as f64;

                // Candidate-level: collapse + aggregazione + Pareto post-collapse.
                let t1 = Instant::now();
                let mut candidati_totali = 0usize;
                let mut front_size = 0usize;
                for _ in 0..iters {
                    let _ = resolver.collapse(&branches);
                    let candidati = aggrega_per_candidato(&branches);
                    candidati_totali = candidati.len();
                    let front = estrai_frontiera_pareto_sui_candidati(&candidati);
                    front_size = front.len();
                }
                let candidate_us = t1.elapsed().as_micros() as f64 / iters as f64;

                let scarto_pct = if candidati_totali > 0 {
                    (1.0 - front_size as f64 / candidati_totali as f64) * 100.0
                } else {
                    0.0
                };
                let overhead_pct = if baseline_us > 0.0 {
                    (candidate_us / baseline_us - 1.0) * 100.0
                } else {
                    0.0
                };

                println!(
                    "{:<6} {:<6} {:<10} {:<10} {:<16.2} {:<16.2} {:<12.1} {:<10.1}",
                    n, rpc, d, candidati_totali, baseline_us, candidate_us, scarto_pct, overhead_pct
                );
            }
        }
    }
}