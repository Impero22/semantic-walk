//! # metriche — la geografia del grafo
//!
//! Metriche **derivate**, non immagazzinate: il grafo le calcola al bisogno,
//! come il gate calcola il verdetto. Funzioni pure: operano su slice e
//! restituiscono valori o `Vec` di valori (il calcolo è O(E) per nodo).
//!
//! - **grado** — quanti vicini ha un fatto. Alto = hub, basso = isolato.
//! - **clustering** — quanto i vicini di un fatto sono vicini tra loro.
//!   Alto = cluster denso, basso = ponte.
//! - **orfano** — grado zero o quasi. Un fatto che non si collega a nessuno.

use crate::{Edge, Graph, NodeId};

/// Il grado di un nodo: il numero di archi incidenti.
///
/// Scansione lineare degli archi: conta quelli con `from == id` o `to == id`.
/// (Gli archi sono ordinati per coppia (from, to) con from < to, ma il grado
/// non sfrutta l'ordine: è una metrica O(E), calcolata al bisogno.)
pub fn grado(grafo: &Graph, id: NodeId) -> usize {
    grafo.archi
        .iter()
        .filter(|e| e.from == id || e.to == id)
        .count()
}

/// Il grado di ogni nodo, in un vettore parallelo a `grafo.nodi`.
///
/// Restituisce `(nodo, grado)` per ogni nodo del grafo.
pub fn gradi(grafo: &Graph) -> Vec<(NodeId, usize)> {
    grafo.nodi
        .iter()
        .map(|&n| (n, grado(grafo, n)))
        .collect()
}

/// `true` se il nodo è un orfano: grado zero (nessun legame).
pub fn e_orfano(grafo: &Graph, id: NodeId) -> bool {
    grado(grafo, id) == 0
}

/// Gli orfani del grafo: i nodi con grado zero.
pub fn orfani(grafo: &Graph) -> Vec<NodeId> {
    grafo.nodi
        .iter()
        .copied()
        .filter(|&n| e_orfano(grafo, n))
        .collect()
}

/// I vicini di un nodo: gli altri estremi dei suoi archi.
///
/// Restituisce una lista di `NodeId` (senza l'id stesso, senza duplicati).
fn vicini(grafo: &Graph, id: NodeId) -> Vec<NodeId> {
    let mut v: Vec<NodeId> = grafo
        .archi
        .iter()
        .filter(|e| e.from == id || e.to == id)
        .map(|e| if e.from == id { e.to } else { e.from })
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Il coefficiente di clustering locale di un nodo: la frazione di coppie
/// di vicini che sono anche collegati tra loro.
///
/// - 1.0 = i vicini formano un cluster completo (denso).
/// - 0.0 = nessun vicino è collegato a un altro (il nodo è un ponte).
/// - `None` se il nodo ha meno di due vicini (indefinito).
pub fn clustering(grafo: &Graph, id: NodeId) -> Option<f64> {
    let v = vicini(grafo, id);
    let n = v.len();
    if n < 2 {
        return None;
    }
    let possibili = n * (n - 1) / 2;
    let presenti = v
        .iter()
        .enumerate()
        .flat_map(|(i, &a)| v.iter().skip(i + 1).map(move |&b| (a, b)))
        .filter(|&(a, b)| grafo.ha_arco(a, b))
        .count();
    Some(presenti as f64 / possibili as f64)
}

/// Il coefficiente di clustering di ogni nodo.
pub fn clusterings(grafo: &Graph) -> Vec<(NodeId, Option<f64>)> {
    grafo.nodi
        .iter()
        .map(|&n| (n, clustering(grafo, n)))
        .collect()
}

/// La lista ordinata degli archi di un nodo (per ispezione).
pub fn archi_di(grafo: &Graph, id: NodeId) -> Vec<&Edge> {
    grafo.archi
        .iter()
        .filter(|e| e.from == id || e.to == id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, GraphConfig};

    fn grafo_test() -> Graph {
        //  1 — 2 — 3
        //  |       |
        //  4 — 5   6
        // Un grafo a stella+catena per testare grado, clustering, orfani.
        let archi = [
            (1, 2, 0.9),
            (2, 3, 0.8),
            (1, 4, 0.7),
            (4, 5, 0.6),
            (3, 6, 0.5),
        ];
        let mut nodi: Vec<NodeId> = (1..=6).map(NodeId).collect();
        nodi.sort_unstable();
        let mut g = Graph::new(GraphConfig::default());
        g.nodi = nodi;
        g.archi = archi
            .iter()
            .map(|&(a, b, s)| Edge {
                from: NodeId(a),
                to: NodeId(b),
                score: s,
                dense: s,
                sparse: s,
                colbert: s,
            })
            .collect();
        g
    }

    #[test]
    fn gradi_ok() {
        let g = grafo_test();
        assert_eq!(grado(&g, NodeId(1)), 2); // 2 e 4
        assert_eq!(grado(&g, NodeId(2)), 2); // 1 e 3
        assert_eq!(grado(&g, NodeId(6)), 1); // solo 3
    }

    #[test]
    fn nessun_orfano_nel_grafo_connesso() {
        let g = grafo_test();
        assert!(orfani(&g).is_empty());
    }

    #[test]
    fn orfano_isolato() {
        let mut g = grafo_test();
        g.nodi.push(NodeId(7)); // isolato
        g.nodi.sort_unstable();
        assert_eq!(orfani(&g), vec![NodeId(7)]);
    }

    #[test]
    fn clustering_completo() {
        // Un triangolo: 1—2, 2—3, 1—3. Clustering di 1 = 1.0.
        // (Gli archi devono essere ordinati per coppia (from, to): ha_arco
        // usa la ricerca binaria.)
        let archi = [(1, 2, 0.9), (1, 3, 0.7), (2, 3, 0.8)];
        let mut g = Graph::new(GraphConfig::default());
        g.nodi = vec![NodeId(1), NodeId(2), NodeId(3)];
        g.archi = archi
            .iter()
            .map(|&(a, b, s)| Edge {
                from: NodeId(a),
                to: NodeId(b),
                score: s,
                dense: s,
                sparse: s,
                colbert: s,
            })
            .collect();
        assert_eq!(clustering(&g, NodeId(1)), Some(1.0));
    }

    #[test]
    fn clustering_ponte() {
        // 1—2—3: 2 è un ponte, i suoi vicini (1 e 3) non sono collegati.
        let archi = [(1, 2, 0.9), (2, 3, 0.8)];
        let mut g = Graph::new(GraphConfig::default());
        g.nodi = vec![NodeId(1), NodeId(2), NodeId(3)];
        g.archi = archi
            .iter()
            .map(|&(a, b, s)| Edge {
                from: NodeId(a),
                to: NodeId(b),
                score: s,
                dense: s,
                sparse: s,
                colbert: s,
            })
            .collect();
        assert_eq!(clustering(&g, NodeId(2)), Some(0.0));
        assert_eq!(clustering(&g, NodeId(1)), None); // un solo vicino
    }
}