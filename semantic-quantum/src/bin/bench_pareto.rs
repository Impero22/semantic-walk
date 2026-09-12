//! Benchmark di latenza: pruning Pareto + collapse quantistico.
//!
//! Misura l'abbattimento del carico di calcolo garantito dal pruning
//! `estrai_frontiera_pareto` prima del `QuantumResolver::collapse`.
//!
//! Tre scenari per ogni N:
//!   - Baseline   : N rami completi direttamente al collapse (senza pruning)
//!   - Con pruning: N rami -> frontiera di Pareto -> collapse sui superstiti
//!   - Scala      : N crescente (16, 64, 256, 1024)
//!
//! Il benchmark è strutturato come binario, non come test criterion, per non
//! introdurre dipendenze esterne nel crate.

use std::time::Instant;

use semantic_quantum::pareto::{estrai_frontiera_pareto, estrai_frontiera_pareto_adattivo, BranchCostVector, SOGLIA_ADATTIVA_DEFAULT, VARIANZA_DIRECT_DEFAULT};
use semantic_quantum::{BranchBuilder, BranchType, QuantumResolver, WalkBranch};

/// Genera N rami con costi casuali ma deterministici (LGC).
///
/// `dominance` in [0,1] controlla la ridondanza: a 0 tutti i rami sono
/// incomparabili (caso peggiore per il pruning), a 1 tutti convergono verso
/// un unico punto (massima dominanza, massimo scarto).
fn gen_branches(n: usize, dominance: f32, seed: u64) -> Vec<BranchCostVector> {
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut next = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((state >> 33) as f32) / (u32::MAX as f32)
    };

    let builder = BranchBuilder::new((0.4, 0.3, 0.3));
    let mut out = Vec::with_capacity(n);

    // Strategia a catena di dominanza controllata.
    //
    // Il ramo 0 è il punto "ancora" con costi bassi. Ogni ramo successivo è
    // il ramo precedente + incrementi non negativi su *almeno un* asse,
    // con probabilità `dominance` di essere dominato dal precedente.
    //
    // A dominance=1: catena pura — ogni ramo domina tutti i successivi
    // (frontiera = 1, massimo scarto).
    // A dominance=0: incrementi casuali su assi casuali — per lo più
    // incomparabili (caso peggiore per il pruning).
    // A dominance=0.5: metà dei rami è dominata dal precedente, metà no.
    let mut cur = (0.05f32, 0.05f32, 0.05f32);
    for i in 0..n {
        let id = (i % 1000) as u64;
        let (s_i, s_g, s_c) = if i == 0 {
            cur
        } else if next() < dominance {
            // Dominato dal precedente: incremento su uno o più assi.
            let inc = next() * 0.2;
            let axis = (next() * 3.0) as usize;
            match axis {
                0 => { cur.0 += inc; }
                1 => { cur.1 += inc; }
                _ => { cur.2 += inc; }
            }
            cur
        } else {
            // Incomparabile: nuovo punto di partenza.
            cur = (0.05 + next() * 0.9, 0.05 + next() * 0.9, 0.05 + next() * 0.9);
            cur
        };
        let (s_i, s_g, s_c) = (s_i.min(1.0), s_g.min(1.0), s_c.min(1.0));
        let colbert_sim = (1.0 - s_c).max(0.0);
        let branch = builder.build_branch(BranchType::Inertial, id, s_i, s_g, colbert_sim, 2.0);
        out.push(BranchCostVector::new(branch, s_i, s_g, s_c));
    }
    out
}

fn main() {
    let resolver = QuantumResolver::new(0.5, 3, 2.0);
    let sizes = [16usize, 64, 128, 256, 1024];
    let dominances = [0.0f32, 0.5, 0.9];
    let iters = 1000;

    println!("{:<8} {:<12} {:<12} {:<14} {:<14} {:<14} {:<14} {:<10} {:<10} {:<10}", "N", "Dominanza", "Frontiera", "Baseline (us)", "Pruning (us)", "Sorted (us)", "Direct (us)", "Risparmio", "Scarto%", "Sorted%");

    for &n in &sizes {
        for &d in &dominances {
            let branches = gen_branches(n, d, 42);

            // Baseline: collapse diretto su N rami.
            let full: Vec<WalkBranch> = branches.iter().map(|c| c.branch.clone()).collect();
            let t0 = Instant::now();
            for _ in 0..iters {
                let _ = resolver.collapse(&full);
            }
            let baseline_us = t0.elapsed().as_micros() as f64 / iters as f64;

            // Con pruning: frontiera di Pareto -> collapse sui superstiti.
            let t1 = Instant::now();
            let mut front_size = 0usize;
            for _ in 0..iters {
                let front = estrai_frontiera_pareto(&branches);
                front_size = front.len();
                let pruned: Vec<WalkBranch> = front.into_iter().map(|c| c.branch).collect();
                let _ = resolver.collapse(&pruned);
            }
            let pruning_us = t1.elapsed().as_micros() as f64 / iters as f64;

            // Regime adattivo (AdaptiveGate a due vie: FullPareto/DirectCollapse).
            let t2 = Instant::now();
            for _ in 0..iters {
                let front = estrai_frontiera_pareto_adattivo(&branches, SOGLIA_ADATTIVA_DEFAULT, VARIANZA_DIRECT_DEFAULT);
                let pruned: Vec<WalkBranch> = front.into_iter().map(|c| c.branch).collect();
                let _ = resolver.collapse(&pruned);
            }
            let sorted_us = t2.elapsed().as_micros() as f64 / iters as f64;

            // Direct collapse (senza pruning) — il terzo regime dell'AdaptiveGate.
            let t3 = Instant::now();
            for _ in 0..iters {
                let _ = resolver.collapse(&full);
            }
            let direct_us = t3.elapsed().as_micros() as f64 / iters as f64;

            let risparmio = if baseline_us > 0.0 {
                (1.0 - pruning_us / baseline_us) * 100.0
            } else {
                0.0
            };
            let sorted_risparmio = if baseline_us > 0.0 {
                (1.0 - sorted_us / baseline_us) * 100.0
            } else {
                0.0
            };
            let scarto_pct = (1.0 - front_size as f64 / n as f64) * 100.0;

            println!(
                "{:<8} {:<12} {:<12} {:<14.2} {:<14.2} {:<14.2} {:<14.2} {:<10.1}% {:<10.1}% {:<10.1}%",
                n, d, front_size, baseline_us, pruning_us, sorted_us, direct_us, risparmio, scarto_pct, sorted_risparmio
            );
        }
    }
}