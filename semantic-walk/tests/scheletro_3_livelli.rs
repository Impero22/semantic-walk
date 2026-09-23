//! # Scheletro dei test a 3 livelli — contratto per-token (frames mode)
//!
//! Ancora esplicita per il prossimo riavvio, come concordato con Camillo il
//! 22/09. Verifica i tre livelli del contratto che lega il `walk` ordinato
//! (canale sparso) alla matrice ColBERT per-token (canale denso):
//!
//! * **Level 1 — Parsing JSON**: la deserializzazione della risposta del
//!   server preserva l'array `0..N-1` senza alterare la corrispondenza con le
//!   righe ColBERT.
//! * **Level 2 — Filtro d'igiene**: l'azzeramento del peso sui token d'igiene
//!   (`st == 0 && id >= 4`) lascia intatta la posizione assoluta `i`, in modo
//!   che il DTW mappi la riga `i` corretta. La **terza via** conserva la
//!   topologia posizionale: tutte le `N` posizioni restano, i non significativi
//!   hanno peso `0.0` (verdetto, non assenza).
//! * **Level 3 — Fixture reale**: la sequenza di 7 token ("gatto dorme" con
//!   `<s>` e `</s>`) deve dimostrare che la biiezione `7 == 7` regge: ogni
//!   riga ColBERT corrisponde al passo `i` del walk, con gli speciali a peso
//!   zero.
//!
//! I test sono **scheletri espliciti**: le asserzioni critiche sono marcate
//! `todo!()` e vanno completate quando il frames mode del server sarà
//! deployato e cattureremo la fixture reale.

use semantic_walk::parse::{colbert_to_trajectory, verifica_coerenza_posizionale, RawColbertTrajectory, RawWalk};

// ---------------------------------------------------------------------------
// Level 1 — Parsing JSON: l'array 0..N-1 è preservato
// ---------------------------------------------------------------------------

/// La deserializzazione di una risposta ColBERT per-token deve preservare
/// l'ordine e il numero esatti delle righe: `embeddings[i]` è il vettore del
/// token `i`, senza riordini né dedup.
#[test]
fn level1_parsing_json_preserva_ordine_righe() {
    // Costruiamo una risposta sintetica che imita il contratto del server:
    // N token, N righe, dimensione D costante.
    let n_tokens = 7usize;
    let dim = 4usize;
    let tokens: Vec<String> = (0..n_tokens).map(|i| format!("tok_{i}")).collect();
    let embeddings: Vec<Vec<f64>> = (0..n_tokens)
        .map(|i| (0..dim).map(|d| (i * 10 + d) as f64).collect())
        .collect();

    let raw = RawColbertTrajectory {
        sequence_id: "fixture_l1".into(),
        tokens: tokens.clone(),
        embeddings: embeddings.clone(),
    };

    let traj = colbert_to_trajectory(&raw).expect("traiettoria ben formata");

    // Il contratto: il numero di passi della traiettoria == N. Verificabile
    // ORA per costruzione: `colbert_to_trajectory` conserva l'array 0..N-1
    // senza riordini né dedup, e `CrispTrajectory::len()` conta gli embedding.
    assert_eq!(traj.len(), n_tokens, "N passi della traiettoria == N token");

    // La riga i deve corrispondere al token i: verifichiamo la biiezione
    // attraverso la coerenza posizionale con un walk allineato.
    let walk = RawWalk {
        sequence_id: "fixture_l1".into(),
        ids: (0..n_tokens).map(|i| (i + 4) as u32).collect(),
        w: (0..n_tokens).map(|i| 0.1 + i as f32).collect(),
        pos: (0..n_tokens).map(|i| i as u32).collect(),
        st: vec![0; n_tokens],
    };
    verifica_coerenza_posizionale(&walk, n_tokens)
        .expect("N passi del walk == N righe ColBERT");

    // La riga `i` deserializzata corrisponde al token `i` dichiarato dal
    // payload: `colbert_to_trajectory` conserva l'array `0..N-1` senza riordini
    // né dedup. Lo verifichiamo sugli embedding, costruiti deterministicamente
    // come `embeddings[i][d] = i*10 + d` — la biiezione è provata se ogni riga
    // della traiettoria coincide con quella originale.
    for i in 0..n_tokens {
        let atteso: Vec<f64> = (0..dim).map(|d| (i * 10 + d) as f64).collect();
        assert_eq!(traj.embeddings[i], atteso, "riga {i} della matrice preservata (token {i})");
    }
}

// ---------------------------------------------------------------------------
// Level 2 — Filtro d'igiene: la posizione assoluta è preservata
// ---------------------------------------------------------------------------

/// Il filtro d'igiene (`st == 0 && id >= 4`) azzera il *peso* dei token non
/// significativi ma non deve toccare la *posizione* assoluta `i`: il DTW deve
/// continuare a mappare la riga `i` corretta della matrice ColBERT.
///
/// **Terza via**: la topologia posizionale è conservata. Tutte le `N` posizioni
/// restano nel buffer (biiezione `N == N` col canale denso); i token non
/// significativi hanno peso `0.0`. Il DTW li esclude dalla *presenza* ma li
/// conta per l'*allineamento*.
#[test]
fn level2_filtro_igiene_preserva_posizione_assoluta() {
    // Walk con 7 passi, di cui due speciali (id 0 e id 2). Il filtro d'igiene
    // azzera il peso degli speciali ma conserva la posizione assoluta.
    let raw = RawWalk {
        sequence_id: "fixture_l2".into(),
        ids: vec![0, 211, 27294, 188, 54, 24022, 2], // 0 e 2 speciali
        w: vec![0.2, 0.15, 0.26, 0.19, 0.23, 0.18, 0.20],
        pos: vec![0, 1, 2, 3, 4, 5, 6],
        st: vec![0, 0, 0, 0, 0, 0, 0],
    };

    let seq = semantic_walk::parse::walk_to_sequence(&raw)
        .expect("walk ben formato produce una sequenza");

    // Terza via: la topologia è conservata — 7 posizioni totali (biiezione
    // col canale denso), non 5. Gli speciali (0 e 2) restano a peso 0.0.
    assert_eq!(seq.num_positions(), 7, "tutte le posizioni conservate (biiezione 7 == 7)");

    // I 5 token significativi mantengono il loro peso e la loro posizione
    // assoluta. Gli speciali hanno peso 0.0 ma la posizione resta.
    let attesi: Vec<u32> = vec![211, 27294, 188, 54, 24022];
    let pesi: Vec<f32> = vec![0.15, 0.26, 0.19, 0.23, 0.18];
    for (i, (&id, &peso)) in attesi.iter().zip(pesi.iter()).enumerate() {
        assert_eq!(seq.tokens_at(i + 1), &[id], "token alla posizione {}", i + 1);
        assert_eq!(seq.weights_at(i + 1), &[peso], "peso alla posizione {}", i + 1);
    }

    // Gli speciali sono presenti come posizioni a peso zero (verdetto, non
    // assenza): <s> in posizione 0, </s> in posizione 6.
    assert_eq!(seq.tokens_at(0), &[0], "token speciale <s> in posizione 0");
    assert_eq!(seq.weights_at(0), &[0.0], "<s> a peso zero");
    assert_eq!(seq.tokens_at(6), &[2], "token speciale </s> in posizione 6");
    assert_eq!(seq.weights_at(6), &[0.0], "</s> a peso zero");

    // Biiezione posizionale col canale denso: la posizione `i` del walk
    // corrisponde alla riga `i` della matrice ColBERT. Con la terza via la
    // topologia è conservata su entrambi i lati (7 == 7), quindi il DTW mappa
    // ogni posizione del walk alla riga ColBERT di pari indice — nessun
    // slittamento indotto dal filtro. La corrispondenza è verificabile ORA per
    // costruzione: `walk_to_sequence` non comprime, `colbert_to_trajectory`
    // non riordina, e `verifica_coerenza_posizionale` ha già asserito 7 == 7.
    // La biiezione si esprime nel fatto che il numero di posizioni della
    // sequenza coincida con le righe della traiettoria a cui il DTW si aggancia.
    let colbert = RawColbertTrajectory {
        sequence_id: "fixture_l2".into(),
        tokens: vec![
            "<s>".into(), "gatto".into(), "dorme".into(), "sul".into(),
            "divano".into(), "e".into(), "</s>".into(),
        ],
        embeddings: (0..7)
            .map(|i| vec![(i * 10 + 0) as f64, (i * 10 + 1) as f64, (i * 10 + 2) as f64, (i * 10 + 3) as f64])
            .collect(),
    };
    let traj = colbert_to_trajectory(&colbert).expect("traiettoria ben formata");

    // La posizione `i` del walk (7 posizioni, speciali inclusi) corrisponde
    // alla riga `i` della traiettoria (7 righe): la biiezione 7 == 7 regge
    // fino al DTW, che aggancia la posizione i-esima alla riga i-esima.
    assert_eq!(seq.num_positions(), traj.len(), "posizioni del walk == righe ColBERT (7 == 7)");
}

// ---------------------------------------------------------------------------
// Level 3 — Fixture reale: solo il padding finale è eliminato alla fonte
// ---------------------------------------------------------------------------

/// La fixture reale catturata dal server ("gatto dorme" con `<s>` e `</s>`)
/// deve dimostrare che la biiezione `7 == 7` regge: ogni riga ColBERT
/// corrisponde al passo `i` del walk, e gli speciali restano come posizioni a
/// peso zero (verdetto, non assenza).
#[test]
fn level3_fixture_reale_solo_padding_finale_eliminato() {
    // Dato reale catturato: 7 token con <s> (0) e </s> (2) ai bordi.
    // La matrice ColBERT ha una riga per ogni token (speciali compresi).
    let colbert_righe = 7usize;

    let raw = RawWalk {
        sequence_id: "il_gatto_dorme".into(),
        ids: vec![0, 211, 27294, 188, 54, 24022, 2],
        w: vec![0.216133, 0.145917, 0.261761, 0.192312, 0.228725, 0.182965, 0.203927],
        pos: vec![0, 1, 2, 3, 4, 5, 6],
        st: vec![0, 0, 0, 0, 0, 0, 0],
    };

    // Coerenza posizionale: 7 passi totali == 7 righe ColBERT.
    verifica_coerenza_posizionale(&raw, colbert_righe)
        .expect("7 passi totali == 7 righe ColBERT");

    let seq = semantic_walk::parse::walk_to_sequence(&raw)
        .expect("walk reale produce una sequenza");

    // Terza via: la biiezione 7 == 7 regge. Tutte le 7 posizioni restano;
    // gli speciali <s> (0) e </s> (2) hanno peso 0.0, i 5 significativi
    // (211, 27294, 188, 54, 24022) mantengono il loro peso e l'ordine esatto.
    assert_eq!(seq.num_positions(), 7, "biiezione 7 == 7 col canale denso");

    // Speciali a peso zero (verdetto, non assenza).
    assert_eq!(seq.tokens_at(0), &[0], "<s> in posizione 0");
    assert_eq!(seq.weights_at(0), &[0.0], "<s> a peso zero");
    assert_eq!(seq.tokens_at(6), &[2], "</s> in posizione 6");
    assert_eq!(seq.weights_at(6), &[0.0], "</s> a peso zero");

    // I 5 token significativi nell'ordine esatto del testo, con i loro pesi.
    let attesi: Vec<u32> = vec![211, 27294, 188, 54, 24022];
    let pesi: Vec<f32> = vec![0.145917, 0.261761, 0.192312, 0.228725, 0.182965];
    for (i, (&id, &peso)) in attesi.iter().zip(pesi.iter()).enumerate() {
        assert_eq!(seq.tokens_at(i + 1), &[id], "token significativo in posizione {}", i + 1);
        assert_eq!(seq.weights_at(i + 1), &[peso], "peso in posizione {}", i + 1);
    }
}