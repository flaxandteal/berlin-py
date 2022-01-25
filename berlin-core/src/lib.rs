pub use deunicode;
pub use rayon;

mod graph;
pub mod location;
pub mod locations_db;
pub mod search;

const SCORE_SOFT_MAX: u64 = 1000;
const SEARCH_INCLUSION_THRESHOLD: u64 = 500;
const LOC_CODE_BOOST: u64 = 1;

pub fn normalize(s: &str) -> String {
    deunicode::deunicode(s).to_lowercase()
}
