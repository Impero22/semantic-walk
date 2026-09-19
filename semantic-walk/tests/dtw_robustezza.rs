// Test di robustezza per il dtw di Camillo (distanza coseno normalizzata)
use semantic_walk::dtw::KinematicAligner;

#[test]
fn norma_zero_distanza_massima() {
    let u = vec![0.0, 0.0, 0.0];
    let v = vec![1.0, 0.0, 0.0];
    let d = KinematicAligner::cosine_distance(&u, &v).unwrap();
    assert!((d - 1.0).abs() < 1e-12, "norma zero: atteso 1.0, ottenuto {d}");
}

#[test]
fn disallineamento_dimensionale_errore() {
    let a = vec![1.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    assert!(KinematicAligner::cosine_distance(&a, &b).is_err());
}

#[test]
fn vettori_identici_non_normalizzati_distanza_zero() {
    let p = vec![3.0, 4.0, 0.0]; // norma 5
    let d = KinematicAligner::cosine_distance(&p, &p).unwrap();
    assert!(d < 1e-12, "vettori identici: atteso 0, ottenuto {d}");
}

#[test]
fn vettori_ortogonali_distanza_uno() {
    let x = vec![1.0, 0.0, 0.0];
    let y = vec![0.0, 1.0, 0.0];
    let d = KinematicAligner::cosine_distance(&x, &y).unwrap();
    assert!((d - 1.0).abs() < 1e-12, "ortogonali: atteso 1.0, ottenuto {d}");
}

#[test]
fn vettori_opposti_distanza_due() {
    let x = vec![1.0, 0.0, 0.0];
    let z = vec![-1.0, 0.0, 0.0];
    let d = KinematicAligner::cosine_distance(&x, &z).unwrap();
    assert!((d - 2.0).abs() < 1e-12, "opposti: atteso 2.0, ottenuto {d}");
}

#[test]
fn sequenza_vuota_errore() {
    let aligner = KinematicAligner::new(3);
    let empty: Vec<Vec<f64>> = vec![];
    let seq = vec![vec![1.0, 0.0]];
    assert!(aligner.align(&empty, &seq).is_err());
}

#[test]
fn dtw_lunghezze_diverse_allinea() {
    let aligner = KinematicAligner::new(3);
    let a = vec![vec![1.0,0.0], vec![0.0,1.0], vec![1.0,0.0]];
    let b = vec![vec![1.0,0.0], vec![0.0,1.0]];
    let res = aligner.align(&a, &b).unwrap();
    assert!(!res.warp_path.is_empty());
    assert!(res.normalized_score >= 0.0);
    assert!(res.divergence_token >= 0.0);
}

#[test]
fn nan_nei_vettori_errore_non_distanza_zero() {
    // Finding #2 della review: NaN in input produceva 0.0 (falso "identico")
    // per via di f64::max(NaN, 0.0) == 0.0. Ora deve ritornare un errore.
    let u = vec![f64::NAN, 0.0, 0.0];
    let v = vec![1.0, 0.0, 0.0];
    let res = KinematicAligner::cosine_distance(&u, &v);
    assert!(res.is_err(), "NaN in input deve produrre errore, non distanza 0.0");
}

#[test]
fn nan_in_entrambi_i_vettori_errore() {
    let u = vec![f64::NAN, f64::NAN];
    let v = vec![f64::NAN, f64::NAN];
    let res = KinematicAligner::cosine_distance(&u, &v);
    assert!(res.is_err(), "NaN in entrambi i vettori deve produrre errore");
}

#[test]
fn align_con_nan_errore() {
    let aligner = KinematicAligner::new(3);
    let a = vec![vec![f64::NAN, 0.0], vec![0.0, 1.0]];
    let b = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
    let res = aligner.align(&a, &b);
    assert!(res.is_err(), "sequenza con NaN deve produrre errore nell'allineamento");
}
