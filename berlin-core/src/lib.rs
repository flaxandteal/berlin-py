use std::cmp::max;

pub use deunicode;
pub use rayon;
use rayon::prelude::*;
use regex::Regex;
use strsim::jaro_winkler as similarity_algo;
pub use ustr;
use ustr::{Ustr, UstrMap};

use crate::json_decode::Location;

pub mod json_decode;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

#[derive(Default)]
pub struct CodeBank {
    pub all: UstrMap<Location>,
}

pub fn search(cb: &CodeBank, st: SearchTerm, limit: usize) -> Vec<(Ustr, u64)> {
    let res = cb.all.par_iter().filter_map(|(key, loc)| {
        let score = loc.search(&st);
        match score > 500 {
            true => Some((*key, score)),
            false => None,
        }
    });
    let mut res = res.collect::<Vec<_>>();
    res.sort_unstable_by(|a, b| b.1.cmp(&a.1));
    if res.len() > limit {
        res[..limit].to_vec()
    } else {
        res
    }
}

pub fn search_in_string(subject: &str, search_term: &SearchTerm) -> u64 {
    let similarity = (similarity_algo(subject, &search_term.normalized) * 1000.) as u64;
    match similarity > 500 {
        true => match search_term.re.is_match(subject) {
            true if search_term.normalized.len() == subject.len() => max(990, similarity),
            true => max(950, similarity),
            false => similarity,
        },
        false => similarity,
    }
}

pub fn normalize(s: &str) -> String {
    deunicode::deunicode(s).to_lowercase()
}

#[derive(Debug)]
pub struct SearchTerm {
    pub raw: String,
    pub normalized: String,
    pub codes: Vec<Ustr>,
    pub names: Vec<Ustr>,
    pub without_codes: String,
    pub re: Regex,
}

pub fn mk_search_term(raw: String) -> SearchTerm {
    let normalized = normalize(&raw);
    let re_str = format!(r"(?i)\b{}\b", normalized);
    let re = Regex::new(&*re_str).unwrap();
    let mut codes: Vec<Ustr> = vec![];
    let mut names: Vec<Ustr> = vec![];
    let mut without_codes: Vec<String> = vec![];
    normalized
        .split(" ")
        .for_each(|w| match Ustr::from_existing(w) {
            None => without_codes.push(w.to_string()),
            Some(known_ustr) => match w.len() {
                0 | 1 => {} // ignore
                2 | 3 => codes.push(known_ustr),
                _ => {
                    names.push(known_ustr);
                    without_codes.push(w.to_string())
                }
            },
        });
    SearchTerm {
        re,
        raw,
        normalized,
        codes,
        names,
        without_codes: without_codes.join(" "),
    }
}
