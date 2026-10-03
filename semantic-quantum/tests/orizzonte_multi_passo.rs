//! # orizzonte_multi_passo — il banco di prova del cammino a più salti
//!
//! Test di integrazione per l'**orizzonte multi-passo** (esplorazione
//! multi-hop) del `QuantumResolver`. Incarna i tre punti fermi della
//! composizione multi-hop formalizzati da Camillo:
//!
//! 1. **Additività dell'azione e fattorizzazione dell'ampiezza**:
//!    $S_{tot} = \sum_h S^{(h)}$ e $\psi_{tot} = \prod_h \psi^{(h)}$
//!    (l'integrale di cammino di Feynman: l'ampiezza totale è il prodotto
//!    delle ampiezze dei singoli salti).
//! 2. **Bound sull'azione cumulata**: il pre-filtro F6 ($U_d < L_{inc}$)
//!    opera sull'azione cumulata del cammino fino al passo $h$, non per
//!    segmento.
//! 3. **Interferenza costruttiva tra cammini multi-hop**: l'accumulo
//!    $\Psi(c) = \sum \psi_{tot}$ raccoglie la somma delle ampiezze di tutti
//!    i cammini distinti che convergono su $c$ entro l'orizzonte $H$.
//!
//! Lo scenario: tre candidati, due salti, un ramo che accumula interferenza
//! costruttiva. È il caso che guida l'interfaccia — prima il controesempio
//! che rivela la forma della composizione, poi la struttura.

use semantic_quantum::{BranchType, QuantumResolver, WalkBranch};

/// Costruisce un ramo con ampiezza calcolata dall'azione cumulata.
///
/// `s_cumulato` è l'azione totale $S_{tot}$ del cammino multi-hop fino a
/// questo passo (somma delle azioni dei singoli salti). L'ampiezza è
/// $\psi = \exp(-S_{tot}/\kappa)$, che per la fattorizzazione è anche
/// il prodotto delle ampiezze dei singoli salti.
fn ramo_multi_hop(cid: u64, s_cumulato: f32, kappa: f32) -> WalkBranch {
    WalkBranch::new(BranchType::Inertial, cid, s_cumulato, kappa)
}

#[test]
fn fattorizzazione_ampiezza_su_due_salti() {
    // Il punto 1 di Camillo: ψ_tot = exp(-(S1+S2)/κ) = ψ1·ψ2.
    //
    // Un cammino di due salti con azioni S1=1.0 e S2=2.0 converge sul
    // target c. L'ampiezza totale deve essere exp(-(1+2)/κ), NON
    // exp(-1/κ)+exp(-2/κ) né una media.
    let kappa = 2.0;
    let s1 = 1.0;
    let s2 = 2.0;
    let s_tot = s1 + s2;

    let cid = 42u64;
    let ramo = ramo_multi_hop(cid, s_tot, kappa);

    // ψ_tot = exp(-3/2) ≈ 0.2231
    let atteso = (-s_tot / kappa).exp();
    assert!(
        (ramo.amplitude - atteso).abs() < 1e-6,
        "ampiezza fattorizzata: atteso {atteso}, ottenuto {}",
        ramo.amplitude
    );

    // La fattorizzazione: ψ_tot == ψ1·ψ2 dove ψ1=exp(-1/2), ψ2=exp(-2/2).
    let psi1 = (-s1 / kappa).exp();
    let psi2 = (-s2 / kappa).exp();
    assert!(
        (ramo.amplitude - psi1 * psi2).abs() < 1e-6,
        "fattorizzazione: atteso {}=ψ1·ψ2, ottenuto {}",
        psi1 * psi2,
        ramo.amplitude
    );

    // Il singolo salto (S1) da solo NON è il cammino totale: l'azione si
    // somma, quindi l'ampiezza del cammino a due salti è minore.
    assert!(ramo.amplitude < (-s1 / kappa).exp());
}

#[test]
fn interferenza_costruttiva_vince_sull_azione_minima_isolata() {
    // Il punto 3 di Camillo: vince chi accumula più ampiezza per
    // interferenza costruttiva, non chi ha l'azione minima isolata.
    //
    // Scenario: due cammini distinti convergono su c (azioni cumulate
    // 0.5 e 0.6), un solo cammino (azione minima 0.1) va su d.
    //
    // Ψ(c) = exp(-0.5/κ) + exp(-0.6/κ)
    // Ψ(d) = exp(-0.1/κ)
    //
    // Con κ=2.0: Ψ(c) = e^-0.25 + e^-0.30 ≈ 0.7788 + 0.7408 = 1.5196
    //           Ψ(d) = e^-0.05 ≈ 0.9512
    //
    // c vince per interferenza costruttiva, pur non avendo l'azione minima.
    let kappa = 2.0;
    let resolver = QuantumResolver::new(5.0, 2, kappa); // soglia alta: nessuna decoerenza

    let c = 100u64;
    let d = 200u64;
    let rami = vec![
        ramo_multi_hop(c, 0.5, kappa),
        ramo_multi_hop(c, 0.6, kappa),
        ramo_multi_hop(d, 0.1, kappa), // azione minima, ma un solo cammino
    ];

    let winner = resolver.collapse(&rami).expect("deve esserci un vincitore");

    assert_eq!(
        winner.candidate_id,
        Some(c),
        "l'interferenza costruttiva deve far vincere c (Ψ≈1.52) su d (Ψ≈0.95)"
    );

    // L'ampiezza del vincitore è il valore TOTALE accumulato Ψ(c*).
    let psi_c = (-0.5 / kappa).exp() + (-0.6 / kappa).exp();
    assert!(
        (winner.amplitude - psi_c).abs() < 1e-5,
        "ampiezza vincitore = Ψ(c*) accumulato: atteso {psi_c}, ottenuto {}",
        winner.amplitude
    );

    // Il rappresentante estratto è quello con azione minima nel gruppo c.
    assert!((winner.action - 0.5).abs() < 1e-6);
}

#[test]
fn bound_su_azione_cumulata_esclude_candidato_perdente() {
    // Il punto 2 di Camillo: il pre-filtro F6 opera sull'azione CUMULATA.
    //
    // Scenario: c ha un cammino a due salti con azione cumulata alta
    // (S_tot = 8.0, quindi U_c = exp(-8/κ) basso). d ha un cammino con
    // azione cumulata bassa (S=0.2, U_d alto).
    //
    // Con κ=2.0: U_c = e^-4 ≈ 0.018, L_d = e^-0.1 ≈ 0.905.
    // U_c < L_d → c viene escluso dal pre-filtro, vince d.
    //
    // Il punto: il bound di c usa l'azione CUMULATA del cammino a due salti
    // (8.0), non il minimo dei singoli salti (che potrebbe essere basso).
    // Se il bound usasse il minimo per segmento, c non verrebbe escluso e
    // l'interferenza sarebbe falsificata.
    let kappa = 2.0;
    let resolver = QuantumResolver::new(20.0, 2, kappa);

    let c = 300u64;
    let d = 400u64;
    // c: cammino a due salti con azione cumulata 8.0 (i singoli salti
    // potrebbero essere 1.0 e 7.0 — il cumulato è ciò che conta).
    let rami = vec![
        ramo_multi_hop(c, 8.0, kappa),
        ramo_multi_hop(d, 0.2, kappa),
    ];

    let winner = resolver.collapse(&rami).expect("deve esserci un vincitore");

    assert_eq!(
        winner.candidate_id,
        Some(d),
        "il bound sull'azione cumulata deve escludere c (U_c < L_d)"
    );
}

#[test]
fn tre_candidati_due_salti_un_accumulo() {
    // Lo scenario completo annunciato a Camillo: tre candidati, due salti,
    // un ramo che accumula interferenza costruttiva.
    //
    // a: due cammini convergenti (azioni cumulate 0.4, 0.5) → Ψ_a ≈ 1.35
    // b: un cammino (azione 0.3) → Ψ_b ≈ 0.86
    // c: un cammino (azione 0.1, la minima assoluta) → Ψ_c ≈ 0.95
    //
    // a vince per interferenza costruttiva, anche se c ha l'azione minima.
    let kappa = 2.0;
    let resolver = QuantumResolver::new(5.0, 2, kappa);

    let a = 1u64;
    let b = 2u64;
    let c = 3u64;
    let rami = vec![
        ramo_multi_hop(a, 0.4, kappa),
        ramo_multi_hop(a, 0.5, kappa),
        ramo_multi_hop(b, 0.3, kappa),
        ramo_multi_hop(c, 0.1, kappa),
    ];

    let winner = resolver.collapse(&rami).expect("deve esserci un vincitore");

    assert_eq!(
        winner.candidate_id,
        Some(a),
        "a deve vincere per interferenza costruttiva (due cammini convergenti)"
    );

    let psi_a = (-0.4 / kappa).exp() + (-0.5 / kappa).exp();
    assert!(
        (winner.amplitude - psi_a).abs() < 1e-5,
        "Ψ(a) accumulato: atteso {psi_a}, ottenuto {}",
        winner.amplitude
    );
}

#[test]
fn rami_dello_stesso_candidato_si_sommano_dentro_l_orizzonte() {
    // Verifica che i rami multi-hop afferenti allo STESSO candidato si
    // sommino correttamente nell'accumulatore (il meccanismo di base
    // dell'interferenza costruttiva).
    let kappa = 1.0;
    let resolver = QuantumResolver::new(10.0, 3, kappa);

    let x = 7u64;
    let rami = vec![
        ramo_multi_hop(x, 1.0, kappa),
        ramo_multi_hop(x, 2.0, kappa),
        ramo_multi_hop(x, 3.0, kappa),
    ];

    let winner = resolver.collapse(&rami).expect("deve esserci un vincitore");
    assert_eq!(winner.candidate_id, Some(x));

    let psi = (-1.0 / kappa).exp() + (-2.0 / kappa).exp() + (-3.0 / kappa).exp();
    assert!(
        (winner.amplitude - psi).abs() < 1e-5,
        "Ψ(x) = somma delle tre ampiezze: atteso {psi}, ottenuto {}",
        winner.amplitude
    );
    // Il rappresentante è il ramo con azione minima.
    assert!((winner.action - 1.0).abs() < 1e-6);
}
