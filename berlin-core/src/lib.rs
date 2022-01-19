pub use deunicode;
pub use rayon;
use rayon::prelude::*;
use strsim::jaro_winkler as similarity_algo;
use tracing::info;
pub use ustr;
use ustr::{Ustr, UstrSet};

use locations_db::LocationsDb;

use crate::json_decode::Location;

pub mod json_decode;
pub mod locations_db;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub fn search(cb: &LocationsDb, st: SearchTerm, limit: usize) -> Vec<(Ustr, u64)> {
    let locations_by_code = cb.find_by_name_many(&st.codes);
    let codes = locations_by_code
        .iter()
        .map(|l| cb.get_codes(l))
        .flatten()
        .collect::<UstrSet>();
    info!("Codes for locations found: {:?}", codes);
    let exact_locations = cb
        .find_by_name_many(&st.exact_matches)
        .into_iter()
        .map(|l| (l, 1000))
        .collect::<Vec<_>>();
    // info!("Exact locations found: {:?}", exact_locations);
    let mut exact_locations = cb.boost_by_codes(exact_locations, &codes);
    // info!("Exact locations found: {:?}", exact_locations);
    if exact_locations.len() >= limit {
        exact_locations.truncate(limit);
        return exact_locations;
    }
    let res = cb.all.par_iter().filter_map(|(key, loc)| {
        let score = loc.search(&st);
        match score > 500 {
            true => Some((*key, score)),
            false => None,
        }
    });
    let mut res = res.collect::<Vec<_>>();
    res.sort_unstable_by(|a, b| b.1.cmp(&a.1));
    res.truncate(1000);
    let res = cb.boost_by_codes(res, &codes);
    exact_locations.extend(res);
    exact_locations.sort_unstable_by(|a, b| b.1.cmp(&a.1));
    exact_locations.truncate(limit);
    exact_locations
}

pub fn search_in_string(subject: &str, search_term: &SearchTerm) -> u64 {
    let words_count = subject.split(" ").count();
    match search_term.exact_matches.iter().any(|m| m == &subject) {
        true => 1000,
        false => match words_count {
            0 => 0,
            1 => search_term
                .not_exact_matches
                .words
                .iter()
                .map(
                    |w| match w.len() > subject.len() - 2 && w.len() < subject.len() + 2 {
                        true => (similarity_algo(subject, &w) * 1000.) as u64,
                        false => 0,
                    },
                )
                .max()
                .unwrap_or(0),
            2 => search_many_strings(subject, &search_term.not_exact_matches.doublets),
            _ => search_many_strings(subject, &search_term.not_exact_matches.triplets),
        },
    }
}

fn search_many_strings(subject: &str, terms: &[String]) -> u64 {
    terms
        .iter()
        .map(|w| ((similarity_algo(subject, &w) * 1000.) as u64))
        .max()
        .unwrap_or(0)
}

pub fn normalize(s: &str) -> String {
    deunicode::deunicode(s).to_lowercase()
}

#[derive(Debug)]
pub struct SearchTerm {
    pub raw: String,
    pub normalized: String,
    pub codes: Vec<Ustr>,
    pub exact_matches: Vec<Ustr>,
    pub not_exact_matches: Matches<String>,
}

#[derive(Debug, Default)]
pub struct Matches<T> {
    words: Vec<T>,
    doublets: Vec<T>,
    triplets: Vec<T>,
}

pub fn mk_search_term(raw: String) -> SearchTerm {
    let normalized = normalize(&raw);
    // let re_str = format!(r"(?i)\b{}\b", normalized);
    // let re = Regex::new(&*re_str).unwrap();
    let mut codes: Vec<Ustr> = vec![];
    let mut exact_matches: Vec<Ustr> = Vec::default();
    let mut not_exact_matches: Matches<String> = Matches::default();
    let split_words = normalized.split(" ").collect::<Vec<_>>();
    for (i, w) in split_words.iter().enumerate() {
        if split_words.len() > i + 1 {
            let doublet: String = [w, split_words[i + 1]].join(" ");
            match Ustr::from_existing(&doublet) {
                Some(u) => exact_matches.push(u),
                None => not_exact_matches.doublets.push(doublet.clone()),
            }
            if split_words.len() > i + 2 {
                let triplet = [&doublet, split_words[i + 2]].join(" ");
                match Ustr::from_existing(&triplet) {
                    Some(u) => exact_matches.push(u),
                    None => not_exact_matches.triplets.push(triplet),
                }
            }
        }
        match Ustr::from_existing(w) {
            None => not_exact_matches.words.push(w.to_string()),
            Some(known_ustr) => match w.len() {
                0 | 1 => {} // ignore
                2 | 3 => codes.push(known_ustr),
                _ => {
                    exact_matches.push(known_ustr);
                }
            },
        }
    }
    exact_matches.sort_unstable_by(|a, b| b.len().cmp(&a.len()));
    SearchTerm {
        raw,
        normalized,
        codes,
        exact_matches,
        not_exact_matches,
    }
}
