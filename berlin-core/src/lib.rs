use std::collections::HashSet;
use std::hash::Hash;

pub use deunicode;
pub use rayon;
pub use stop_words;

mod graph;
pub mod location;
pub mod locations_db;
pub mod search;

const SCORE_SOFT_MAX: i64 = 1000;
const SEARCH_INCLUSION_THRESHOLD: i64 = 500;
const GRAPH_EDGE_THRESHOLD: i64 = 600;
const STOP_WORDS_PENALTY: i64 = 10;

pub fn normalize(s: &str) -> String {
    deunicode::deunicode(s).to_lowercase()
}

pub fn dedup<T: Eq + Hash>(vec: Vec<T>) -> Vec<T> {
    vec.into_iter()
        .collect::<HashSet<T>>()
        .into_iter()
        .collect()
}
