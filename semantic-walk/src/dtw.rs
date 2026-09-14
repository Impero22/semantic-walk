use semantic_combiner::{combine, NormalizedAxes};

pub const LAMBDA_CALIBRATO: f64 = 10.64;
pub const PESI_CALIBRATI: [f64; 3] = [0.215, 0.552, 0.233];

#[derive(Debug, Clone)]
pub struct TrajectoryPoint {
    pub dense: f64,
    pub sparse: f64,
    pub colbert: f64,
}

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

    pub fn align(
        &self,
        seq_a: &[TrajectoryPoint],
        seq_b: &[TrajectoryPoint],
    ) -> Result<TrajectoryAlignment, &'static str> {
        let n = seq_a.len();
        let m = seq_b.len();

        if n == 0 || m == 0 {
            return Err("Le sequenze di traiettoria non possono essere vuote");
        }

        let mut cost_matrix = vec![vec![f64::INFINITY; m + 1]; n + 1];
        cost_matrix[0][0] = 0.0;

        for i in 1..=n {
            let p_a = &seq_a[i - 1];
            let window_start = (i as isize - self.window_size as isize).max(1) as usize;
            let window_end = (i + self.window_size).min(m);

            for j in window_start..=window_end {
                let p_b = &seq_b[j - 1];

                let sim_dense = 1.0 - (p_a.dense - p_b.dense).abs();
                let sim_sparse = 1.0 - (p_a.sparse - p_b.sparse).abs();
                let sim_colbert = 1.0 - (p_a.colbert - p_b.colbert).abs();

                let axes = NormalizedAxes::normalize(
                    sim_dense,
                    sim_sparse,
                    sim_colbert,
                    LAMBDA_CALIBRATO,
                );

                let combined_sim = combine(&axes, PESI_CALIBRATI);
                let local_cost = (1.0 - combined_sim).max(0.0);

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
    fn test_dtw_alignment_basic() {
        let aligner = KinematicAligner::new(3);
        let seq_a = vec![
            TrajectoryPoint { dense: 0.9, sparse: 0.1, colbert: 0.8 },
            TrajectoryPoint { dense: 0.7, sparse: 0.2, colbert: 0.6 },
        ];
        let seq_b = vec![
            TrajectoryPoint { dense: 0.9, sparse: 0.1, colbert: 0.8 },
            TrajectoryPoint { dense: 0.7, sparse: 0.2, colbert: 0.6 },
        ];

        let res = aligner.align(&seq_a, &seq_b).unwrap();
        assert_eq!(res.warp_path.len(), 2);
        assert!(res.normalized_score < 1e-3);
        assert_eq!(res.divergence_token, 0.0);
    }
}
