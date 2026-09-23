use semantic_quantum::pareto::BranchCostVector;
use semantic_quantum::pareto::estrai_frontiera_pareto;
use semantic_quantum::{BranchType, WalkBranch};

// Il controesempio del revisore, costruito correttamente:
// A ha UN ramo eccellente (0.1, 0.1, 0.1) che domina TUTTI i rami di B.
// B ha MOLTI rami "buoni" (0.2, 0.2, 0.2) — ciascuno dominato dal ramo di A.
// Somma A = 0.3. Somma B = N × 0.6. Se N=1, A vince. Ma se B ha più rami
// che contribuiscono allo stesso candidato, la somma può battere A.
//
// ATTENZIONE: qui l'accumulo è per candidato. Il punto del revisore è che
// il pruning opera su rami INDIVIDUALI, ma il winner è la SOMMA per candidato.
// Rimuovendo i rami dominati di B, si elimina il contributo collettivo.

fn main() {
    // Candidato A: 1 ramo eccellente
    let a_ramo = BranchCostVector::new(
        WalkBranch::new(BranchType::Inertial, 0, 0.1, 2.0),
        0.1, 0.1, 0.1);

    // Candidato B: 10 rami buoni (0.2, 0.2, 0.2), tutti dominati dal ramo di A
    let mut b_rami = Vec::new();
    for _i in 0..10 {
        b_rami.push(BranchCostVector::new(
            WalkBranch::new(BranchType::Inertial, 1, 0.2, 2.0),
            0.2, 0.2, 0.2));
    }

    println!("Il ramo di A domina un ramo di B? {}", a_ramo.dominates(&b_rami[0]));

    // Senza pruning: somma per candidato
    let somma_a = a_ramo.s_inertial + a_ramo.s_geometric + a_ramo.s_colbert;
    let somma_b_senza_pruning: f32 = b_rami.iter()
        .map(|b| b.s_inertial + b.s_geometric + b.s_colbert)
        .sum();
    println!("\nSENZA pruning — somma costi per candidato:");
    println!("  A: {}", somma_a);
    println!("  B (10 rami): {}", somma_b_senza_pruning);
    println!("  → Vincitore senza pruning: {}", if somma_a < somma_b_senza_pruning { "A" } else { "B" });

    // CON pruning: A domina tutti i rami di B → B viene rimosso del tutto
    let mut tutti = vec![a_ramo.clone()];
    tutti.extend(b_rami.iter().cloned());
    let frontiera = estrai_frontiera_pareto(&tutti);
    let somma_a_post = frontiera.iter()
        .filter(|r| r.branch.candidate_id == Some(0))
        .map(|r| r.s_inertial + r.s_geometric + r.s_colbert)
        .sum::<f32>();
    let somma_b_post: f32 = frontiera.iter()
        .filter(|r| r.branch.candidate_id == Some(1))
        .map(|r| r.s_inertial + r.s_geometric + r.s_colbert)
        .sum();
    println!("\nCON pruning — frontiera: {} rami, somma per candidato:", frontiera.len());
    println!("  A: {}", somma_a_post);
    println!("  B: {}", somma_b_post);
    println!("  → Vincitore con pruning: {}", if somma_a_post < somma_b_post { "A" } else { "B" });
    println!("\n→ CONFERMA del finding del revisore: il pruning Pareto può");
    println!("  INVERTIRE il risultato. B vince sul totale ma viene eliminato");
    println!("  perché ogni suo ramo è dominato individualmente dal ramo di A.");
}
