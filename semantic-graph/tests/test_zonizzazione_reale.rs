use semantic_graph::zonizzazione::{carica_dizionario, carica_traiettorie, similarita_traiettorie};
use std::path::Path;

#[test]
fn test_caricamento_e_confronto_dati_reali() {
    let path_diz = Path::new("../zonizzazione/dizionario_celle_v1.0.json");
    let path_traj = Path::new("../zonizzazione/traiettorie_v1.0.json");

    if !path_diz.exists() || !path_traj.exists() {
        return;
    }

    let diz = carica_dizionario(path_diz).expect("Errore caricamento dizionario");
    assert_eq!(diz.k, 256);
    assert_eq!(diz.centroidi.len(), 256);

    let traj = carica_traiettorie(path_traj).expect("Errore caricamento traiettorie");
    assert_eq!(traj.len(), 153);

    if let Some(t1) = traj.get("1") {
        let sim_self = similarita_traiettorie(t1, t1);
        assert!((sim_self - 1.0).abs() < 1e-5);

        if let Some(t2) = traj.get("2") {
            let sim_cross = similarita_traiettorie(t1, t2);
            assert!(sim_cross >= 0.0 && sim_cross <= 1.0);
        }
    }
}
