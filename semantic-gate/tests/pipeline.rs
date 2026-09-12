use std::time::{Duration, Instant};
use semantic_gate::{Gate, Verdict};
use semantic_combiner::{NormalizedAxes, combine, LAMBDA_CALIBRATO, PESI_CALIBRATI};
use semantic_colbert::maxsim;

#[test]
fn test_pipeline_completa_gate_colbert_combiner() {
    let gate = Gate::default();
    let deadline = Instant::now() + Duration::from_secs(1);

    // 1. Metriche economiche in ingresso
    let dense_score = 0.8;
    let sparse_score = 12.0;

    // 2. Valutazione del gate
    let verdetto = gate.decide(dense_score, sparse_score, deadline);
    assert_eq!(verdetto, Verdict::Passa);

    // 3. Esecuzione Late Interaction (ColBERT MaxSim)
    let query_tokens = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
    let doc_tokens = vec![vec![0.9, 0.1], vec![0.1, 0.9]];
    let q_refs: Vec<&[f64]> = query_tokens.iter().map(|v| v.as_slice()).collect();
    let d_refs: Vec<&[f64]> = doc_tokens.iter().map(|v| v.as_slice()).collect();

    let colbert_score = maxsim(&q_refs, &d_refs);
    assert!(!colbert_score.is_nan());

    // 4. Aggregazione tricanale Pareto-coerente
    let axes = NormalizedAxes::normalize(dense_score, sparse_score, colbert_score, LAMBDA_CALIBRATO);
    let final_score = combine(&axes, PESI_CALIBRATI);

    assert!(final_score > 0.0 && final_score <= 1.0);
}
