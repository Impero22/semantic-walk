//! # sweep_percentile — calibrazione del percentile operativo
//!
//! Benchmark di calibrazione: esegue uno sweep di `percentile_cutoff` sui
//! dati reali di zonizzazione e misura tre metriche chiave del grafo per
//! ciascun valore:
//!
//! 1. **grado medio e varianza del grado** — assenza di hub sovrasaturi;
//! 2. **numero di componenti connesse** — l'adattamento locale non frammenta;
//! 3. **coefficiente di clustering medio** — selettività della topologia.
//!
//! ## Stato attuale (sonda provvisoria)
//!
//! I dati reali di zonizzazione producono oggi una **sola sonda**: la
//! similarità tra traiettorie (Jaccard pesato sulle celle). La usiamo come
//! `dense` provvisoria. Le sonde `sparse` e `colbert` arriveranno con il
//! CrispEmbed dedicato; quando sarà pronto, lo stesso script gira identico
//! con la tripletta vera (basta riempire i tre campi di `Fatto`).
//!
//! ## Uso
//!
//! ```bash
//! cargo run -p semantic-graph --example sweep_percentile
//! ```
//!
//! Il percorso dei dati è relativo alla radice del workspace; se i file non
//! esistono, il benchmark esce con un messaggio chiaro.

use semantic_graph::metriche::clustering;
use semantic_graph::{Graph, GraphConfig, NodeId};
use semantic_graph::zonizzazione::{carica_dizionario, carica_traiettorie, similarita_traiettorie};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Un insieme disgiunto (union-find) per contare le componenti connesse.
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
    }
}

/// Numero di componenti connesse del grafo (nodi isolati contano come una
/// componente ciascuno).
fn componenti_connesse(g: &Graph) -> usize {
    if g.nodi.is_empty() {
        return 0;
    }
    let idx: HashMap<NodeId, usize> = g
        .nodi
        .iter()
        .enumerate()
        .map(|(i, &n)| (n, i))
        .collect();
    let mut uf = UnionFind::new(g.nodi.len());
    for e in &g.archi {
        if let (Some(&a), Some(&b)) = (idx.get(&e.from), idx.get(&e.to)) {
            uf.union(a, b);
        }
    }
    let radici: HashSet<usize> = (0..g.nodi.len()).map(|i| uf.find(i)).collect();
    radici.len()
}

/// Media e varianza del grado dei nodi.
fn grado_media_varianza(g: &Graph) -> (f64, f64) {
    if g.nodi.is_empty() {
        return (0.0, 0.0);
    }
    let gradi: Vec<usize> = g.nodi.iter().map(|&n| metriche_grado(g, n)).collect();
    let n = gradi.len() as f64;
    let media = gradi.iter().sum::<usize>() as f64 / n;
    let varianza = gradi
        .iter()
        .map(|&d| {
            let diff = d as f64 - media;
            diff * diff
        })
        .sum::<f64>()
        / n;
    (media, varianza)
}

/// Grado di un nodo (copia locale per evitare dipendenza dal modulo metriche).
fn metriche_grado(g: &Graph, id: NodeId) -> usize {
    g.archi
        .iter()
        .filter(|e| e.from == id || e.to == id)
        .count()
}

/// Clustering medio dei nodi (ignora i nodi con meno di 2 vicini).
fn clustering_medio(g: &Graph) -> f64 {
    let valori: Vec<f64> = g
        .nodi
        .iter()
        .filter_map(|&n| clustering(g, n))
        .collect();
    if valori.is_empty() {
        return 0.0;
    }
    valori.iter().sum::<f64>() / valori.len() as f64
}

fn main() {
    let path_diz = Path::new("zonizzazione/dizionario_celle_v1.0.json");
    let path_traj = Path::new("zonizzazione/traiettorie_v1.0.json");

    if !path_diz.exists() || !path_traj.exists() {
        eprintln!(
            "Dati reali non trovati. Atteso: {} e {}\n\
             Esegui dalla radice del workspace (semantic-geo).",
            path_diz.display(),
            path_traj.display()
        );
        std::process::exit(1);
    }

    let _diz = carica_dizionario(path_diz).expect("Errore caricamento dizionario");
    let traj = carica_traiettorie(path_traj).expect("Errore caricamento traiettorie");

    // Id delle traiettorie, ordinati.
    let mut ids: Vec<u64> = traj.keys().filter_map(|k| k.parse().ok()).collect();
    ids.sort_unstable();

    // Matrice di similarità tra le traiettorie (precalcolata).
    let mut sim: HashMap<(u64, u64), f64> = HashMap::new();
    for (i, &a) in ids.iter().enumerate() {
        for &b in ids.iter().skip(i + 1) {
            let t1 = traj.get(&a.to_string()).unwrap();
            let t2 = traj.get(&b.to_string()).unwrap();
            let s = similarita_traiettorie(t1, t2) as f64;
            sim.insert((a, b), s);
            sim.insert((b, a), s);
        }
    }

    // Costruzione diretta del grafo dalla matrice di similarità.
    // Poiché abbiamo una sola sonda (le celle), il punteggio di un arco è
    // proprio la similarità celle. Usiamo la matrice precalcolata per lo
    // sweep del percentile.

    let percentili: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 0.9];

    println!("=== SWEEP PERCENTILE_CUTOFF — dati reali zonizzazione ===");
    println!(
        "Nodi: {} | Sonde: dense=similarità celle (provvisoria), sparse/colbert=0",
        ids.len()
    );
    println!();

    for &p in &percentili {
        // Costruisce il grafo per questo percentile: per ogni nodo, i suoi
        // top-k vicini (per similarità celle) sopra il percentile locale.
        let k = 5usize;
        let mut archi: Vec<(u64, u64, f64)> = Vec::new();

        for &a in &ids {
            // Candidati: tutti gli altri nodi con la loro similarità.
            let mut candidati: Vec<(u64, f64)> = ids
                .iter()
                .filter(|&&b| b != a)
                .map(|&b| (b, sim[&(a, b)]))
                .collect();
            candidati.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal));
            candidati.truncate(k);

            // Soglia effettiva: percentile dei punteggi dei top-k.
            let scores: Vec<f64> = candidati.iter().map(|c| c.1).collect();
            let soglia = percentile(&scores, p);

            for (b, score) in candidati {
                if score >= soglia {
                    let (from, to) = if a <= b { (a, b) } else { (b, a) };
                    if from != to {
                        archi.push((from, to, score));
                    }
                }
            }
        }

        archi.sort_by(|x, y| x.0.cmp(&y.0).then(x.1.cmp(&y.1)));
        archi.dedup_by(|x, y| x.0 == y.0 && x.1 == y.1);

        let g = Graph {
            config: GraphConfig {
                k,
                soglia: 0.0,
                percentile_cutoff: p,
                pesi: [1.0, 0.0, 0.0],
            },
            nodi: ids.iter().map(|&i| NodeId(i)).collect(),
            archi: archi
                .iter()
                .map(|&(f, t, s)| semantic_graph::Edge {
                    from: NodeId(f),
                    to: NodeId(t),
                    score: s,
                    dense: s,
                    sparse: 0.0,
                    colbert: 0.0,
                })
                .collect(),
        };

        let (grado_medio, grado_var) = grado_media_varianza(&g);
        let comp = componenti_connesse(&g);
        let clust = clustering_medio(&g);

        println!(
            "percentile={:<4} | archi={:<4} | grado_medio={:.2} | varianza_grado={:.2} | componenti={:<3} | clustering_medio={:.3}",
            p,
            g.archi.len(),
            grado_medio,
            grado_var,
            comp,
            clust
        );
    }

    println!();
    println!("Nota: sonda densa provvisoria (similarità celle). Con le sonde complete");
    println!("(CrispEmbed) lo stesso script gira identico riempiendo dense/sparse/colbert.");
}

/// Percentile (interpolazione lineare) di una serie di punteggi.
fn percentile(scores: &[f64], p: f64) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    let p = p.clamp(0.0, 1.0);
    let mut ordinati = scores.to_vec();
    ordinati.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = p * (ordinati.len() - 1) as f64;
    let lower = idx.floor() as usize;
    let upper = idx.ceil() as usize;
    if lower == upper {
        ordinati[lower]
    } else {
        let weight = idx - lower as f64;
        ordinati[lower] * (1.0 - weight) + ordinati[upper] * weight
    }
}