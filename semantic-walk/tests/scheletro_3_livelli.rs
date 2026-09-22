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
//!   che il DTW mappi la riga `i` corretta.
//! * **Level 3 — Fixture reale**: la sequenza di 7 token ("gatto dorme" con
//!   `<s>` e `</s>`) deve dimostrare che l'unica eliminazione ammessa alla
//!   fonte è il padding finale `<pad>`.
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

    // Asserzione critica da completare quando il server espone il frames mode:
    // la riga `i` deserializzata deve corrispondere al token `i` dichiarato
    // dal payload, senza che il parsing ne alteri l'ordine né la lunghezza.
    todo!("con la fixture reale: asserire che la riga i della matrice corrisponda al token i dichiarato dal payload");
}

// ---------------------------------------------------------------------------
// Level 2 — Filtro d'igiene: la posizione assoluta è preservata
// ---------------------------------------------------------------------------

/// Il filtro d'igiene (`st == 0 && id >= 4`) azzera il *peso* dei token non
/// significativi ma non deve toccare la *posizione* assoluta `i`: il DTW deve
/// continuare a mappare la riga `i` corretta della matrice ColBERT.
#[test]
fn level2_filtro_igiene_preserva_posizione_assoluta() {
    // Walk con 7 passi, di cui alcuni da gittare (soppressi st==1, speciali
    // id<4). Il filtro deve scartarli dal buffer ma la posizione dei
    // sopravvissuti nel cammino deve restare ancorata alla loro posizione
    // assoluta nel testo.
    let raw = RawWalk {
        sequence_id: "fixture_l2".into(),
        ids: vec![0, 211, 27294, 188, 54, 24022, 2], // 0 e 2 speciali
        w: vec![0.2, 0.15, 0.26, 0.19, 0.23, 0.18, 0.20],
        pos: vec![0, 1, 2, 3, 4, 5, 6],
        st: vec![0, 0, 0, 0, 0, 0, 0],
    };

    let seq = semantic_walk::parse::walk_to_sequence(&raw)
        .expect("walk ben formato produce una sequenza");

    // I token speciali (0 e 2) devono essere esclusi dal buffer: il cammino
    // deve avere 5 passi significativi (211, 27294, 188, 54, 24022).
    // Questa parte è verificabile ORA, per costruzione del filtro d'igiene,
    // senza attendere il frames mode del server.
    assert_eq!(seq.num_positions(), 5, "i 5 token significativi sopravvivono al filtro");

    // L'ordine posizionale dei sopravvissuti è quello del testo: il filtro
    // non riordina, scarta soltanto. Ogni posizione ha un solo token (il walk
    // reale ha una voce per occorrenza), quindi tokens_at(i) è un singoletto.
    let attesi: Vec<u32> = vec![211, 27294, 188, 54, 24022];
    for (i, &id) in attesi.iter().enumerate() {
        assert_eq!(seq.tokens_at(i), &[id], "token alla posizione {i}");
    }

    // Asserzione critica rimandata alla fixture reale: il frames mode del
    // server deve garantire che la posizione assoluta `i` nel testo sia
    // preservata anche dopo il filtro, così che il DTW mappi la riga ColBERT
    // corretta. La biiezione è già coperta da `verifica_coerenza_posizionale`
    // a monte; qui resta l'ancora finché non catturiamo il payload reale.
    todo!("con la fixture reale: asserire che la riga i della matrice ColBERT corrisponda al token i del cammino filtrato");
}

// ---------------------------------------------------------------------------
// Level 3 — Fixture reale: solo il padding finale è eliminato alla fonte
// ---------------------------------------------------------------------------

/// La fixture reale catturata dal server ("gatto dorme" con `<s>` e `</s>`)
/// deve dimostrare che l'unica eliminazione ammessa alla fonte è il padding
/// finale `<pad>`: tutti i token significativi restano, e la sequenza
/// ordinata li conserva nell'ordine esatto.
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

    // Dopo il filtro d'igiene restano i 5 token significativi, nell'ordine
    // esatto del testo: gatto(211) dorme(27294) sul(188) divano(54) e(24022).
    // TODO: asserire che la sequenza conserva esattamente questi 5 token
    // nell'ordine posizionale, e che l'unica eliminazione ammessa alla fonte
    // è il padding finale <pad> (non presente in questo frame).
    let _ = &seq;
    todo!("asserire che i 5 token significativi sono 211,27294,188,54,24022 nell'ordine esatto");
}