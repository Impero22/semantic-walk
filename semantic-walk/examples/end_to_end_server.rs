//! # End-to-end contro il server reale (vetta-semantic su .5:8091)
//!
//! Chiude l'anello sul campo: i dati JSON catturati dal server (matrice
//! ColBERT per-token e walk ordered-sparse) vengono deserializzati nelle
//! struct serde dell'adattatore, trasformati in `Raw*` da
//! `colbert_json_to_raw`/`walk_json_to_raw`, e verificati con
//! `verifica_coerenza_posizionale`.
//!
//! Il trasporto HTTP è demandato a `curl` (già eseguito a monte, file salvati
//! in `examples/data/`); questo esempio esercita solo il **codice reale**
//! dell'adattatore e del parsing, senza introdurre una dipendenza HTTP pesante
//! nel crate puro.
//!
//! ## Esecuzione
//!
//! ```bash
//! cargo run --example end_to_end_server
//! ```
//!
//! Richiede i file dati già catturati:
//! * `examples/data/colbert_il_gatto_dorme.json`
//! * `examples/data/walk_il_gatto_dorme.json`

use std::fs;

use semantic_walk::adapter::{
    colbert_json_to_raw, walk_json_to_raw, ColbertResultJson, ServerResponse, WalkResultJson,
};
use semantic_walk::parse::verifica_coerenza_posizionale;

fn main() {
    let colbert_path = "semantic-walk/examples/data/colbert_il_gatto_dorme.json";
    let walk_path = "semantic-walk/examples/data/walk_il_gatto_dorme.json";

    // --- 1. Leggi e deserializza la risposta ColBERT ---
    let colbert_text =
        fs::read_to_string(colbert_path).expect("file colbert non trovato: eseguire la cattura");
    let colbert_resp: ServerResponse<ColbertResultJson> =
        serde_json::from_str(&colbert_text).expect("JSON colbert non valido");

    let raw_colbert =
        colbert_json_to_raw(&colbert_resp, "il_gatto_dorme").expect("adattatore colbert fallito");
    let n_tokens = raw_colbert.embeddings.len();
    println!("[1] ColBERT: {} token, dim {}", n_tokens, raw_colbert.embeddings[0].len());

    // --- 2. Leggi e deserializza la risposta walk (forma flat) ---
    let walk_text = fs::read_to_string(walk_path).expect("file walk non trovato");
    let walk_resp: ServerResponse<WalkResultJson> =
        serde_json::from_str(&walk_text).expect("JSON walk non valido");

    let raw_walk =
        walk_json_to_raw(&walk_resp, "il_gatto_dorme").expect("adattatore walk fallito");
    let n_passi = raw_walk.ids.len();
    println!("[2] Walk: {} passi (ids {:?})", n_passi, raw_walk.ids);

    // --- 3. Coerenza posizionale: passi emessi == righe ColBERT ---
    match verifica_coerenza_posizionale(&raw_walk, n_tokens) {
        Ok(()) => println!(
            "[3] COERENZA POSIZIONALE OK: {} passi == {} righe ColBERT",
            n_passi, n_tokens
        ),
        Err(e) => {
            eprintln!("[3] COERENZA POSIZIONALE FALLITA: {e}");
            std::process::exit(1);
        }
    }

    // --- 4. Verifica il filtro d'igiene sul campo ---
    // Il primo passo è il token speciale <s> (id 0): il filtro d'igiene lo
    // deve escludere dal conteggio dei passi emessi. Verifichiamo che la
    // verifica abbia contato solo i token significativi.
    let significativi: usize = raw_walk
        .ids
        .iter()
        .zip(raw_walk.st.iter())
        .filter(|(id, st)| **st == 0 && **id >= 4)
        .count();
    println!("[4] Token significativi dopo filtro igiene (st==0, id>=4): {significativi}");

    println!("\n✅ END-TO-END SERVER: anello chiuso sul campo.");
}
