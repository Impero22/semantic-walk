// Probe 2: generatore con dominanza reale tra candidati.
use semantic_quantum::pareto::{estrai_frontiera_pareto_sui_candidati, CandidateCostVector};

fn main() {
    // 8 candidati: i primi 4 "buoni" (costi bassi), gli ultimi 4 "dominati"
    // (stessi costi + incremento su tutti e 3 gli assi).
    let mut cand = Vec::new();
    let base = [(0.2,0.3,0.4),(0.25,0.28,0.35),(0.22,0.35,0.3),(0.3,0.2,0.38)];
    for (i,(si,sg,sc)) in base.iter().enumerate() {
        cand.push(CandidateCostVector::new(i as u64, *si, *sg, *sc));
    }
    for (i,(si,sg,sc)) in base.iter().enumerate() {
        cand.push(CandidateCostVector::new((i+4) as u64, si+0.3, sg+0.3, sc+0.3));
    }
    let front = estrai_frontiera_pareto_sui_candidati(&cand);
    println!("candidati={} front={} scarto={:.1}%", cand.len(), front.len(), 100.0*(1.0 - front.len() as f64/cand.len() as f64));
    for c in &front { println!("  FRONT cid={} si={} sg={} sc={}", c.candidate_id, c.s_inertial, c.s_geometric, c.s_colbert); }
}
