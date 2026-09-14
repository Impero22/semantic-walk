//! # ingest — il contratto dei dati CrispEmbed
//!
//! La porta d'ingresso dei dati vettoriali ad alta dimensione (D = 1024)
//! generati da CrispEmbed nel cammino semantico.
//!
//! Qui si definisce il *contratto* dei dati: una traiettoria è una sequenza
//! ordinata di embedding, tutti della stessa dimensione, identificata da un
//! `sequence_id`. L'ordine conta — è una traiettoria, non una borsa.
//!
//! ## Il gate come prima linea
//!
//! Allineare due traiettorie con il DTW è il costo alto. Prima di spendere
//! quel costo, il gate economico (dense + sparse) decide se il confronto
//! vale la pena. Il gate è permissivo: meglio un allineamento sprecato che
//! un confronto perso.

use crate::dtw::{KinematicAligner, TrajectoryAlignment};
use semantic_gate::{Gate, Verdict};

/// Una sequenza di embedding ad alta dimensione generata da CrispEmbed.
///
/// Rappresenta il cammino semantico di un fatto o di una query: i token
/// nell'ordine esatto in cui compaiono, ognuno con il proprio vettore
/// D-dimensionale.
#[derive(Debug, Clone, PartialEq)]
pub struct CrispTrajectory {
    /// Identificativo della sequenza (es. id del fatto nella memoria).
    pub sequence_id: String,
    /// Dimensione comune di ogni embedding (per CrispEmbed: 1024).
    pub dimension: usize,
    /// Gli embedding nell'ordine della traiettoria.
    pub embeddings: Vec<Vec<f64>>,
}

impl CrispTrajectory {
    /// Costruisce una traiettoria, verificando la coerenza dimensionale.
    ///
    /// Fallisce se la dimensione è zero, o se un embedding non rispetta la
    /// dimensione dichiarata. Il contratto è rigido: una traiettoria con
    /// vettori di dimensioni diverse è un dato corrotto, non un dato valido.
    pub fn new(
        sequence_id: impl Into<String>,
        dimension: usize,
        embeddings: Vec<Vec<f64>>,
    ) -> Result<Self, &'static str> {
        if dimension == 0 {
            return Err("La dimensione dell'embedding deve essere maggiore di zero");
        }
        for emb in &embeddings {
            if emb.len() != dimension {
                return Err("Incoerenza nella dimensione di un vettore della traiettoria");
            }
        }
        Ok(Self {
            sequence_id: sequence_id.into(),
            dimension,
            embeddings,
        })
    }

    /// Il numero di embedding (passi) della traiettoria.
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// `true` se la traiettoria non ha passi.
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }
}

/// Allinea due traiettorie CrispEmbed con il DTW.
///
/// Le traiettorie devono avere la stessa dimensione; il DTW opera sui punti
/// geometrici proiettati. Ritorna l'allineamento (warp path, punteggio
/// normalizzato, token di divergenza).
pub fn align_crisp_trajectories(
    traj_a: &CrispTrajectory,
    traj_b: &CrispTrajectory,
    window_size: usize,
) -> Result<TrajectoryAlignment, &'static str> {
    if traj_a.dimension != traj_b.dimension {
        return Err("Disallineamento dimensionale tra le due traiettorie CrispEmbed");
    }

    let aligner = KinematicAligner::new(window_size);
    aligner.align(&traj_a.embeddings, &traj_b.embeddings)
}

/// Il contratto di integrazione col gate: decide se vale la pena allineare.
///
/// Prima di spendere il costo del DTW, il gate economico valuta il confronto
/// con le sonde dense e sparse. Se il verdetto è `Passa`, si procede
/// all'allineamento; se è `Blocca`, si rinuncia (risparmiando il costo);
/// se è `Timeout`, ci si ritira permissivamente e si procede comunque
/// (meglio un allineamento sprecato che un confronto perso).
///
/// Ritorna `None` quando il gate blocca il confronto: il candidato non
/// merita il costo del DTW.
pub fn gate_crisp_alignment(
    gate: &Gate,
    traj_a: &CrispTrajectory,
    traj_b: &CrispTrajectory,
    dense: f64,
    sparse: f64,
    deadline: std::time::Instant,
    window_size: usize,
) -> Result<Option<TrajectoryAlignment>, &'static str> {
    if traj_a.dimension != traj_b.dimension {
        return Err("Disallineamento dimensionale tra le due traiettorie CrispEmbed");
    }

    match gate.decide(dense, sparse, deadline) {
        // Il gate passa (o si ritira permissivamente): vale la pena allineare.
        Verdict::Passa | Verdict::Timeout => {
            let aligner = KinematicAligner::new(window_size);
            aligner.align(&traj_a.embeddings, &traj_b.embeddings).map(Some)
        }
        // Il gate blocca: non spendere il DTW per questo confronto.
        Verdict::Blocca => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn traj(dim: usize, v: Vec<Vec<f64>>) -> CrispTrajectory {
        CrispTrajectory::new("traj", dim, v).unwrap()
    }

    #[test]
    fn test_crisp_ingestion_and_alignment() {
        let traj_a = traj(
            4,
            vec![vec![1.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0, 0.0]],
        );
        let traj_b = traj(
            4,
            vec![vec![1.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0, 0.0]],
        );

        let alignment = align_crisp_trajectories(&traj_a, &traj_b, 2).unwrap();
        assert_eq!(alignment.divergence_token, 0.0);
        assert!(alignment.normalized_score < 1e-12);
    }

    #[test]
    fn test_dimension_mismatch_rejected() {
        let traj_a = traj(4, vec![vec![1.0, 0.0, 0.0, 0.0]]);
        let traj_b = traj(3, vec![vec![1.0, 0.0, 0.0]]);

        assert!(align_crisp_trajectories(&traj_a, &traj_b, 2).is_err());
    }

    #[test]
    fn test_new_rejects_zero_dimension() {
        assert!(CrispTrajectory::new("t", 0, vec![]).is_err());
    }

    #[test]
    fn test_new_rejects_inconsistent_embedding() {
        assert!(CrispTrajectory::new("t", 4, vec![vec![1.0, 0.0, 0.0]]).is_err());
    }

    #[test]
    fn test_gate_blocca_rinuncia_allineamento() {
        // Gate con soglia altissima: blocca tutto.
        let gate = Gate::new(semantic_gate::GateConfig {
            pesi_dense: 0.5,
            pesi_sparse: 0.5,
            soglia: 0.99,
            budget_ns: 10_000_000,
        });
        let traj_a = traj(4, vec![vec![1.0, 0.0, 0.0, 0.0]]);
        let traj_b = traj(4, vec![vec![1.0, 0.0, 0.0, 0.0]]);
        let deadline = Instant::now() + Duration::from_secs(60);

        let res = gate_crisp_alignment(&gate, &traj_a, &traj_b, 0.1, 0.1, deadline, 2).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_gate_passa_esegue_allineamento() {
        // Gate con soglia a zero: passa tutto.
        let gate = Gate::new(semantic_gate::GateConfig {
            pesi_dense: 0.5,
            pesi_sparse: 0.5,
            soglia: 0.0,
            budget_ns: 10_000_000,
        });
        let traj_a = traj(4, vec![vec![1.0, 0.0, 0.0, 0.0]]);
        let traj_b = traj(4, vec![vec![1.0, 0.0, 0.0, 0.0]]);
        let deadline = Instant::now() + Duration::from_secs(60);

        let res = gate_crisp_alignment(&gate, &traj_a, &traj_b, 1.0, 0.0, deadline, 2).unwrap();
        assert!(res.is_some());
    }
}
