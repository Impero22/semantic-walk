//! # Pruning per Dominanza di Pareto (da Camillo)
//!
//! Stadio di **potatura** che precede il collasso quantistico: dato un insieme
//! di rami candidati, ognuno con il proprio vettore di costo sui tre canali
//! $(S_{Inertial}, S_{Geometric}, S_{Colbert})$, mantiene soltanto la
//! **frontiera di Pareto** — i rami non dominati.
//!
//! Un ramo `a` domina `b` quando è minore o uguale su *tutti* gli assi di
//! costo e strettamente minore su *almeno uno*. I rami dominati sono
//! inefficienti: qualunque sia la calibrazione dei pesi, non possono mai
//! vincere il collasso, quindi vengono scartati *prima*.
//!
//! ## Perché è complementare al collapse
//!
//! - Il [`crate::BranchBuilder`] **aggrega** i tre canali in un'azione pesata.
//! - Il modulo presente **pota** sui costi scomposti, prima che i pesi li
//!   schiaccino — robusto alla calibrazione.
//! - Il [`crate::QuantumResolver::collapse`] **collassa** i superstiti,
//!   sommando le ampiezze sui nodi condivisi (interferenza costruttiva).

use crate::WalkBranch;
use semantic_combiner::FactId;

/// Vettore dei costi scomposti dei tre canali per un ramo.
#[derive(Debug, Clone)]
pub struct BranchCostVector {
    pub branch: WalkBranch,
    pub s_inertial: f32,
    pub s_geometric: f32,
    pub s_colbert: f32,
}

impl BranchCostVector {
    pub fn new(
        branch: WalkBranch,
        s_inertial: f32,
        s_geometric: f32,
        s_colbert: f32,
    ) -> Self {
        Self {
            branch,
            s_inertial,
            s_geometric,
            s_colbert,
        }
    }

    /// Ritorna `true` se `self` domina in senso di Pareto `other`.
    ///
    /// Condizione: minore o uguale su tutti e tre gli assi di costo, e
    /// strettamente minore su almeno uno.
    pub fn dominates(&self, other: &Self) -> bool {
        let i_le = self.s_inertial <= other.s_inertial;
        let g_le = self.s_geometric <= other.s_geometric;
        let c_le = self.s_colbert <= other.s_colbert;

        let strict_better = self.s_inertial < other.s_inertial
            || self.s_geometric < other.s_geometric
            || self.s_colbert < other.s_colbert;

        i_le && g_le && c_le && strict_better
    }
}

/// Filtra i rami mantenendo esclusivamente la frontiera di Pareto.
///
/// Un ramo è dominato se esiste un altro ramo che lo domina; i rami non
/// dominati formano la frontiera e vengono restituiti in ordine di input.
///
/// Complessità: $\mathcal{O}(d \cdot N^2)$ — confronto a coppie completo.
/// Per $N$ estesi conviene [`estrai_frontiera_pareto_adattivo`].
pub fn estrai_frontiera_pareto(branches: &[BranchCostVector]) -> Vec<BranchCostVector> {
    let mut pareto_front = Vec::new();

    for (i, candidate) in branches.iter().enumerate() {
        let mut is_dominated = false;
        for (j, other) in branches.iter().enumerate() {
            if i != j && other.dominates(candidate) {
                is_dominated = true;
                break;
            }
        }
        if !is_dominated {
            pareto_front.push(candidate.clone());
        }
    }

    pareto_front
}

/// Varianza totale normalizzata dei tre canali di costo.
///
/// Proxy economico della densità di dominanza del campione. Se i rami sono
/// tutti vicini (varianza bassa), quasi tutti si dominano a vicenda → la
/// frontiera è piccola e il DirectCollapse vince. Se sono sparsi (varianza
/// alta), pochi si dominano → la frontiera è grande e il FullPareto paga,
/// evitando il costo $\mathcal{O}(N)$ del collapse su N esteso.
///
/// La normalizzazione per la media evita che la scala dei costi (arbitraria
/// rispetto alla calibrazione dei pesi) influenzi la decisione.
pub fn varianza_totale_normalizzata(branches: &[BranchCostVector]) -> f32 {
    let n = branches.len();
    if n == 0 {
        return 0.0;
    }
    let mean = |f: &dyn Fn(&BranchCostVector) -> f32| -> f32 {
        branches.iter().map(f).sum::<f32>() / n as f32
    };
    let var = |f: &dyn Fn(&BranchCostVector) -> f32, m: f32| -> f32 {
        branches.iter().map(|b| { let d = f(b) - m; d * d }).sum::<f32>() / n as f32
    };

    let m_i = mean(&|b| b.s_inertial);
    let m_g = mean(&|b| b.s_geometric);
    let m_c = mean(&|b| b.s_colbert);

    let v_i = var(&|b| b.s_inertial, m_i);
    let v_g = var(&|b| b.s_geometric, m_g);
    let v_c = var(&|b| b.s_colbert, m_c);

    let v_tot = v_i + v_g + v_c;
    let m_tot = m_i + m_g + m_c;
    if m_tot > 0.0 {
        v_tot / m_tot
    } else {
        0.0
    }
}

/// Soglia minima di attivazione del pruning (AdaptiveGate).
///
/// Valore dalla proposta di Camillo (13/09/26): sotto questa soglia il
/// FullPareto domina in tutti i regimi di dominanza (N<256) e il confronto a
/// coppie costa poco; sopra, la scelta tra FullPareto e DirectCollapse è
/// guidata dalla densità di dominanza (varianza normalizzata).
pub const SOGLIA_ADATTIVA_DEFAULT: usize = 256;

/// Soglia di varianza normalizzata per il regime DirectCollapse.
///
/// Se la varianza totale normalizzata dei tre canali è sotto questa soglia,
/// i rami sono quasi tutti reciprocamente dominanti: la frontiera è piccola e
/// il pruning non ripaga il confronto a coppie — conviene collassare tutto.
///
/// Valore calibrato sul benchmark (dominance=0.5-0.9 → DirectCollapse vince).
pub const VARIANZA_DIRECT_DEFAULT: f32 = 0.01;

/// Pruning adattivo (AdaptiveGate) — regime a due vie.
///
/// Decide *se* potare in base alla dimensione dell'insieme **e** alla densità
/// di dominanza stimata dalla varianza normalizzata:
/// - $N \le$ soglia: FullPareto (costa comunque poco, guardia di ingresso).
/// - $N >$ soglia e varianza **alta**: FullPareto — i rami sono sparsi, pochi
///   si dominano, la frontiera è grande e il pruning evita il costo del
///   collapse su N esteso.
/// - $N >$ soglia e varianza **bassa**: DirectCollapse (nessuna potatura) —
///   i rami sono quasi tutti reciprocamente dominanti, la frontiera è
///   piccola e il confronto a coppie non ripaga.
///
/// Il discriminante non è solo N: è la **densità di dominanza** del campione,
/// stimata economicamente dalla varianza. Un gate che guardasse solo N
/// sceglierebbe male quando la dominanza è alta ma N è piccolo.
///
/// Il chiamante può scegliere soglia e varianza; i default sono
/// [`SOGLIA_ADATTIVA_DEFAULT`] e [`VARIANZA_DIRECT_DEFAULT`].
pub fn estrai_frontiera_pareto_adattivo(
    branches: &[BranchCostVector],
    soglia: usize,
    varianza_direct: f32,
) -> Vec<BranchCostVector> {
    if branches.len() <= soglia {
        estrai_frontiera_pareto(branches)
    } else if varianza_totale_normalizzata(branches) >= varianza_direct {
        estrai_frontiera_pareto(branches)
    } else {
        // DirectCollapse: nessuna potatura, il chiamante collassa tutto.
        branches.to_vec()
    }
}

// ============================================================
// Pruning post-collapse sui candidati (da Camillo, 22/09/26).
//
// Il pruning branch-level *prima* del collapse è incompatibile con l'obiettivo
// del collapse, che è l'interferenza costruttiva Ψ(c) = Σ exp(-S_r/κ): ogni
// ramo contribuisce all'ampiezza del proprio candidato, e scartare un ramo
// "dominato" mutila la funzione d'onda prima dell'integrazione.
//
// Il pruning di Pareto ha quindi un ruolo legittimo SOLO **dopo** l'aggregazione,
// sui candidati collassati: ogni candidato ha un vettore di costo aggregato
// (es. la media dei costi dei suoi rami, o il costo del suo rappresentante),
// e il Pareto seleziona tra i candidati dominati quelli da scartare — senza
// alterare l'esito del collapse, che è già avvenuto.
//
// ## Distinzione fondamentale
// - **Branch-level (PRIMA del collapse)**: VIETATO. Mutila la funzione d'onda.
// - **Candidate-level (DOPO il collapse)**: LEGITTIMO. Selezione tra candidati
//   già collassati, ortogonale all'argmax del collapse.
// ============================================================

/// Vettore dei costi aggregati per un candidato (post-collapse).
#[derive(Debug, Clone)]
pub struct CandidateCostVector {
    pub candidate_id: FactId,
    /// Costo inerziale aggregato (es. media dei rami del candidato).
    pub s_inertial: f32,
    /// Costo geometrico aggregato.
    pub s_geometric: f32,
    /// Costo colbert aggregato.
    pub s_colbert: f32,
}

impl CandidateCostVector {
    pub fn new(
        candidate_id: FactId,
        s_inertial: f32,
        s_geometric: f32,
        s_colbert: f32,
    ) -> Self {
        Self {
            candidate_id,
            s_inertial,
            s_geometric,
            s_colbert,
        }
    }

    /// Ritorna `true` se `self` domina in senso di Pareto `other`.
    pub fn dominates(&self, other: &Self) -> bool {
        let i_le = self.s_inertial <= other.s_inertial;
        let g_le = self.s_geometric <= other.s_geometric;
        let c_le = self.s_colbert <= other.s_colbert;
        let strict_better = self.s_inertial < other.s_inertial
            || self.s_geometric < other.s_geometric
            || self.s_colbert < other.s_colbert;
        i_le && g_le && c_le && strict_better
    }
}

/// Filtra i candidati collassati mantenendo la frontiera di Pareto.
///
/// Da applicare **dopo** il collapse (vedi nota di testa del modulo). Opera sui
/// costi aggregati per candidato, non sui rami: non tocca l'interferenza
/// costruttiva già computata dal [`crate::QuantumResolver::collapse`].
pub fn estrai_frontiera_pareto_sui_candidati(
    candidates: &[CandidateCostVector],
) -> Vec<CandidateCostVector> {
    let mut pareto_front = Vec::new();
    for (i, candidate) in candidates.iter().enumerate() {
        let mut is_dominated = false;
        for (j, other) in candidates.iter().enumerate() {
            if i != j && other.dominates(candidate) {
                is_dominated = true;
                break;
            }
        }
        if !is_dominated {
            pareto_front.push(candidate.clone());
        }
    }
    pareto_front
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BranchType;
    use semantic_combiner::FactId;

    fn make_cost_vec(id: FactId, s_i: f32, s_g: f32, s_c: f32) -> BranchCostVector {
        let branch = WalkBranch {
            branch_type: BranchType::Inertial,
            candidate_id: Some(id),
            action: s_i + s_g + s_c,
            amplitude: 0.0,
            s_inertial: s_i,
            s_geometric: s_g,
            s_colbert: s_c,
        };
        BranchCostVector::new(branch, s_i, s_g, s_c)
    }

    #[test]
    fn test_pareto_dominanza_semplice() {
        let b1 = make_cost_vec(1, 1.0, 1.0, 1.0); // Domina b2
        let b2 = make_cost_vec(2, 2.0, 2.0, 2.0); // Dominato da b1
        let b3 = make_cost_vec(3, 0.5, 3.0, 1.0); // Incomparabile con b1

        let input = vec![b1, b2, b3];
        let front = estrai_frontiera_pareto(&input);

        assert_eq!(front.len(), 2);
        let ids: Vec<FactId> = front.iter().map(|b| b.branch.candidate_id.unwrap()).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
        assert!(!ids.contains(&2));
    }

    #[test]
    fn test_pareto_doppio_dominato() {
        // b1 domina b2 e b3 su tutti gli assi: solo b1 sopravvive.
        let b1 = make_cost_vec(1, 1.0, 1.0, 1.0);
        let b2 = make_cost_vec(2, 2.0, 2.0, 2.0);
        let b3 = make_cost_vec(3, 3.0, 3.0, 3.0);

        let front = estrai_frontiera_pareto(&[b1, b2, b3]);
        assert_eq!(front.len(), 1);
        assert_eq!(front[0].branch.candidate_id, Some(1));
    }

    #[test]
    fn test_pareto_tutti_incomparabili() {
        // Ogni ramo è migliore su un asse diverso: nessuno domina, tutti restano.
        let b1 = make_cost_vec(1, 0.1, 3.0, 3.0);
        let b2 = make_cost_vec(2, 3.0, 0.1, 3.0);
        let b3 = make_cost_vec(3, 3.0, 3.0, 0.1);

        let front = estrai_frontiera_pareto(&[b1, b2, b3]);
        assert_eq!(front.len(), 3);
    }

    #[test]
    fn test_pareto_vuoto() {
        let front = estrai_frontiera_pareto(&[]);
        assert!(front.is_empty());
    }

    #[test]
    fn test_pareto_adattivo_sotto_soglia_usa_classica() {
        // Sotto la soglia, l'adattivo si comporta come la classica.
        let b1 = make_cost_vec(1, 1.0, 1.0, 1.0);
        let b2 = make_cost_vec(2, 2.0, 2.0, 2.0);
        let b3 = make_cost_vec(3, 0.5, 3.0, 1.0);

        let input = vec![b1, b2, b3];
        let front = estrai_frontiera_pareto_adattivo(&input, 10, VARIANZA_DIRECT_DEFAULT);
        assert_eq!(front.len(), 2);
    }

    #[test]
    fn test_pareto_adattivo_sopra_soglia_varianza_bassa_usa_direct() {
        // Sopra la soglia con varianza bassa (tutti identici → dominanza alta),
        // l'adattivo non pota: DirectCollapse restituisce tutto.
        let mut input = Vec::new();
        for i in 0..70 {
            input.push(make_cost_vec(i as FactId, 1.0, 1.0, 1.0));
        }
        let front = estrai_frontiera_pareto_adattivo(&input, 64, VARIANZA_DIRECT_DEFAULT);
        assert_eq!(front.len(), 70);
    }

    #[test]
    fn test_pareto_adattivo_sopra_soglia_varianza_alta_usa_classica() {
        // Sopra la soglia con varianza alta (rami sparsi → dominanza bassa),
        // l'adattivo pota con la classica: la frontiera esclude i dominati.
        let mut input = Vec::new();
        for i in 0..70 {
            // b_i domina tutti i successivi: frontiera di 1 elemento.
            let v = (70 - i) as f32;
            input.push(make_cost_vec(i as FactId, v, v, v));
        }
        let front = estrai_frontiera_pareto_adattivo(&input, 64, VARIANZA_DIRECT_DEFAULT);
        assert_eq!(front.len(), 1);
    }

    #[test]
    fn test_varianza_totale_normalizzata_discrimina() {
        // Rami sparsi → varianza alta; rami identici → varianza ~0.
        let sparsi = vec![
            make_cost_vec(1, 0.1, 3.0, 3.0),
            make_cost_vec(2, 3.0, 0.1, 3.0),
            make_cost_vec(3, 3.0, 3.0, 0.1),
        ];
        let identici = vec![
            make_cost_vec(1, 1.0, 1.0, 1.0),
            make_cost_vec(2, 1.0, 1.0, 1.0),
            make_cost_vec(3, 1.0, 1.0, 1.0),
        ];
        let v_sparsi = varianza_totale_normalizzata(&sparsi);
        let v_identici = varianza_totale_normalizzata(&identici);
        assert!(v_sparsi > v_identici);
        assert!(v_identici < VARIANZA_DIRECT_DEFAULT);
    }

}