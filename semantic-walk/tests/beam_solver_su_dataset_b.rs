//! # beam_solver_su_dataset_b — il solver sul grafo reale del Dataset B
//!
//! Test di integrazione che lancia il `BeamSolver` sul grafo di prossimità
//! costruito dai frame densi reali del Dataset B, secondo l'**Opzione A**
//! concordata con Camillo (03/10):
//!
//! * `H` è un **massimo target**, non una lunghezza imposta — i cammini
//!   validi hanno lunghezza variabile `1 ≤ h ≤ H`.
//! * La verifica dell'esplorazione completa: da un nodo della componente
//!   principale, a profondità `H=10` il solver deve produrre almeno un
//!   cammino di profondità 10 (che raggiunge l'orizzonte).
//!
//! ## La convenzione dello stato cinematico
//!
//! Per essere coerente con la verifica di throughput già fatta nel benchmark
//! `bench_frontier_throughput`, lo stato del nodo è derivato dalla
//! **geometria locale del frame** (non dalla media delle sonde degli archi
//! di `ProximityAdapter`):
//!
//! * `velocity` — norma del vettore del frame.
//! * `acceleration` — delta di norma tra frame consecutivi.
//! * `curvature` — angolo di deviazione tra frame consecutivi.
//!
//! L'adapter dedicato (`FrameAdapter`) implementa `GraphAdapter` su un
//! `semantic_graph::Graph` costruito con le stesse adiacenze del benchmark
//! (catena temporale + prossimità cosine).

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_combiner::FactId;
use semantic_graph::{Edge, Graph, GraphConfig, NodeId};
use semantic_walk::solver::{BeamSolver, GraphAdapter, SolverConfig};
use semantic_walk::KinematicState;

// ---------------------------------------------------------------------------
// Configurazione (stessa del benchmark di throughput)
// ---------------------------------------------------------------------------

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_b")
}

fn cache_path() -> PathBuf {
    data_dir().join("dataset_b_cache_fixed.npz")
}

const BETA: f32 = 1.0;
const GAMMA: f32 = 1.0;
const KAPPA: f32 = 2.0;
const PROX_THRESHOLD: f32 = 0.7;

/// Orizzonte per la verifica dell'esplorazione completa (Opzione A).
const H_TARGET: usize = 10;

// ---------------------------------------------------------------------------
// Strutture dati (caricamento NPZ, stesso schema del benchmark B)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct IndexEntry {
    category: String,
    text_a: String,
    text_b: String,
}

type DatasetIndex = HashMap<String, IndexEntry>;

fn read_dense(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
) -> Result<Vec<Vec<f64>>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    let shape = reader.shape().to_vec();
    let flat = reader.into_vec::<f32>()?;

    let dim = if shape.len() >= 2 { shape[1] as usize } else { 1 };
    let n = if flat.is_empty() { 0 } else { flat.len() / dim };

    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        let row = flat[i * dim..(i + 1) * dim]
            .iter()
            .map(|&x| x as f64)
            .collect::<Vec<f64>>();
        rows.push(row);
    }
    Ok(rows)
}

fn frame_norm(v: &[f64]) -> f32 {
    let s: f64 = v.iter().map(|x| x * x).sum();
    s.sqrt() as f32
}

fn cosine(a: &[f64], b: &[f64]) -> f32 {
    let na = frame_norm(a) as f64;
    let nb = frame_norm(b) as f64;
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    (dot / (na * nb)) as f32
}

fn angle_between(a: &[f64], b: &[f64]) -> f32 {
    let c = cosine(a, b).clamp(-1.0, 1.0);
    c.acos()
}

/// Deriva lo stato cinematico di un frame dalla geometria locale
/// (convenzione del benchmark di throughput).
fn derive_state(traj: &[Vec<f64>], i: usize, frame: &[f64]) -> KinematicState {
    let norm = frame_norm(frame);
    let prev_norm = if i > 0 { frame_norm(&traj[i - 1]) } else { norm };
    let acc = norm - prev_norm;
    let curvature = if i > 0 {
        angle_between(&traj[i - 1], frame)
    } else {
        0.0
    };
    KinematicState { velocity: norm, acceleration: acc, curvature }
}

// ---------------------------------------------------------------------------
// Costruzione del grafo di prossimità dai frame densi
// ---------------------------------------------------------------------------

/// Costruisce un `semantic_graph::Graph` dai frame densi delle traiettorie.
///
/// Le adiacenze sono le stesse del benchmark: catena temporale (ogni frame →
/// i successivi nella stessa traiettoria, fino a +2) più prossimità cosine
/// intra-traiettoria (frame lontani ma simili). Ogni frame diventa un nodo
/// con `NodeId` = indice globale del frame.
fn costruisci_grafo(trajectories: &[Vec<Vec<f64>>]) -> Graph {
    let mut nodi: Vec<NodeId> = Vec::new();
    let mut archi: Vec<Edge> = Vec::new();

    // Passo 1: crea i nodi (un nodo per frame).
    let mut base = 0usize;
    for traj in trajectories {
        for _ in traj {
            nodi.push(NodeId(base as u64));
            base += 1;
        }
    }

    // Deduplica le coppie (from, to): la stessa coppia può emergere sia dalla
    // catena temporale (passo 2) sia dalla prossimità cosine (passo 3).
    // Senza deduplicazione `neighbors` restituirebbe vicini duplicati,
    // raddoppiando artificialmente i rami espansi dal `BeamSolver`.
    let mut visti: HashSet<(u64, u64)> = HashSet::new();

    // Aggiunge un arco solo se la coppia (from, to) non è già stata inserita.
    let push_arco = |archi: &mut Vec<Edge>, visti: &mut HashSet<(u64, u64)>, from: usize, to: usize, sim: f32| {
        if visti.insert((from as u64, to as u64)) {
            archi.push(Edge {
                from: NodeId(from as u64),
                to: NodeId(to as u64),
                score: sim as f64,
                dense: sim as f64,
                sparse: 0.0,
                colbert: 0.0,
            });
        }
    };

    // Passo 2: catena temporale — ogni frame → i successivi (fino a +2).
    let mut base2 = 0usize;
    for traj in trajectories {
        let n = traj.len();
        for i in 0..n {
            let from = base2 + i;
            for j in (i + 1)..(i + 3).min(n) {
                let to = base2 + j;
                let sim = cosine(&traj[i], &traj[j]);
                push_arco(&mut archi, &mut visti, from, to, sim);
            }
        }
        base2 += n;
    }

    // Passo 3: prossimità cosine intra-traiettoria (frame lontani ma simili).
    let mut base3 = 0usize;
    for traj in trajectories {
        let n = traj.len();
        for i in 0..n {
            let from = base3 + i;
            for j in (i + 1)..n {
                let sim = cosine(&traj[i], &traj[j]);
                if sim >= PROX_THRESHOLD {
                    let to = base3 + j;
                    push_arco(&mut archi, &mut visti, from, to, sim);
                }
            }
        }
        base3 += n;
    }

    Graph { config: GraphConfig::default(), nodi, archi }
}

/// L'adapter dedicato: deriva lo stato dalla geometria locale dei frame
/// (convenzione del benchmark), non dalla media delle sonde degli archi.
struct FrameAdapter {
    grafo: Graph,
    /// Lo stato cinematico di ogni frame, indicizzato per `NodeId`.
    stati: HashMap<NodeId, KinematicState>,
    /// La mappa di adiacenza precalcolata: per ogni nodo, i suoi vicini.
    ///
    /// Evita la scansione O(|E|) dell'intero vettore `archi` a ogni chiamata
    /// di `neighbors`: con i ~54.000 archi del Dataset B e centinaia di nodi
    /// espansi su 10 livelli, la scansione lineare avrebbe prodotto decine di
    /// milioni di iterazioni. Popolata una sola volta in `new`.
    adiacenze: HashMap<u64, Vec<u64>>,
}

impl FrameAdapter {
    fn new(trajectories: &[Vec<Vec<f64>>]) -> Self {
        let grafo = costruisci_grafo(trajectories);

        // Deriva lo stato locale di ogni frame dalla geometria.
        let mut stati = HashMap::new();
        let mut base = 0usize;
        for traj in trajectories {
            for (i, frame) in traj.iter().enumerate() {
                stati.insert(NodeId((base + i) as u64), derive_state(traj, i, frame));
            }
            base += traj.len();
        }

        // Precalcola la mappa di adiacenza: per ogni nodo, i suoi vicini
        // (deduplicati già a monte da `costruisci_grafo`).
        let mut adiacenze: HashMap<u64, Vec<u64>> = HashMap::new();
        for e in &grafo.archi {
            adiacenze.entry(e.from.0).or_default().push(e.to.0);
            adiacenze.entry(e.to.0).or_default().push(e.from.0);
        }

        FrameAdapter { grafo, stati, adiacenze }
    }
}

impl GraphAdapter for FrameAdapter {
    fn neighbors(&self, node: FactId) -> Vec<(FactId, KinematicState)> {
        // Accesso O(1) alla lista di adiacenza del nodo: nessuna scansione
        // dell'intero vettore `archi`.
        let mut out = Vec::new();
        if let Some(vicini) = self.adiacenze.get(&node) {
            for &other in vicini {
                let state = self.stati.get(&NodeId(other)).copied().unwrap_or_default();
                out.push((other, state));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

/// Lancia il `BeamSolver` sul Dataset B con l'Opzione A: H come massimo
/// target, cammini validi `1 ≤ h ≤ H`, e verifica l'esplorazione completa
/// da un nodo della componente principale a profondità `H=10`.
#[test]
fn beam_solver_su_dataset_b() {
    let index_file = File::open(data_dir().join("dataset_b_index.json")).unwrap();
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader).unwrap();
    assert!(!index.is_empty(), "il Dataset B non deve essere vuoto");

    let npz_file = File::open(&cache_path()).unwrap();
    let mut zip = ZipArchive::new(BufReader::new(npz_file)).unwrap();

    // Carica le traiettorie dense di tutte le coppie.
    let mut trajectories: Vec<Vec<Vec<f64>>> = Vec::new();
    for (pair_id, _entry) in &index {
        let seq_a = read_dense(&mut zip, &format!("{}_a_dense", pair_id)).unwrap();
        let seq_b = read_dense(&mut zip, &format!("{}_b_dense", pair_id)).unwrap();
        if !seq_a.is_empty() {
            trajectories.push(seq_a);
        }
        if !seq_b.is_empty() {
            trajectories.push(seq_b);
        }
    }
    assert!(!trajectories.is_empty(), "nessuna traiettoria caricata");

    let adapter = FrameAdapter::new(&trajectories);
    let n_nodes = adapter.grafo.nodi.len();
    assert!(n_nodes > 0, "il grafo deve avere nodi");

    // Sceglie un nodo radice della componente principale: il primo nodo con
    // almeno un vicino (non isolato) è un candidato ragionevole.
    let root_id = {
        let mut root = None;
        for n in &adapter.grafo.nodi {
            let has_neighbor = adapter.grafo.archi.iter().any(|e| e.from == *n || e.to == *n);
            if has_neighbor {
                root = Some(n.0);
                break;
            }
        }
        root.expect("deve esistere almeno un nodo non isolato")
    };
    let root_state = adapter.stati.get(&NodeId(root_id)).copied().unwrap_or_default();

    // Costruisce il solver con l'Opzione A: H = massimo target.
    let config = SolverConfig {
        horizon: H_TARGET,
        kappa: KAPPA,
        beta: BETA,
        gamma: GAMMA,
        c_sem: 100.0,
        m_sem: 1.0,
        min_step: 0.002,
    };
    let solver = BeamSolver::new(adapter, config);

    // Esegue la ricerca (Opzione A: cammini di lunghezza 1..=H).
    let paths = solver.solve(root_id, root_state);

    // Verifica 1: deve esistere almeno un cammino superstite.
    assert!(
        !paths.is_empty(),
        "il solver deve produrre almeno un cammino superstite"
    );

    // Verifica 2: tutti i cammini hanno lunghezza 1 ≤ h ≤ H (Opzione A).
    for p in &paths {
        assert!(
            !p.nodes.is_empty(),
            "un cammino superstite non può essere vuoto (la radice è esclusa)"
        );
        assert!(
            p.nodes.len() <= H_TARGET,
            "cammino di lunghezza {} supera l'orizzonte massimo {}",
            p.nodes.len(),
            H_TARGET
        );
    }

    // Verifica 3 (esplorazione completa): almeno un cammino raggiunge la
    // profondità H=10 (l'orizzonte massimo) — la componente principale è
    // esplorata fino in fondo.
    let max_len = paths.iter().map(|p| p.nodes.len()).max().unwrap_or(0);
    assert!(
        max_len >= H_TARGET,
        "l'esplorazione deve raggiungere la profondità H={}: massimo osservato {}",
        H_TARGET,
        max_len
    );

    // Verifica 4 (integrità): ogni cammino ha costo non negativo e ampiezza
    // in (0, 1] — l'azione inerziale è un costo, mai un guadagno.
    for p in &paths {
        assert!(p.total_cost >= 0.0, "costo inerziale negativo: {}", p.total_cost);
        assert!(p.amplitude > 0.0 && p.amplitude <= 1.0, "ampiezza fuori range: {}", p.amplitude);
    }
}
