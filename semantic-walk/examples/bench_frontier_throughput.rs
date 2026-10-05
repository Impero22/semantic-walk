#![allow(dead_code)]

//! # Benchmark Throughput — potatura F6 a orizzonte crescente
//!
//! Integra la frontiera (`semantic-walk::frontier`) con i dati reali del
//! Dataset B, costruendo un **grafo di prossimità** dalle traiettorie dense.
//!
//! ## Il grafo
//!
//! Ogni frame di una traiettoria diventa un nodo con `KinematicState`
//! derivato dalla geometria locale:
//!
//! * `velocity` — norma del vettore del frame (la "velocità" del token nello
//!   spazio semantico).
//! * `acceleration` — delta di norma tra frame consecutivi (il jerk della
//!   camminata).
//! * `curvature` — angolo di deviazione tra vettori consecutivi (la curva).
//!
//! Le adiacenze sono la **catena temporale** (ogni frame → i successivi nella
//! stessa traiettoria) più la **prossimità cosine** (frame vicini tra
//! traiettorie diverse). La frontiera esplora questo grafo a orizzonte
//! crescente.
//!
//! ## La misura
//!
//! Per ogni orizzonte `H`:
//!
//! * **nodi totali** — i nodi che l'espansione genererebbe senza potatura.
//! * **nodi dopo prune** — i nodi che sopravvivono alla potatura F6.
//! * **risparmio %** — la frazione di espansioni risparmiate.
//!
//! L'interpretazione è quella del *riflesso permissivo*: la potatura elimina
//! solo i cammini che non possono vincere (upper bound di ampiezza sotto
//! l'incumbent oltre `PRUNE_EPSILON`), mai quelli nella fascia d'incertezza.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use npyz::NpyFile;
use serde::Deserialize;
use zip::ZipArchive;

use semantic_walk::frontier::{Frontier, FrontierNode, LevelDiagnostics, PRUNE_EPSILON};
use semantic_walk::KinematicState;

// ---------------------------------------------------------------------------
// Configurazione
// ---------------------------------------------------------------------------

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/dataset_b")
}

fn cache_path() -> PathBuf {
    data_dir().join("dataset_b_cache_fixed.npz")
}

/// Coefficienti dell'azione inerziale (rigidità della camminata).
const BETA: f32 = 1.0;
const GAMMA: f32 = 1.0;

/// Damping dell'ampiezza (kappa_break).
const KAPPA: f32 = 2.0;

/// Soglia di prossimità cosine per le adiacenze extra-temporali.
const PROX_THRESHOLD: f32 = 0.7;

/// Il minimo incremento inerziale per livello (bound per-livello).
/// Non è un lower bound teorico garantito da `u ≠ v` (le componenti
/// cinematiche sono f32 reali e possono avvicinarsi a zero arbitrariamente):
/// è il minimo **osservato** sugli archi reali del grafo, misurato a runtime
/// da `measure_min_step`. Il bound deve stare dove il fenomeno è, non sotto.
const MIN_STEP: f32 = 0.01; // ricalibrato a runtime dal minimo reale osservato

// ---------------------------------------------------------------------------
// Strutture dati
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
struct IndexEntry {
    category: String,
    text_a: String,
    text_b: String,
}

type DatasetIndex = HashMap<String, IndexEntry>;

/// Un nodo del grafo di prossimità.
struct GraphNode {
    /// Lo stato cinematico derivato dalla geometria locale del frame.
    state: KinematicState,
    /// I vicini (catena temporale + prossimità cosine).
    neighbors: Vec<usize>,
}

/// Il grafo di prossimità costruito dalle traiettorie.
struct ProximityGraph {
    nodes: Vec<GraphNode>,
}

impl ProximityGraph {
    /// Misura il minimo incremento inerziale osservato su tutti gli archi
    /// reali del grafo. Serve a calibrare `MIN_STEP` come lower bound
    /// **empirico effettivo** sui dati, non come costante arbitraria.
    fn measure_min_step(&self, beta: f32, gamma: f32) -> f32 {
        let mut min = f32::INFINITY;
        for node in &self.nodes {
            for &nb in &node.neighbors {
                let a = node
                    .state
                    .inertial_action(&self.nodes[nb].state, beta, gamma, 100.0, 1.0);
                if a > 0.0 && a < min {
                    min = a;
                }
            }
        }
        if min.is_finite() {
            min
        } else {
            1e-6
        }
    }

    /// Costruisce il grafo dai frame densi di tutte le coppie.
    fn from_trajectories(trajectories: &[Vec<Vec<f64>>]) -> Self {
        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut index_of = HashMap::new();

        // Passo 1: crea i nodi con lo stato cinematico derivato.
        for traj in trajectories {
            for (i, frame) in traj.iter().enumerate() {
                let state = derive_state(traj, i, frame);
                let idx = nodes.len();
                nodes.push(GraphNode { state, neighbors: Vec::new() });
                index_of.insert((traj as *const Vec<Vec<f64>> as usize, i), idx);
            }
        }

        // Passo 2: catena temporale — ogni frame → i successivi nella stessa
        // traiettoria (fino a +2).
        let mut base = 0usize;
        for traj in trajectories {
            let n = traj.len();
            for i in 0..n {
                let idx = base + i;
                for j in (i + 1)..(i + 3).min(n) {
                    nodes[idx].neighbors.push(base + j);
                }
            }
            base += n;
        }

        // Passo 3: prossimità cosine tra frame di traiettorie diverse.
        // Campiona per non rendere il grafo completo (O(n²) proibitivo).
        let total = nodes.len();
        for i in 0..total {
            let vi = &trajectories[traj_of(i, trajectories)];
            // limitiamo ai frame entro lo stesso documento per prossimità reale
            let _ = vi;
        }

        // Prossimità intra-traiettoria: connetti frame lontani ma simili.
        let mut base2 = 0usize;
        for traj in trajectories {
            let n = traj.len();
            for i in 0..n {
                let idx = base2 + i;
                let vi = &traj[i];
                for j in (i + 1)..n {
                    let sim = cosine(vi, &traj[j]);
                    if sim >= PROX_THRESHOLD {
                        nodes[idx].neighbors.push(base2 + j);
                    }
                }
            }
            base2 += n;
        }

        Self { nodes }
    }
}

fn traj_of(idx: usize, trajectories: &[Vec<Vec<f64>>]) -> usize {
    let mut acc = 0usize;
    for (t, traj) in trajectories.iter().enumerate() {
        acc += traj.len();
        if idx < acc {
            return t;
        }
    }
    trajectories.len().saturating_sub(1)
}

/// Deriva lo stato cinematico di un frame dalla geometria locale.
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

// ---------------------------------------------------------------------------
// Caricamento NPZ (stesso schema del benchmark B)
// ---------------------------------------------------------------------------

fn read_npz_array<T: npyz::Deserialize>(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let fname = npyz::npz::file_name_from_array_name(name);
    let file = zip.by_name(&fname)?;
    let reader = NpyFile::new(file)?;
    let data = reader.into_vec::<T>()?;
    Ok(data)
}

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

// ---------------------------------------------------------------------------
// Ricerca a frontiera con potatura
// ---------------------------------------------------------------------------

/// Esegue la ricerca a frontiera su un grafo a partire da una radice.
///
/// Ritorna `(nodi_totali_generati, nodi_dopo_prune, risparmio)` per l'orizzonte,
/// più la diagnostica per-livello dell'azione inerziale (profilazione).
fn frontier_search(
    graph: &ProximityGraph,
    root_idx: usize,
    horizon: usize,
    min_step: f32,
) -> (usize, usize, f64, Vec<LevelDiagnostics>) {
    let mut frontier = Frontier::new(KAPPA, horizon, min_step);

    // Radice: stato del nodo radice, costo 0, profondità 0.
    let root = FrontierNode::root(
        root_idx as u64,
        graph.nodes[root_idx].state,
        min_step,
    );
    frontier.push(root);

    let mut total_generated = 0usize;
    let mut depth = 0usize;
    // Diagnostica per-livello: azioni inerziali degli archi attraversati.
    let mut level_diag: Vec<LevelDiagnostics> = Vec::new();

    while depth < horizon {
        let level_size = frontier.len();
        if level_size == 0 {
            break;
        }
        total_generated += level_size;

        // Espandi il livello corrente, raccogliendo le azioni inerziali.
        let mut next_level = Frontier::new(KAPPA, horizon, min_step);
        let mut actions: Vec<f32> = Vec::new();
        while let Some(node) = frontier.pop() {
            let nid = node.node_id as usize;
            for &nb in &graph.nodes[nid].neighbors {
                let child = node.extend(
                    nb as u64,
                    graph.nodes[nb].state,
                    BETA,
                    GAMMA,
                    100.0,
                    1.0,
                    min_step,
                    KAPPA,
                );
                if let Some(child) = child {
                    // L'azione inerziale di questo passo è la differenza di costo.
                    actions.push(child.cum_cost - node.cum_cost);
                    next_level.push(child);
                }
            }
        }
        // Registra la distribuzione dell'azione inerziale di questo livello.
        if let Some(d) = LevelDiagnostics::from_actions(actions) {
            level_diag.push(d);
        }

        // Potatura F6 esatta sul livello successivo.
        next_level.prune();
        let after_prune = next_level.len();

        // Sostituisci la frontiera col livello potato.
        frontier = next_level;
        depth += 1;

        if depth == horizon {
            total_generated += after_prune;
        }
    }

    let after_prune_final = frontier.len();
    let saved = total_generated.saturating_sub(after_prune_final);
    let ratio = if total_generated > 0 {
        saved as f64 / total_generated as f64
    } else {
        0.0
    };
    (total_generated, after_prune_final, ratio, level_diag)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[FRONTIER] Cache: {}", cache_path().display());

    let index_file = File::open(data_dir().join("dataset_b_index.json"))?;
    let reader = BufReader::new(index_file);
    let index: DatasetIndex = serde_json::from_reader(reader)?;
    println!("[FRONTIER] Coppie totali: {}", index.len());

    let npz_file = File::open(&cache_path())?;
    let mut zip = ZipArchive::new(BufReader::new(npz_file))?;

    // Carica le traiettorie dense di tutte le coppie.
    let mut trajectories: Vec<Vec<Vec<f64>>> = Vec::new();
    for (pair_id, _entry) in &index {
        let seq_a = read_dense(&mut zip, &format!("{}_a_dense", pair_id))?;
        let seq_b = read_dense(&mut zip, &format!("{}_b_dense", pair_id))?;
        if !seq_a.is_empty() {
            trajectories.push(seq_a);
        }
        if !seq_b.is_empty() {
            trajectories.push(seq_b);
        }
    }
    println!(
        "[FRONTIER] Traiettorie caricate: {}, frame totali: {}",
        trajectories.len(),
        trajectories.iter().map(|t| t.len()).sum::<usize>()
    );

    // Costruisce il grafo di prossimità.
    let graph = ProximityGraph::from_trajectories(&trajectories);
    let n_nodes = graph.nodes.len();
    let n_edges: usize = graph.nodes.iter().map(|n| n.neighbors.len()).sum();
    println!("[FRONTIER] Grafo: {} nodi, {} archi", n_nodes, n_edges);

    // Calibra MIN_STEP come lower bound empirico: il minimo incremento
    // inerziale osservato sugli archi reali. Il bound deve stare dove il
    // fenomeno è, non sotto (lezione SPARSE_EPSILON).
    let min_step = graph.measure_min_step(BETA, GAMMA);
    println!(
        "[FRONTIER] MIN_STEP ricalibrato: minimo ΔS_inerziale osservato = {:.6}",
        min_step
    );

    // Soglie di orizzonte da testare.
    let horizons = [1usize, 2, 3, 4, 5, 8];

    println!("\n=== Throughput della potatura F6 a orizzonte crescente ===");
    println!(
        "{:>4} | {:>12} {:>12} | {:>12} {:>10}",
        "H", "nodi_totali", "nodi_dopo", "risparmio_abs", "risparmio_%"
    );

    // Campiona un sottoinsieme di radici per stabilità.
    let n_roots = n_nodes.min(200);
    let roots: Vec<usize> = (0..n_roots).collect();

    for &h in &horizons {
        let mut total_gen = 0usize;
        let mut total_after = 0usize;
        // Aggrega la diagnostica per-livello su tutte le radici campionate.
        let mut agg_min = f32::INFINITY;
        let mut agg_max = f32::NEG_INFINITY;
        let mut agg_sum = 0.0f32;
        let mut agg_count = 0usize;
        for &r in &roots {
            let (gen, after, _ratio, level_diag) = frontier_search(&graph, r, h, min_step);
            total_gen += gen;
            total_after += after;
            for d in &level_diag {
                agg_count += d.count;
                agg_sum += d.mean * d.count as f32;
                if d.min < agg_min {
                    agg_min = d.min;
                }
                if d.max > agg_max {
                    agg_max = d.max;
                }
            }
        }
        let saved = total_gen.saturating_sub(total_after);
        let ratio = if total_gen > 0 {
            saved as f64 / total_gen as f64
        } else {
            0.0
        };
        // Stampa la diagnostica aggregata (profilazione, non potatura).
        let agg_mean = if agg_count > 0 {
            agg_sum / agg_count as f32
        } else {
            0.0
        };
        println!(
            "{:>4} | {:>12} {:>12} | {:>12} {:>9.2}%",
            h, total_gen, total_after, saved, ratio * 100.0
        );
        // Profilazione: distribuzione dell'azione inerziale sugli archi reali.
        if agg_count > 0 {
            println!(
                "     |  ΔS_inerziale: min={:.4} max={:.4} media={:.4} ({} archi)",
                agg_min, agg_max, agg_mean, agg_count
            );
        }
    }

    println!("\n[FRONTIER] PRUNE_EPSILON = {}", PRUNE_EPSILON);
    println!("[FRONTIER] Prossimità cosine per adiacenze: ≥ {}", PROX_THRESHOLD);

    Ok(())
}