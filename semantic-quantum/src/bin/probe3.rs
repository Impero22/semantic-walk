// Probe 3: verifica che la dominanza dei centri si conservi dopo la
// generazione dei rami e l'aggregazione per candidato.
use semantic_quantum::pareto::{estrai_frontiera_pareto_sui_candidati, CandidateCostVector};
use semantic_quantum::{BranchBuilder, BranchType, WalkBranch};
use std::collections::HashMap;

fn gen_branches(n: usize, dominance: f32, seed: u64, rpc: usize) -> Vec<WalkBranch> {
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut next = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (((state >> 11) as f64) / ((1u64 << 53) as f64)) as f32
    };
    let builder = BranchBuilder::new((0.4, 0.3, 0.3));
    let mut out = Vec::with_capacity(n);
    let rpc = rpc.max(1);
    let m = n / rpc;
    for c in 0..m {
        let mut center = (0.05 + next()*0.9, 0.05 + next()*0.9, 0.05 + next()*0.9);
        if next() < dominance {
            let offset = 0.15 + next()*0.3;
            center.0 = (center.0 + offset).min(1.0);
            center.1 = (center.1 + offset).min(1.0);
            center.2 = (center.2 + offset).min(1.0);
        }
        for _ in 0..rpc {
            let perturb = 0.02 * next();
            let (s_i, s_g, s_c) = ((center.0+perturb).min(1.0), (center.1+perturb).min(1.0), (center.2+perturb).min(1.0));
            out.push(builder.build_branch(BranchType::Inertial, c as u64, s_i, s_g, (1.0-s_c).max(0.0), 2.0));
        }
    }
    out
}

fn aggrega(branches: &[WalkBranch]) -> Vec<CandidateCostVector> {
    let mut sums: HashMap<u64, (f32,f32,f32,usize)> = HashMap::new();
    for b in branches {
        let Some(cid) = b.candidate_id else { continue };
        let e = sums.entry(cid).or_insert((0.0,0.0,0.0,0));
        e.0 += b.s_inertial; e.1 += b.s_geometric; e.2 += b.s_colbert; e.3 += 1;
    }
    sums.into_iter().map(|(id,(si,sg,sc,n))| {
        let n = n.max(1) as f32;
        CandidateCostVector::new(id, si/n, sg/n, sc/n)
    }).collect()
}

fn main() {
    // Stampa il range di action e amplitude per capire i proxy.
    for &(n, d, rpc) in &[(16usize, 0.9f32, 8usize), (64, 0.9, 8)] {
        let branches = gen_branches(n, d, 42, rpc);
        let cand = aggrega(&branches);
        let front = estrai_frontiera_pareto_sui_candidati(&cand);
        println!("N={} d={} rpc={}: cand={} front={} scarto={:.1}%", n, d, rpc, cand.len(), front.len(), 100.0*(1.0-front.len() as f64/cand.len() as f64));
        // stampa i valori aggregati
        let mut rows: Vec<_> = cand.iter().map(|c| (c.candidate_id, c.s_inertial, c.s_geometric, c.s_colbert)).collect();
        rows.sort_by_key(|r| r.0);
        for (id, si, sg, sc) in &rows { println!("   cid={} si={:.4} sg={:.4} sc={:.4}", id, si, sg, sc); }
        // Stampa anche action/amplitude grezzi per capire i proxy
        println!("   --- raw action/amplitude per candidato ---");
        let mut sums: HashMap<u64, (f32,f32)> = HashMap::new();
        for b in &branches {
            let e = sums.entry(b.candidate_id.unwrap()).or_insert((0.0,0.0));
            e.0 += b.action; e.1 += b.amplitude;
        }
        let mut s: Vec<_> = sums.into_iter().map(|(k,(a,am))| (k, a/(rpc as f32), am/(rpc as f32))).collect();
        s.sort_by_key(|r| r.0);
        for (id, a, am) in &s { println!("   cid={} avg_action={:.4} avg_amplitude={:.4}", id, a, am); }
    }
}
