use std::cmp::max;

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

pub fn search(cb: &CodeBank, search_term: String, limit: usize) -> Vec<(Ustr, u64)> {
    let re_str = format!(r"(?i)\b{}\b", search_term);
    let re = Regex::new(&*re_str).unwrap();
    let res = cb.all.par_iter().filter_map(|(key, loc)| {
        let score = loc.search(&search_term, &re);
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

pub fn search_in_string(subject: &str, search_term: &str, re: &Regex) -> u64 {
    let similarity = (similarity_algo(subject, search_term) * 1000.) as u64;
    match similarity > 500 {
        true => match re.is_match(subject) {
            true if search_term.len() == subject.len() => max(990, similarity),
            true => max(950, similarity),
            false => similarity,
        },
        false => similarity,
    }
}
