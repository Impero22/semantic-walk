#[derive(Debug, Clone)]
pub struct TrajectoryAlignment {
    pub normalized_score: f64,
    pub divergence_token: f64,
    pub warp_path: Vec<(usize, usize)>,
}

pub struct KinematicAligner {
    pub window_size: usize,
}

impl KinematicAligner {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }

    /// Calcola la distanza coseno tra due vettori D-dimensionali: 1.0 - (u · v) / (|u| * |v|)
    pub fn cosine_distance(u: &[f64], v: &[f64]) -> Result<f64, &'static str> {
        if u.len() != v.len() {
            return Err("Disallineamento dimensionale tra i vettori");
        }
        if u.is_empty() {
            return Err("I vettori non possono essere vuoti");
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
        let divergence_token = (n as f64 - m as f64).abs() / path_len;

        Ok(TrajectoryAlignment {
            normalized_score,
            divergence_token,
            warp_path,
        })
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
}
