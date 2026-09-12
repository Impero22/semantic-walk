use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DizionarioCelle {
    pub versione: String,
    #[serde(rename = "K")]
    pub k: usize,
    pub dim: usize,
    pub metric: String,
    pub centroidi: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
pub struct TraiettoriaFatto {
    pub celle: Vec<u16>,
    pub pos: Vec<u32>,
    pub conf: Vec<f32>,
}

pub type MappaTraiettorie = HashMap<String, TraiettoriaFatto>;

pub fn carica_dizionario<P: AsRef<Path>>(path: P) -> Result<DizionarioCelle, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let dizionario = serde_json::from_reader(reader)?;
    Ok(dizionario)
}

pub fn carica_traiettorie<P: AsRef<Path>>(path: P) -> Result<MappaTraiettorie, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let traiettorie = serde_json::from_reader(reader)?;
    Ok(traiettorie)
}

pub fn similarita_traiettorie(t1: &TraiettoriaFatto, t2: &TraiettoriaFatto) -> f32 {
    if t1.celle.is_empty() || t2.celle.is_empty() {
        return 0.0;
    }

    let mut map1: HashMap<u16, f32> = HashMap::new();
    for (&c, &conf) in t1.celle.iter().zip(t1.conf.iter()) {
        *map1.entry(c).or_insert(0.0) += conf;
    }

    let mut map2: HashMap<u16, f32> = HashMap::new();
    for (&c, &conf) in t2.celle.iter().zip(t2.conf.iter()) {
        *map2.entry(c).or_insert(0.0) += conf;
    }

    let mut intersezione = 0.0f32;
    let mut unione = 0.0f32;

    let tutte_celle: HashSet<_> = map1.keys().chain(map2.keys()).copied().collect();

    for c in tutte_celle {
        let w1 = map1.get(&c).copied().unwrap_or(0.0);
        let w2 = map2.get(&c).copied().unwrap_or(0.0);
        intersezione += w1.min(w2);
        unione += w1.max(w2);
    }

    if unione == 0.0 {
        0.0
    } else {
        intersezione / unione
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarita_traiettorie_identiche() {
        let t = TraiettoriaFatto {
            celle: vec![107, 229, 15],
            pos: vec![0, 1, 2],
            conf: vec![0.9, 0.8, 0.95],
        };
        let sim = similarita_traiettorie(&t, &t);
        assert!((sim - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_similarita_traiettorie_disgiunte() {
        let t1 = TraiettoriaFatto {
            celle: vec![1, 2],
            pos: vec![0, 1],
            conf: vec![0.9, 0.9],
        };
        let t2 = TraiettoriaFatto {
            celle: vec![3, 4],
            pos: vec![0, 1],
            conf: vec![0.9, 0.9],
        };
        let sim = similarita_traiettorie(&t1, &t2);
        assert_eq!(sim, 0.0);
    }
}
