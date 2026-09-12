//! # semantic-quantum
//!
//! La stanza del **collapse quantistico** del progetto `semantic-geo`.
//!
//! Dove la navigazione dello spazio semantico smette di essere una ricerca
//! deterministica (Dijkstra, A*) e diventa un **integrale di cammino** alla
//! Feynman: si generano rami concorrenti in sovrapposizione di stato, ognuno
//! con la propria azione $S$ e ampiezza $\psi$, e si lascia che la coerenza
//! decida quale traiettoria emerge.
//!
//! ## La metafora che si fa carne
//!
//! Il cuore del crate è il [`QuantumResolver::collapse`], che implementa il
//! contratto a quattro passaggi:
//!
//! 1. **Filtro di decoerenza** — un ramo la cui azione supera
//!    `divergence_threshold` non è solo penalizzato: *cessa di esistere*
//!    ($\psi_i = 0$).
//! 2. **Accumulo in HashMap** — per ogni ramo valido afferente a un
//!    `candidate_id`, l'ampiezza $\psi_i = \exp(-S_i / \kappa_{break})$
//!    incrementa l'accumulatore $\Psi(c)$.
//! 3. **Selezione per massimo** — il candidato vincente è
//!    $c^* = \arg\max_c \Psi(c)$: prevalgono i nodi verso cui convergono più
//!    cammini coerenti (interferenza costruttiva).
//! 4. **Estrazione del ramo vincitore** — nel gruppo afferente a $c^*$ si
//!    estrae il rappresentante con azione minima, sovrascrivendo la sua
//!    ampiezza con il valore totale accumulato $\Psi(c^*)$.
//!
//! L'azione complessiva di un ramo integra i tre canali pesati:
//!
//! $$S_i = w_I \cdot S_{Inertial} + w_G \cdot S_{Geometric} + w_C \cdot S_{SemanticColbert}$$
//!
//! La stessa filosofia del combinatore trivettoriale, portata dal punto al
//! cammino.

use std::collections::HashMap;

use std::sync::Arc;
use semantic_colbert::{ColbertScorer, NativeMaxSim};
use semantic_combiner::FactId;
use semantic_colbert::maxsim;

/// Stadio di potatura per dominanza di Pareto (modulo di Camillo).
///
/// Scarta i rami inefficienti (dominati su tutti e tre gli assi di costo)
/// prima del collasso quantistico.
pub mod pareto;

/// I tre canali che alimentano l'azione di un ramo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BranchType {
    /// Derivato dalle derivate del moto (velocity, acceleration, curvature)
    /// di [`KinematicState`](semantic_walk::KinematicState).
    Inertial,
    /// Pesato sulla distanza metrica e densità dei nodi in `semantic-graph`.
    Geometric,
    /// Calcolato tramite il punteggio di similarità token-by-token `maxsim`
    /// di `semantic-colbert`.
    SemanticColbert,
}

/// Un ramo concorrente in sovrapposizione di stato.
#[derive(Debug, Clone)]
pub struct WalkBranch {
    /// Il canale che ha generato il ramo.
    pub branch_type: BranchType,
    /// Il nodo candidato a cui il ramo afferisce.
    pub candidate_id: Option<FactId>,
    /// L'azione quantistica $S$ — costo vettoriale del ramo.
    pub action: f32,
    /// L'ampiezza di probabilità $\psi$.
    pub amplitude: f32,
}

impl WalkBranch {
    /// Crea un ramo con ampiezza calcolata dall'azione.
    ///
    /// L'ampiezza è $\psi = \exp(-S / \kappa_{break})$: un'azione minore
    /// produce un'ampiezza maggiore (la traiettoria più economica è la più
    /// probabile, in accordo con l'integrale di cammino).
    pub fn new(branch_type: BranchType, candidate_id: FactId, action: f32, kappa_break: f32) -> Self {
        let amplitude = (-action / kappa_break).exp();
        Self { branch_type, candidate_id: Some(candidate_id), action, amplitude }
    }
}

/// Assembla i tre canali in un'azione totale omogenea.
///
/// Traduce i punteggi dei tre canali (inerziale, geometrico, colbert) in
/// contributi omogenei di *costo* $S_i$, che il [`QuantumResolver::collapse`]
/// potrà confrontare e sommare.
///
/// La formula (formalizzata da Camillo):
///
/// $$S_i = w_I \cdot S_{Inertial} + w_G \cdot S_{Geometric} + w_C \cdot (1 - \text{MaxSim})$$
///
/// Il canale colbert è **invertito** ($1 - \text{MaxSim}$) perché restituisce
/// una similarità in $[0, 1]$ mentre l'azione è un costo: più i token sono
/// simili, minore è il contributo di costo.
#[derive(Debug)]
pub struct BranchBuilder {
    /// Pesi dell'azione totale: `(w_I, w_G, w_C)`.
    pub weights: (f32, f32, f32),
    /// Lo scorer di Late Interaction per il canale colbert.
    ///
    /// Iniettato come `Arc<dyn ColbertScorer + Send + Sync>` per permettere
    /// lo scambio trasparente tra `NativeMaxSim` (fallback/test) e
    /// `CrispEmbedScorer` (motore C++ in produzione) senza accoppiare il
    /// builder al motore specifico.
    pub colbert_scorer: Arc<dyn ColbertScorer + Send + Sync>,
}

impl Default for BranchBuilder {
    fn default() -> Self {
        // Pesi calibrati con λ = 10.64 (criterio p95 satura al 90%):
        // w_dense = 0.215, w_sparse = 0.552, w_colbert = 0.233.
        Self {
            weights: (0.215, 0.552, 0.233),
            colbert_scorer: Arc::new(NativeMaxSim),
        }
    }
}

impl BranchBuilder {
    /// Crea un builder con i tre pesi dei canali e lo scorer colbert.
    pub fn new(weights: (f32, f32, f32)) -> Self {
        Self {
            weights,
            colbert_scorer: Arc::new(NativeMaxSim),
        }
    }

    /// Crea un builder con pesi e scorer colbert espliciti.
    pub fn with_scorer(
        weights: (f32, f32, f32),
        colbert_scorer: Arc<dyn ColbertScorer + Send + Sync>,
    ) -> Self {
        Self { weights, colbert_scorer }
    }

    /// Calcola il costo colbert $S_{SemanticColbert} = 1 - \text{MaxSim}$.
    ///
    /// Se lo scorer ritorna `NaN` (input invalidi o motore non disponibile),
    /// il costo diventa la **penalità massima** $1.0$: il ramo sopravvive ma
    /// con costo massimo, spinto verso la decoerenza nel collapse — meglio
    /// un costo ignoto trattato come massimo che un costo falso.
    pub fn compute_colbert_cost(&self, query_tokens: &[&[f64]], doc_tokens: &[&[f64]]) -> f32 {
        let maxsim = self.colbert_scorer.compute_maxsim(query_tokens, doc_tokens);
        if maxsim.is_nan() {
            1.0
        } else {
            (1.0 - maxsim as f32).clamp(0.0, 1.0)
        }
    }

    /// Costruisce un ramo calcolando il canale colbert dai token.
    ///
    /// Variante di [`Self::build_branch`] che riceve le matrici di token
    /// (query e documento) e calcola internamente il MaxSim tramite lo
    /// scorer iniettato, con fallback `NaN → 1.0`. Utile quando il chiamante
    /// ha accesso ai token ma non vuole (o non deve) sapere come calcolare
    /// il punteggio di Late Interaction.
    pub fn build_branch_from_tokens(
        &self,
        branch_type: BranchType,
        candidate_id: FactId,
        s_inertial: f32,
        s_geometric: f32,
        query_tokens: &[&[f64]],
        doc_tokens: &[&[f64]],
        kappa_break: f32,
    ) -> WalkBranch {
        let s_colbert = self.compute_colbert_cost(query_tokens, doc_tokens);
        self.build_branch(branch_type, candidate_id, s_inertial, s_geometric, 1.0 - s_colbert, kappa_break)
    }

    /// Costruisce un ramo con azione totale omogenea dai tre contributi.
    ///
    /// * `s_inertial` — l'azione inerziale $S_{Inertial}$ da
    ///   [`KinematicState::inertial_action`](semantic_walk::KinematicState).
    /// * `s_geometric` — il costo geometrico (distanza metrica, densità).
    /// * `colbert_similarity` — il punteggio `MaxSim` in $[0, 1]$; viene
    ///   invertito in un costo prima dell'integrazione.
    ///
    /// L'ampiezza è calcolata dall'azione come in [`WalkBranch::new`]:
    /// $\psi = \exp(-S / \kappa_{break})$, così che il ramo possa essere
    /// accumulato dal [`QuantumResolver`] in fase di collapse.
    pub fn build_branch(
        &self,
        branch_type: BranchType,
        candidate_id: FactId,
        s_inertial: f32,
        s_geometric: f32,
        colbert_similarity: f32,
        kappa_break: f32,
    ) -> WalkBranch {
        let s_colbert = (1.0 - colbert_similarity).max(0.0);
        let (w_i, w_g, w_c) = self.weights;
        let total_action = w_i * s_inertial + w_g * s_geometric + w_c * s_colbert;
        let amplitude = (-total_action / kappa_break).exp();
        WalkBranch {
            branch_type,
            candidate_id: Some(candidate_id),
            action: total_action,
            amplitude,
        }
    }
}

/// Il risolutore che esegue il collapse dello stato.
#[derive(Debug, Clone)]
pub struct QuantumResolver {
    /// Soglia massima di deviazione tollerata: oltre questa azione, il ramo
    /// subisce decoerenza immediata ($\psi_i = 0$).
    pub divergence_threshold: f32,
    /// Profondità dei passaggi futuri (orizzonte di esplorazione).
    pub horizon: usize,
    /// Parametro di rottura/damping: scala la conversione azione → ampiezza.
    pub kappa_break: f32,
}

impl QuantumResolver {
    pub fn new(divergence_threshold: f32, horizon: usize, kappa_break: f32) -> Self {
        Self { divergence_threshold, horizon, kappa_break }
    }

    /// Valuta un ramo colbert tramite `maxsim` token-by-token.
    pub fn evaluate_colbert_branch(
        &self,
        query_tokens: &[&[f64]],
        doc_tokens: &[&[f64]],
    ) -> f64 {
        maxsim(query_tokens, doc_tokens)
    }

    /// Esegue il **collapse** dello stato multi-ramo.
    ///
    /// Implementa il contratto a quattro passaggi confermato da Camillo:
    ///
    /// 1. **Filtro di decoerenza**: se $S_i > \text{divergence\_threshold}$,
    ///    l'ampiezza del ramo si azzera.
    /// 2. **Accumulo in HashMap**: per ogni ramo valido afferente a $c$,
    ///    $\psi_i = \exp(-S_i / \kappa_{break})$ incrementa $\Psi(c)$.
    /// 3. **Selezione per massimo**: $c^* = \arg\max_c \Psi(c)$ — l'interferenza
    ///    costruttiva decide, non l'azione minima isolata.
    /// 4. **Estrazione del ramo vincitore**: nel gruppo afferente a $c^*$ si
    ///    estrae il rappresentante con azione minima, sovrascrivendo la sua
    ///    ampiezza con il valore totale accumulato $\Psi(c^*)$.
    ///
    /// Ritorna `None` se non c'è alcun ramo valido (tutti decoerenti, o lista
    /// vuota). I rami senza `candidate_id` vengono ignorati: un ramo senza
    /// destinazione non può collassare su nulla.
    pub fn collapse(&self, branches: &[WalkBranch]) -> Option<WalkBranch> {
        // Passaggio 1+2: filtro di decoerenza + accumulo ampiezze per candidato.
        let mut accum: HashMap<FactId, f32> = HashMap::new();
        for b in branches {
            let Some(cid) = b.candidate_id else { continue };
            // Filtro di decoerenza: oltre la soglia il ramo cessa di esistere.
            if b.action > self.divergence_threshold {
                continue;
            }
            // L'ampiezza del ramo è già stata calcolata alla creazione
            // (exp(-S/κ)); qui la accumuliamo sul candidato.
            let amp = b.amplitude;
            *accum.entry(cid).or_insert(0.0) += amp;
        }

        // Passaggio 3: selezione per massimo Ψ(c) — interferenza costruttiva.
        let winner_id = accum
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| *id)?;

        let total_psi = accum[&winner_id];

        // Passaggio 4: nel gruppo afferente al vincitore, estrai il
        // rappresentante con azione minima, sovrascrivendo la sua ampiezza
        // con il valore totale accumulato Ψ(c*).
        branches
            .iter()
            .filter(|b| b.candidate_id == Some(winner_id))
            .min_by(|a, b| a.action.partial_cmp(&b.action).unwrap_or(std::cmp::Ordering::Equal))
            .map(|b| {
                let mut winner = b.clone();
                winner.amplitude = total_psi;
                winner
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolver() -> QuantumResolver {
        QuantumResolver::new(0.5, 3, 2.0)
    }

    fn branch(bt: BranchType, cid: FactId, action: f32) -> WalkBranch {
        WalkBranch::new(bt, cid, action, 2.0)
    }

    #[test]
    fn test_quantum_colbert_integration() {
        let resolver = QuantumResolver::new(0.3, 3, 2.0);
        let q = vec![vec![1.0, 0.0]];
        let d = vec![vec![1.0, 0.0]];
        let q_ref: Vec<&[f64]> = q.iter().map(|v| v.as_slice()).collect();
        let d_ref: Vec<&[f64]> = d.iter().map(|v| v.as_slice()).collect();

        let score = resolver.evaluate_colbert_branch(&q_ref, &d_ref);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn collapse_sceglie_interferenza_costruttiva() {
        // Due rami a media azione convergono sullo stesso candidato (id 1):
        // la loro ampiezza si somma. Un singolo ramo a bassa azione (id 2)
        // da solo ha ampiezza minore. L'interferenza costruttiva deve vincere.
        let resolver = resolver();
        let branches = vec![
            branch(BranchType::Inertial, 1, 0.3),       // ψ = e^-0.15 ≈ 0.861
            branch(BranchType::Geometric, 1, 0.4),      // ψ = e^-0.20 ≈ 0.819
            branch(BranchType::SemanticColbert, 2, 0.1), // ψ = e^-0.05 ≈ 0.951 (singolo)
        ];
        let winner = resolver.collapse(&branches).expect("deve collassare");
        assert_eq!(winner.candidate_id, Some(1));
        // Ψ(1) = 0.861 + 0.819 ≈ 1.680 > 0.951
        assert!((winner.amplitude - 1.680).abs() < 1e-2);
    }

    #[test]
    fn collapse_estrae_azione_minima_nel_gruppo_vincente() {
        // Il candidato 1 vince per interferenza; nel suo gruppo il
        // rappresentante è quello con azione minima (0.2), non il più recente.
        let resolver = resolver();
        let branches = vec![
            branch(BranchType::Inertial, 1, 0.2),
            branch(BranchType::Geometric, 1, 0.45),
            branch(BranchType::SemanticColbert, 1, 0.35),
        ];
        let winner = resolver.collapse(&branches).expect("deve collassare");
        assert_eq!(winner.candidate_id, Some(1));
        assert!((winner.action - 0.2).abs() < 1e-6);
        // Ampiezza sovrascritta col totale accumulato.
        let expected_psi: f32 = (-0.2_f32 / 2.0).exp() + (-0.45_f32 / 2.0).exp() + (-0.35_f32 / 2.0).exp();
        assert!((winner.amplitude - expected_psi).abs() < 1e-4);
    }

    #[test]
    fn collapse_decoerenza_azzeramento() {
        // Tutti i rami oltre la soglia: nessun candidato valido → None.
        let resolver = resolver();
        let branches = vec![
            branch(BranchType::Inertial, 1, 0.6),
            branch(BranchType::Geometric, 2, 0.7),
        ];
        assert!(resolver.collapse(&branches).is_none());
    }

    #[test]
    fn collapse_ignora_rami_senza_candidato() {
        let resolver = resolver();
        let mut b = branch(BranchType::Inertial, 1, 0.1);
        b.candidate_id = None;
        let branches = vec![b];
        assert!(resolver.collapse(&branches).is_none());
    }

    #[test]
    fn collapse_lista_vuota() {
        let resolver = resolver();
        assert!(resolver.collapse(&[]).is_none());
    }

    #[test]
    fn collapse_decoerenza_parziale() {
        // Un ramo oltre soglia (decoerente) e uno valido: vince quello valido.
        let resolver = resolver();
        let branches = vec![
            branch(BranchType::Inertial, 1, 0.9),  // decoerente, ignorato
            branch(BranchType::Geometric, 2, 0.1), // valido
        ];
        let winner = resolver.collapse(&branches).expect("deve collassare");
        assert_eq!(winner.candidate_id, Some(2));
    }

    // ============================================================
    // BranchBuilder — assemblaggio dell'azione dai tre canali
    // ============================================================

    fn builder() -> BranchBuilder {
        BranchBuilder::new((0.4, 0.3, 0.3))
    }

    #[test]
    fn builder_inverte_il_canale_colbert() {
        // MaxSim = 1.0 (perfetta similarità) → costo colbert 0.0.
        let b = builder().build_branch(BranchType::SemanticColbert, 1, 0.0, 0.0, 1.0, 2.0);
        assert!((b.action - 0.0).abs() < 1e-6);
        // MaxSim = 0.0 (nessuna similarità) → costo colbert 1.0.
        let b = builder().build_branch(BranchType::SemanticColbert, 1, 0.0, 0.0, 0.0, 2.0);
        assert!((b.action - 0.3).abs() < 1e-6); // w_C * 1.0
    }

    #[test]
    fn builder_integra_i_tre_canali_pesati() {
        // S_i = 0.4*2 + 0.3*1 + 0.3*(1-0.5) = 0.8 + 0.3 + 0.15 = 1.25
        let b = builder().build_branch(BranchType::Inertial, 1, 2.0, 1.0, 0.5, 2.0);
        assert!((b.action - 1.25).abs() < 1e-6);
        assert_eq!(b.branch_type, BranchType::Inertial);
        assert_eq!(b.candidate_id, Some(1));
        // L'ampiezza è calcolata dall'azione: ψ = e^(-1.25/2) ≈ 0.535
        let expected = (-1.25_f32 / 2.0).exp();
        assert!((b.amplitude - expected).abs() < 1e-4);
    }

    #[test]
    fn builder_clampa_il_costo_colbert_a_zero() {
        // MaxSim leggermente oltre 1.0 (rumore numerico): il costo non deve
        // diventare negativo — l'azione è un costo, mai un guadagno.
        let b = builder().build_branch(BranchType::SemanticColbert, 1, 0.0, 0.0, 1.05, 2.0);
        assert!(b.action >= 0.0);
    }

    #[test]
    fn builder_da_token_usa_lo_scorer_nativo() {
        // Token identici → MaxSim = 1.0 → costo colbert 0.0.
        let tok: Vec<Vec<f64>> = vec![vec![1.0, 0.0], vec![0.5, 0.5]];
        let q: Vec<&[f64]> = tok.iter().map(|v| v.as_slice()).collect();
        let d = q.clone();
        let b = builder().build_branch_from_tokens(BranchType::SemanticColbert, 1, 0.0, 0.0, &q, &d, 2.0);
        assert!((b.action - 0.0).abs() < 1e-4);
    }

    #[test]
    fn builder_da_token_fallback_nan_a_penalita_massima() {
        // Uno scorer che ritorna sempre NaN (motore non disponibile):
        // il costo colbert deve diventare la penalità massima 1.0.
        struct AlwaysNan;
        impl std::fmt::Debug for AlwaysNan {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("AlwaysNan")
            }
        }
        impl ColbertScorer for AlwaysNan {
            fn compute_maxsim(&self, _q: &[&[f64]], _d: &[&[f64]]) -> f64 {
                f64::NAN
            }
        }
        let b = BranchBuilder::with_scorer((0.4, 0.3, 0.3), Arc::new(AlwaysNan));
        let tok: Vec<Vec<f64>> = vec![vec![1.0, 0.0]];
        let q: Vec<&[f64]> = tok.iter().map(|v| v.as_slice()).collect();
        // w_C * 1.0 = 0.3
        let br = b.build_branch_from_tokens(BranchType::SemanticColbert, 1, 0.0, 0.0, &q, &q, 2.0);
        assert!((br.action - 0.3).abs() < 1e-6);
    }

    #[test]
    fn builder_da_token_coerente_con_assemblatore_puro() {
        // Il percorso con token deve produrre lo stesso ramo del percorso
        // puro quando il MaxSim è noto a priori.
        let b = builder();
        let tok: Vec<Vec<f64>> = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let q: Vec<&[f64]> = tok.iter().map(|v| v.as_slice()).collect();
        let d = q.clone();
        let from_tokens = b.build_branch_from_tokens(BranchType::Inertial, 1, 2.0, 1.0, &q, &d, 2.0);
        // MaxSim nativo di token identici = 1.0 → colbert_similarity = 1.0
        let direct = b.build_branch(BranchType::Inertial, 1, 2.0, 1.0, 1.0, 2.0);
        assert!((from_tokens.action - direct.action).abs() < 1e-4);
    }

    // ============================================================
    // Integrazione end-to-end builder → collapse
    // (piano di verifica di Camillo)
    // ============================================================

    #[test]
    fn end_to_end_decoerenza_da_strappo_cinematico() {
        // Traiettoria con deviazione elevata (Δκ ≫ 0): l'azione inerziale
        // porta il ramo oltre divergence_threshold → scarto nel collapse.
        let resolver = QuantumResolver::new(0.5, 3, 2.0);
        let b = builder();
        // S_inertial alta (strappo cinematico) + geometrica moderata:
        // 0.4*2.0 + 0.3*0.2 + 0.3*(1-0.1) = 0.8 + 0.06 + 0.27 = 1.13 > 0.5
        let branch = b.build_branch(BranchType::Inertial, 7, 2.0, 0.2, 0.9, 2.0);
        let winner = resolver.collapse(&[branch]);
        assert!(winner.is_none(), "il ramo deviante deve essere scartato");
    }

    #[test]
    fn end_to_end_compensazione_semantica() {
        // Rami con alta affinità semantica (MaxSim ≈ 1.0) riescono a
        // bilanciare una lieve deviazione inerziale: il ramo resta attivo
        // nell'accumulo di ampiezza Ψ(c).
        let resolver = QuantumResolver::new(0.5, 3, 2.0);
        let b = builder();
        // S_inertial lieve (0.3) + geometrica nulla + colbert quasi perfetto:
        // 0.4*0.3 + 0.3*0.0 + 0.3*(1-0.95) = 0.12 + 0 + 0.015 = 0.135 < 0.5
        let branch = b.build_branch(BranchType::SemanticColbert, 7, 0.3, 0.0, 0.95, 2.0);
        let winner = resolver.collapse(&[branch]).expect("il ramo deve restare attivo");
        assert_eq!(winner.candidate_id, Some(7));
        // L'ampiezza è stata calcolata dal resolver: ψ = e^(-0.135/2) ≈ 0.935
        let expected = (-0.135_f32 / 2.0).exp();
        assert!((winner.amplitude - expected).abs() < 1e-3);
    }

    proptest::proptest! {
        /// Proprietà fondamentale: il collapse non restituisce mai un ramo
        /// decoerente (azione oltre la soglia) come vincitore.
        ///
        /// È il contratto più importante: la decoerenza ($\psi_i = 0$) deve
        /// escludere i rami devianti, qualunque sia la loro distribuzione.
        #[test]
        fn collapse_mai_decoerente(
            actions in proptest::collection::vec(0.0_f32..1.0, 0..8),
            threshold in 0.1_f32..0.9,
        ) {
            let resolver = QuantumResolver::new(threshold, 3, 2.0);
            let branches: Vec<WalkBranch> = actions
                .iter()
                .enumerate()
                .map(|(i, &a)| branch(BranchType::Inertial, (i % 3) as FactId, a))
                .collect();
            if let Some(winner) = resolver.collapse(&branches) {
                // Il vincitore deve avere azione entro soglia (non decoerente).
                assert!(winner.action <= threshold);
                // E la sua ampiezza deve essere la somma accumulata, quindi
                // maggiore di qualunque singola ampiezza valida.
                let max_single: f32 = branches
                    .iter()
                    .filter(|b| b.action <= threshold)
                    .map(|b| b.amplitude)
                    .fold(0.0_f32, f32::max);
                assert!(winner.amplitude >= max_single - 1e-4);
            }
        }

        /// Proprietà di convergenza: più rami sullo stesso candidato non
        /// possono mai *ridurre* la sua ampiezza accumulata rispetto a un
        /// singolo ramo — l'accumulo è additivo.
        #[test]
        fn collapse_accumulo_additivo(
            actions in proptest::collection::vec(0.01_f32..0.4, 1..6),
        ) {
            let resolver = QuantumResolver::new(0.5, 3, 2.0);
            let branches: Vec<WalkBranch> = actions
                .iter()
                .map(|&a| branch(BranchType::Inertial, 42, a))
                .collect();
            let winner = resolver.collapse(&branches).expect("deve collassare");
            let expected: f32 = actions.iter().map(|&a| (-a / 2.0).exp()).sum();
            assert!((winner.amplitude - expected).abs() < 1e-3);
        }
    }

    /// End-to-end: il pruning di Pareto (modulo di Camillo) precede il collapse
    /// e ne riduce il numero di rami senza alterare il vincitore.
    ///
    /// I rami dominati su tutti e tre gli assi vengono scartati *prima* del
    /// collapse, che riceve solo la frontiera di Pareto. Il vincitore del
    /// collapse sui rami potati deve coincidere con quello sui rami completi.
    #[test]
    fn end_to_end_pareto_pruning_prima_del_collapse() {
        use crate::pareto::{estrai_frontiera_pareto, BranchCostVector};

        let resolver = QuantumResolver::new(0.5, 3, 2.0);
        let b = builder();

        // Tre candidati:
        //  - c1: azione totale minima (0.135) — domina su tutti gli assi
        //  - c2: dominato da c1 su tutti gli assi
        //  - c3: incomparabile con c1 (migliore su inerziale, peggiore su colbert)
        let r1 = b.build_branch(BranchType::Inertial, 1, 0.3, 0.0, 0.95, 2.0);
        let r2 = b.build_branch(BranchType::Inertial, 2, 0.6, 0.3, 0.5, 2.0);
        let r3 = b.build_branch(BranchType::Inertial, 3, 0.1, 0.9, 0.2, 2.0);

        let costs = vec![
            BranchCostVector::new(r1, 0.3, 0.0, 0.05),
            BranchCostVector::new(r2, 0.6, 0.3, 0.5),
            BranchCostVector::new(r3, 0.1, 0.9, 0.8),
        ];

        // Il pruning scarta il ramo dominato (c2).
        let front = estrai_frontiera_pareto(&costs);
        let ids: Vec<FactId> = front
            .iter()
            .map(|c| c.branch.candidate_id.unwrap())
            .collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
        assert!(!ids.contains(&2));

        // Il collapse sui rami potati seleziona lo stesso vincitore dei rami completi.
        let pruned_branches: Vec<WalkBranch> = front.into_iter().map(|c| c.branch).collect();
        let winner_pruned = resolver.collapse(&pruned_branches).expect("deve collassare");
        let winner_full = resolver.collapse(&costs.iter().map(|c| c.branch.clone()).collect::<Vec<_>>())
            .expect("deve collassare");

        assert_eq!(winner_pruned.candidate_id, winner_full.candidate_id);
    }

    /// **Test di integrazione finale** (proposto da Camillo, 13/09/26):
    /// collega il `KinematicState` di semantic-walk al nuovo AdaptiveGate.
    ///
    /// Il flusso completo è:
    ///
    /// 1. **KinematicState::inertial_action** — genera l'azione inerziale
    ///    $S_{Inertial}$ tra stati successivi della traiettoria.
    /// 2. **BranchBuilder::build_branch** — assembla i tre canali (inerziale,
    ///    geometrico, colbert) in azione totale omogenea.
    /// 3. **estrai_frontiera_pareto_adattivo** — l'AdaptiveGate a due vie
    ///    decide se potare (FullPareto) o lasciar passare tutto
    ///    (DirectCollapse) in base a soglia e varianza.
    /// 4. **QuantumResolver::collapse** — il collasso quantistico seleziona
    ///    il vincitore per interferenza costruttiva.
    ///
    /// Il test verifica che l'AdaptiveGate, alimentato dalle azioni inerziali
    /// *reali* del KinematicState, produca lo stesso vincitore del flusso
    /// completo senza pruning — e che i rami dominati vengano scartati.
    #[test]
    fn end_to_end_kinematic_state_adattivo() {
        use crate::pareto::{
            estrai_frontiera_pareto_adattivo, BranchCostVector, SOGLIA_ADATTIVA_DEFAULT,
            VARIANZA_DIRECT_DEFAULT,
        };
        use semantic_walk::KinematicState;

        let resolver = QuantumResolver::new(0.5, 3, 2.0);
        let b = builder();

        // Tre traiettorie candidate, ognuna con uno stato cinematico iniziale
        // e uno successivo. Le azioni inerziali misurano la penalità di
        // deviazione (alpha=1.0, beta=0.5, gamma=2.0):
        //
        //  - c1: cammino quasi rettilineo (Δv=0.1, Δa=0.0, Δκ=0.0)
        //        → S_inertial = 1.0·0.01 + 0.5·0 + 2.0·0 = 0.01  (il più economico)
        //  - c2: scatto di velocità (Δv=0.5, Δa=0.2, Δκ=0.1)
        //        → S_inertial = 1.0·0.25 + 0.5·0.04 + 2.0·0.1 = 0.47
        //  - c3: curva leggera (Δv=0.1, Δa=0.0, Δκ=0.05)
        //        → S_inertial = 1.0·0.01 + 0.5·0 + 2.0·0.05 = 0.11
        //        (migliore di c1 su colbert, peggiore su inerziale → incomparabile)
        let s1 = KinematicState { velocity: 1.0, acceleration: 0.0, curvature: 0.0 };
        let s1_next = KinematicState { velocity: 1.1, acceleration: 0.0, curvature: 0.0 };
        let s2 = KinematicState { velocity: 1.0, acceleration: 0.0, curvature: 0.0 };
        let s2_next = KinematicState { velocity: 1.5, acceleration: 0.2, curvature: 0.1 };
        let s3 = KinematicState { velocity: 1.0, acceleration: 0.0, curvature: 0.0 };
        let s3_next = KinematicState { velocity: 1.1, acceleration: 0.0, curvature: 0.05 };

        let (alpha, beta, gamma) = (1.0, 0.5, 2.0);
        let s_inertial_1 = s1.inertial_action(&s1_next, alpha, beta, gamma);
        let s_inertial_2 = s2.inertial_action(&s2_next, alpha, beta, gamma);
        let s_inertial_3 = s3.inertial_action(&s3_next, alpha, beta, gamma);

        // Verifica dei valori attesi (formula di Camillo).
        assert!((s_inertial_1 - 0.01).abs() < 1e-6);
        assert!((s_inertial_2 - 0.47).abs() < 1e-6);
        assert!((s_inertial_3 - 0.11).abs() < 1e-6);

        // I tre canali: c1 domina su inerziale e geometrico, c2 è dominato da
        // c1 su tutti e tre gli assi (inerziale 0.47 > 0.01, geometrico
        // 0.3 > 0.0, colbert 0.5 < 0.95), c3 è incomparabile con c1
        // (migliore su colbert 0.98 > 0.95, peggiore su inerziale 0.11 > 0.01
        // e geometrico 0.1 > 0.0).
        let r1 = b.build_branch(BranchType::Inertial, 1, s_inertial_1, 0.0, 0.95, 2.0);
        let r2 = b.build_branch(BranchType::Inertial, 2, s_inertial_2, 0.3, 0.5, 2.0);
        let r3 = b.build_branch(BranchType::Inertial, 3, s_inertial_3, 0.1, 0.98, 2.0);

        let costs = vec![
            BranchCostVector::new(r1, s_inertial_1, 0.0, 0.05),
            BranchCostVector::new(r2, s_inertial_2, 0.3, 0.5),
            BranchCostVector::new(r3, s_inertial_3, 0.1, 0.02),
        ];

        // L'AdaptiveGate con N=3 (sotto la soglia 256) usa il FullPareto.
        let front = estrai_frontiera_pareto_adattivo(
            &costs,
            SOGLIA_ADATTIVA_DEFAULT,
            VARIANZA_DIRECT_DEFAULT,
        );
        let ids: Vec<FactId> = front
            .iter()
            .map(|c| c.branch.candidate_id.unwrap())
            .collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
        assert!(!ids.contains(&2), "il ramo dominato c2 deve essere scartato");

        // Il collapse sui rami potati seleziona lo stesso vincitore dei rami completi.
        let pruned_branches: Vec<WalkBranch> = front.into_iter().map(|c| c.branch).collect();
        let winner_pruned = resolver.collapse(&pruned_branches).expect("deve collassare");
        let winner_full = resolver
            .collapse(&costs.iter().map(|c| c.branch.clone()).collect::<Vec<_>>())
            .expect("deve collassare");

        assert_eq!(winner_pruned.candidate_id, winner_full.candidate_id);
        assert_eq!(winner_pruned.candidate_id, Some(1), "il cammino più economico vince");
    }
}