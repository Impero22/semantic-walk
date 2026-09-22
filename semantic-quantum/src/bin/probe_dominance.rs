// Probe: verifica se i candidati aggregati hanno dominanza reale.
use semantic_quantum::pareto::{estrai_frontiera_pareto_sui_candidati, CandidateCostVector};
use semantic_quantum::{BranchBuilder, BranchType, WalkBranch};

fn gen_branches(n: usize, dominance: f32, seed: u64, rpc: usize) -> Vec<WalkBranch> {
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut next = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (((state >> 11) as f64) / ((1u64 << 53) as f64)) as f32
    };
    let builder = BranchBuilder::new((0.4, 0.3, 0.3));
    let mut out = Vec::with_capacity(n);
    let rpc = rpc.max(1);
    let mut cur = (0.05 + next()*0.9, 0.05 + next()*0.9, 0.05 + next()*0.9);
    for i in 0..n {
        let id = (i / rpc) as u64;
        let (s_i, s_g, s_c) = if i == 0 { cur }
        else if next() < dominance {
            let inc = next()*0.2;
            match (next()*3.0) as usize { 0 => cur.0 += inc, 1 => cur.1 += inc, _ => cur.2 += inc }
            cur
        } else { cur = (0.05 + next()*0.9, 0.05 + next()*0.9, 0.05 + next()*0.9); cur };
        let (s_i, s_g, s_c) = (s_i.min(1.0), s_g.min(1.0), s_c.min(1.0));
        out.push(builder.build_branch(BranchType::Inertial, id, s_i, s_g, (1.0-s_c).max(0.0), 2.0));
    }
    out
}

fn aggrega(branches: &[WalkBranch]) -> Vec<CandidateCostVector> {
    use std::collections::HashMap;
    let mut sums: HashMap<u64, (f32,f32,f32,usize)> = HashMap::new();
    for b in branches {
        let Some(cid) = b.candidate_id else { continue };
        let e = sums.entry(cid).or_insert((0.0,0.0,0.0,0));
        e.0 += b.action; e.1 += b.amplitude; e.2 += b.action*b.amplitude; e.3 += 1;
    }
    sums.into_iter().map(|(id,(si,sg,sc,n))| {
        let n = n.max(1) as f32;
        CandidateCostVector::new(id, si/n, sg/n, sc/n)
    }).collect()
}

fn main() {
    // Caso: N=64, rpc=8, dominance=0.9 (massima dominanza tra rami consecutivi)
    let branches = gen_branches(64, 0.9, 42, 8);
    let cand = aggrega(&branches);
    let front = estrai_frontiera_pareto_sui_candidati(&cand);
    println!("candidati={} front={} scarto={}", cand.len(), front.len(), 100.0*(1.0 - front.len() as f64 / cand.len() as f64));
    for c in &cand {
        println!("  cid={} si={:.4} sg={:.4} sc={:.4}", c.candidate_id, c.s_inertial, c.s_geometric, c.s_colbert);
    }
}
