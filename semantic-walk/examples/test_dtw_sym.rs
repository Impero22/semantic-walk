use semantic_walk::dtw::KinematicAligner;
use semantic_walk::ordered_sparse::OrderedSparseSequence;

fn main() {
    let x = vec![1.0, 0.0];
    let y = vec![0.0, 1.0];
    let conn = vec![0.7, 0.7];
    
    let seq_a = vec![x.clone(), conn.clone(), y.clone()];
    let seq_b = vec![y.clone(), conn.clone(), x.clone()];
    
    let aligner = KinematicAligner::new(10);
    
    let sparse_a = OrderedSparseSequence::from_frames(&vec![
        vec![(1, 1.0)], vec![(2, 1.0)], vec![(3, 1.0)],
    ]).unwrap();
    let sparse_b = OrderedSparseSequence::from_frames(&vec![
        vec![(3, 1.0)], vec![(2, 1.0)], vec![(1, 1.0)],
    ]).unwrap();
    
    let r_ab = aligner.align_with_ordered_sparse(&seq_a, &seq_b, &sparse_a, &sparse_b, 5, 1, 10);
    let r_ba = aligner.align_with_ordered_sparse(&seq_b, &seq_a, &sparse_b, &sparse_a, 5, 1, 10);
    
    println!("align(A,B) = {:?}", r_ab.map(|o| o.map(|a| a.normalized_score)));
    println!("align(B,A) = {:?}", r_ba.map(|o| o.map(|a| a.normalized_score)));
}
