use crate::ordered_sparse::OrderedSparseSequence;

/// Un passo cinematico del cammino di allineamento.
///
/// Misura la dinamica locale del disallineamento tra i punti allineati:
/// * `velocity` — modulo del vettore errore ‖e_k‖₂;
/// * `acceleration` — derivata discreta della velocità lungo il cammino;
/// * `curvature` — deviazione angolare del vettore errore tra passi consecutivi.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KinematicStep {
    pub velocity: f64,
    pub acceleration: f64,
    pub curvature: f64,
}

#[derive(Debug, Clone)]
pub struct TrajectoryAlignment {
    pub normalized_score: f64,
    pub divergence_token: f64,
    pub warp_path: Vec<(usize, usize)>,
    pub kinematic: Vec<KinematicStep>,
}

pub struct KinematicAligner {
    pub window_size: usize,
    pub alpha: f64,
}

impl KinematicAligner {
    /// Costruttore retrocompatibile: `alpha = 0.0` (τ_div = |N−M|/L, non content-sensitive).
    pub fn new(window_size: usize) -> Self {
        Self { window_size, alpha: 0.0 }
    }

    /// Costruttore con peso esplicito della componente content-sensitive in τ_div.
    pub fn with_alpha(window_size: usize, alpha: f64) -> Self {
        Self { window_size, alpha }
    }

    /// Calcola la distanza coseno tra due vettori D-dimensionali: 1.0 - (u · v) / (|u| * |v|)
    pub fn cosine_distance(u: &[f64], v: &[f64]) -> Result<f64, &'static str> {
        if u.len() != v.len() {
            return Err("Disallineamento dimensionale tra i vettori");
        }
        if u.is_empty() {
            return Err("I vettori non possono essere vuoti");
        }
        if u.iter().any(|x| x.is_nan()) || v.iter().any(|x| x.is_nan()) {
            return Err("NaN nei vettori di input: distanza coseno indefinita");
        }

        let mut dot = 0.0;
        let mut norm_u_sq = 0.0;
        let mut norm_v_sq = 0.0;

        for (&x, &y) in u.iter().zip(v.iter()) {
            dot += x * y;
            norm_u_sq += x * x;
            norm_v_sq += y * y;
        }

        if norm_u_sq == 0.0 || norm_v_sq == 0.0 {
            return Ok(1.0);
        }

        let sim = (dot / (norm_u_sq.sqrt() * norm_v_sq.sqrt())).clamp(-1.0, 1.0);
        Ok((1.0 - sim).max(0.0))
    }

    /// Calcola le metriche cinematiche (v_k, a_k, κ_k) lungo il cammino di allineamento.
    ///
    /// Per ogni passo k del warp_path, con e_k = x_{i_k} − y_{j_k}:
    /// * `v_k = ‖e_k‖₂` (norma L2 del disallineamento locale);
    /// * `a_k = v_k − v_{k−1}` (derivata discreta della velocità, a_0 = 0.0);
    /// * `κ_k = 1.0 − (e_k·e_{k−1}) / (‖e_k‖₂·‖e_{k−1}‖₂)` (deviazione angolare, κ_0 = 0.0).
    fn compute_kinematic<T: AsRef<[f64]>>(
        seq_a: &[T],
        seq_b: &[T],
        warp_path: &[(usize, usize)],
    ) -> Vec<KinematicStep> {
        let mut kinematic = Vec::with_capacity(warp_path.len());

        // e_k per ogni passo (vettore errore). Salviamo le norme e i vettori
        // per calcolare le derivate e le deviazioni angolari tra passi consecutivi.
        let mut prev_error: Option<Vec<f64>> = None;
        let mut prev_velocity: Option<f64> = None;

        for &(i, j) in warp_path {
            let x = seq_a[i].as_ref();
            let y = seq_b[j].as_ref();

            let e: Vec<f64> = x.iter().zip(y.iter()).map(|(&a, &b)| a - b).collect();
            let v = e.iter().map(|c| c * c).sum::<f64>().sqrt();

            let a = match prev_velocity {
                Some(pv) => v - pv,
                None => 0.0,
            };

            let kappa = match &prev_error {
                Some(pe) => {
                    let dot: f64 = e.iter().zip(pe.iter()).map(|(&a, &b)| a * b).sum();
                    let norm_e = v;
                    let norm_pe = pe.iter().map(|c| c * c).sum::<f64>().sqrt();
                    if norm_e == 0.0 || norm_pe == 0.0 {
                        0.0
                    } else {
                        (1.0 - (dot / (norm_e * norm_pe))).max(0.0)
                    }
                }
                None => 0.0,
            };

            kinematic.push(KinematicStep { velocity: v, acceleration: a, curvature: kappa });

            prev_error = Some(e);
            prev_velocity = Some(v);
        }

        kinematic
    }

    /// Calcola l'allineamento DTW vettoriale su sequenze di punti D-dimensionali
    pub fn align<T: AsRef<[f64]>>(
        &self,
        seq_a: &[T],
        seq_b: &[T],
    ) -> Result<TrajectoryAlignment, &'static str> {
        let n = seq_a.len();
        let m = seq_b.len();

        if n == 0 || m == 0 {
            return Err("Le sequenze di traiettoria non possono essere vuote");
        }

        let dim = seq_a[0].as_ref().len();
        if dim == 0 {
            return Err("La dimensione dei vettori deve essere maggiore di zero");
        }

        let mut cost_matrix = vec![vec![f64::INFINITY; m + 1]; n + 1];
        cost_matrix[0][0] = 0.0;

        for i in 1..=n {
            let p_a = seq_a[i - 1].as_ref();
            if p_a.len() != dim {
                return Err("Dimensione del vettore non coerente lungo la sequenza A");
            }

            let window_start = (i as isize - self.window_size as isize).max(1) as usize;
            let window_end = (i + self.window_size).min(m);

            for j in window_start..=window_end {
                let p_b = seq_b[j - 1].as_ref();
                if p_b.len() != dim {
                    return Err("Dimensione del vettore non coerente lungo la sequenza B");
                }

                let local_cost = Self::cosine_distance(p_a, p_b)?;

                let min_prev = cost_matrix[i - 1][j]
                    .min(cost_matrix[i][j - 1])
                    .min(cost_matrix[i - 1][j - 1]);

                cost_matrix[i][j] = local_cost + min_prev;
            }
        }

        let mut curr_i = n;
        let mut curr_j = m;
        let mut warp_path = Vec::new();
        warp_path.push((curr_i - 1, curr_j - 1));

        while curr_i > 1 || curr_j > 1 {
            if curr_i == 1 {
                curr_j -= 1;
            } else if curr_j == 1 {
                curr_i -= 1;
            } else {
                let diag = cost_matrix[curr_i - 1][curr_j - 1];
                let left = cost_matrix[curr_i][curr_j - 1];
                let up = cost_matrix[curr_i - 1][curr_j];

                if diag <= left && diag <= up {
                    curr_i -= 1;
                    curr_j -= 1;
                } else if left <= up {
                    curr_j -= 1;
                } else {
                    curr_i -= 1;
                }
            }
            warp_path.push((curr_i - 1, curr_j - 1));
        }

        warp_path.reverse();

        let total_cost = cost_matrix[n][m];
        let path_len = warp_path.len() as f64;
        let normalized_score = total_cost / path_len;
        let divergence_token =
            ((n as f64 - m as f64).abs() / path_len) + self.alpha * normalized_score;
        let kinematic = Self::compute_kinematic(seq_a, seq_b, &warp_path);

        Ok(TrajectoryAlignment {
            normalized_score,
            divergence_token,
            warp_path,
            kinematic,
        })
    }

    /// **Allineamento DTW con guida ordered-sparse (due strati)**.
    pub fn align_with_ordered_sparse<T: AsRef<[f64]>>(
        &self,
        seq_a: &[T],
        seq_b: &[T],
        sparse_a: &OrderedSparseSequence,
        sparse_b: &OrderedSparseSequence,
        min_overlap_threshold: u32,
        w_min: usize,
        w_max: usize,
    ) -> Result<Option<TrajectoryAlignment>, &'static str> {
        let n = seq_a.len();
        let m = seq_b.len();

        if n == 0 || m == 0 {
            return Err("Le sequenze di traiettoria non possono essere vuote");
        }

        // Coerenza dimensionale tra traiettorie dense e guide sparse.
        if sparse_a.num_positions() != n || sparse_b.num_positions() != m {
            return Err("Disallineamento tra sequenze dense e guide ordered-sparse");
        }

        let dim = seq_a[0].as_ref().len();
        if dim == 0 {
            return Err("La dimensione dei vettori deve essere maggiore di zero");
        }

        // Strato 1 — Guardiano O(1): pruning topologico prima di allocare
        // la matrice di allineamento.
        let overlap = sparse_a.global_overlap(sparse_b);
        if overlap < min_overlap_threshold {
            return Ok(None);
        }

        // Strato 2 — Banda di Sakoe-Chiba dinamica.
        let (w_min, w_max) = if w_min <= w_max {
            (w_min, w_max)
        } else {
            (w_max, w_min)
        };
        let w_base = self.window_size;

        let mut cost_matrix = vec![vec![f64::INFINITY; m + 1]; n + 1];
        cost_matrix[0][0] = 0.0;

        for i in 1..=n {
            let p_a = seq_a[i - 1].as_ref();
            if p_a.len() != dim {
                return Err("Dimensione del vettore non coerente lungo la sequenza A");
            }

            let j = if i - 1 < m {
                sparse_a.positional_jaccard(sparse_b, i - 1)
            } else {
                0.0
            };
            let w_i = if j >= 0.7 {
                w_min
            } else if j < 0.3 {
                w_max
            } else {
                let t = (j - 0.3) / 0.4;
                (w_min as f32 + (w_max as f32 - w_min as f32) * t) as usize
            };
            let w_i = w_i.min(w_base.max(1));

            let window_start = (i as isize - w_i as isize).max(1) as usize;
            let window_end = (i + w_i).min(m);

            for jj in window_start..=window_end {
                let p_b = seq_b[jj - 1].as_ref();
                if p_b.len() != dim {
                    return Err("Dimensione del vettore non coerente lungo la sequenza B");
                }

                let mut local_cost = Self::cosine_distance(p_a, p_b)?;

                if j < 0.3 {
                    local_cost *= 1.0 + (0.3 - j) as f64;
                }

                let min_prev = cost_matrix[i - 1][jj]
                    .min(cost_matrix[i][jj - 1])
                    .min(cost_matrix[i - 1][jj - 1]);

                cost_matrix[i][jj] = local_cost + min_prev;
            }
        }

        let mut curr_i = n;
        let mut curr_j = m;
        let mut warp_path = Vec::new();
        warp_path.push((curr_i - 1, curr_j - 1));

        while curr_i > 1 || curr_j > 1 {
            if curr_i == 1 {
                curr_j -= 1;
            } else if curr_j == 1 {
                curr_i -= 1;
            } else {
                let diag = cost_matrix[curr_i - 1][curr_j - 1];
                let left = cost_matrix[curr_i][curr_j - 1];
                let up = cost_matrix[curr_i - 1][curr_j];

                if diag <= left && diag <= up {
                    curr_i -= 1;
                    curr_j -= 1;
                } else if left <= up {
                    curr_j -= 1;
                } else {
                    curr_i -= 1;
                }
            }
            warp_path.push((curr_i - 1, curr_j - 1));
        }

        warp_path.reverse();

        let total_cost = cost_matrix[n][m];

        if !total_cost.is_finite() {
            return Ok(None);
        }

        let path_len = warp_path.len() as f64;
        let normalized_score = total_cost / path_len;
        let divergence_token =
            ((n as f64 - m as f64).abs() / path_len) + self.alpha * normalized_score;
        let kinematic = Self::compute_kinematic(seq_a, seq_b, &warp_path);

        Ok(Some(TrajectoryAlignment {
            normalized_score,
            divergence_token,
            warp_path,
            kinematic,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dtw_vectorial_alignment_basic() {
        let aligner = KinematicAligner::new(3);
        let seq_a = vec![
            vec![1.0, 0.0, 0.0, 0.5],
            vec![0.0, 1.0, 0.0, 0.5],
        ];
        let seq_b = vec![
            vec![1.0, 0.0, 0.0, 0.5],
            vec![0.0, 1.0, 0.0, 0.5],
        ];

        let res = aligner.align(&seq_a, &seq_b).unwrap();
        assert_eq!(res.warp_path.len(), 2);
        assert!(res.normalized_score < 1e-12);
        assert_eq!(res.divergence_token, 0.0);
    }

    #[test]
    fn test_cosine_distance_unnormalized() {
        let u = vec![2.0, 0.0, 0.0];
        let v = vec![5.0, 0.0, 0.0];
        let dist = KinematicAligner::cosine_distance(&u, &v).unwrap();
        assert!(dist < 1e-12);
    }

    fn seq_identica(n: usize) -> Vec<Vec<f64>> {
        (0..n).map(|_| vec![1.0, 0.0, 0.0, 0.5]).collect()
    }

    fn sparse_identico(n: usize) -> OrderedSparseSequence {
        OrderedSparseSequence::from_frames(
            &(0..n)
                .map(|_| vec![(1u32, 0.5f32), (2u32, 0.3f32)])
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn sparse_disgiunto(n: usize) -> OrderedSparseSequence {
        OrderedSparseSequence::from_frames(
            &(0..n)
                .map(|_| vec![(100_000u32, 0.5f32), (4_000_000u32, 0.3f32)])
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    const SOGLIA_DISCRIMINANTE: u32 = 90;

    #[test]
    fn test_strato1_pruning_ritiro_geometrico() {
        let aligner = KinematicAligner::new(3);
        let seq = seq_identica(4);
        let sa = sparse_identico(4);
        let sb = sparse_disgiunto(4);
        let res = aligner
            .align_with_ordered_sparse(&seq, &seq, &sa, &sb, SOGLIA_DISCRIMINANTE, 1, 3)
            .unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_strato1_nessun_ritiro_se_overlap_ok() {
        let aligner = KinematicAligner::new(3);
        let seq = seq_identica(4);
        let sa = sparse_identico(4);
        let sb = sparse_identico(4);
        let res = aligner
            .align_with_ordered_sparse(&seq, &seq, &sa, &sb, SOGLIA_DISCRIMINANTE, 1, 3)
            .unwrap();
        assert!(res.is_some());
        let a = res.unwrap();
        assert!(a.normalized_score < 1e-12);
    }

    #[test]
    fn test_strato1_soglia_zero_non_ritira() {
        let aligner = KinematicAligner::new(3);
        let seq = seq_identica(4);
        let sa = sparse_identico(4);
        let sb = sparse_disgiunto(4);
        let res = aligner
            .align_with_ordered_sparse(&seq, &seq, &sa, &sb, 0, 1, 3)
            .unwrap();
        assert!(res.is_some());
    }

    #[test]
    fn test_disallineamento_dimensione_sparse_dense() {
        let aligner = KinematicAligner::new(3);
        let seq = seq_identica(4);
        let sa = sparse_identico(4);
        let sb = sparse_identico(3);
        let res = aligner.align_with_ordered_sparse(&seq, &seq, &sa, &sb, 4, 1, 3);
        assert!(res.is_err());
    }

    #[test]
    fn test_wmin_wmax_invertiti_normalizzati() {
        let aligner = KinematicAligner::new(3);
        let seq = seq_identica(4);
        let sa = sparse_identico(4);
        let sb = sparse_identico(4);
        let res = aligner
            .align_with_ordered_sparse(&seq, &seq, &sa, &sb, 0, 5, 1)
            .unwrap();
        assert!(res.is_some());
    }

    #[test]
    fn test_n_maggiore_di_m_non_panica() {
        let aligner = KinematicAligner::new(3);
        let seq_a = seq_identica(5);
        let seq_b = seq_identica(3);
        let sa = sparse_identico(5);
        let sb = sparse_identico(3);
        let res = aligner
            .align_with_ordered_sparse(&seq_a, &seq_b, &sa, &sb, 0, 1, 3)
            .unwrap();
        assert!(res.is_some());
    }

    #[test]
    fn test_ritiro_geometrico_banda_troppo_stretta() {
        let aligner = KinematicAligner::new(1);
        let a: Vec<[f64; 2]> = (0..10).map(|i| [i as f64, 0.0]).collect();
        let b: Vec<[f64; 2]> = vec![[0.0, 0.0], [9.0, 0.0]];
        let sa = sparse_identico(10);
        let sb = sparse_identico(2);
        let res = aligner
            .align_with_ordered_sparse(&a, &b, &sa, &sb, 0, 1, 1)
            .unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_alpha_zero_tau_div_identica_a_oggi() {
        // Con alpha=0.0 (default di `new`), τ_div è identica alla versione
        // pre-patch: solo la componente |N−M|/L, senza contributo content-sensitive.
        let aligner = KinematicAligner::new(3);
        let seq_a = vec![
            vec![1.0, 0.0, 0.0, 0.5],
            vec![0.0, 1.0, 0.0, 0.5],
            vec![1.0, 0.0, 0.0, 0.5],
        ];
        let seq_b = vec![
            vec![1.0, 0.0, 0.0, 0.5],
            vec![0.0, 1.0, 0.0, 0.5],
        ];

        let res = aligner.align(&seq_a, &seq_b).unwrap();
        let expected_base = (3.0_f64 - 2.0_f64).abs() / res.warp_path.len() as f64;
        assert!((res.divergence_token - expected_base).abs() < 1e-9);
    }

    #[test]
    fn test_alpha_positivo_arricchisce_tau_div() {
        // Con alpha>0, τ_div = |N−M|/L + alpha * normalized_score: il contributo
        // content-sensitive è presente e cresce con alpha.
        let aligner = KinematicAligner::with_alpha(3, 1.0);
        let seq_a = vec![
            vec![1.0, 0.0, 0.0, 0.5],
            vec![0.0, 1.0, 0.0, 0.5],
            vec![1.0, 0.0, 0.0, 0.5],
        ];
        let seq_b = vec![
            vec![1.0, 0.0, 0.0, 0.5],
            vec![0.0, 1.0, 0.0, 0.5],
        ];

        let res = aligner.align(&seq_a, &seq_b).unwrap();
        let expected_base = (3.0_f64 - 2.0_f64).abs() / res.warp_path.len() as f64;
        let expected = expected_base + 1.0 * res.normalized_score;
        assert!((res.divergence_token - expected).abs() < 1e-9);
    }

    #[test]
    fn test_kinematic_popolato_lunghezza_warp_path() {
        // Il campo `kinematic` è popolato con un passo per ogni punto del warp_path.
        let aligner = KinematicAligner::new(3);
        let seq_a = vec![
            vec![1.0, 0.0, 0.0, 0.5],
            vec![0.0, 1.0, 0.0, 0.5],
            vec![1.0, 0.0, 0.0, 0.5],
        ];
        let seq_b = vec![
            vec![1.0, 0.0, 0.0, 0.5],
            vec![0.0, 1.0, 0.0, 0.5],
        ];

        let res = aligner.align(&seq_a, &seq_b).unwrap();
        assert_eq!(res.kinematic.len(), res.warp_path.len());
    }

    #[test]
    fn test_kinematic_primo_passo_velocita_zero() {
        // Il primo passo del warp_path allinea punti identici (stesso vettore):
        // e_0 = 0 → v_0 = 0, a_0 = 0, κ_0 = 0.
        let aligner = KinematicAligner::new(3);
        let seq_a = vec![vec![1.0, 0.0, 0.0, 0.5]];
        let seq_b = vec![vec![1.0, 0.0, 0.0, 0.5]];

        let res = aligner.align(&seq_a, &seq_b).unwrap();
        assert_eq!(res.kinematic.len(), 1);
        let k0 = res.kinematic[0];
        assert!(k0.velocity.abs() < 1e-9);
        assert!(k0.acceleration.abs() < 1e-9);
        assert!(k0.curvature.abs() < 1e-9);
    }
}
