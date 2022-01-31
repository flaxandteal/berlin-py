use serde::Serialize;
use strsim::normalized_levenshtein as similarity_algo;
use ustr::Ustr;

use crate::locations_db::LocationsDb;
use crate::{dedup, SCORE_SOFT_MAX};

const STOP_WORDS: [&str; 10] = [
    "at", "to", "in", "on", "of", "for", "by", "and", "was", "did",
];

#[derive(Debug, Default, Serialize)]
pub struct NGrams {
    pub(crate) words: Vec<String>,
    pub(crate) doublets: Vec<String>,
    pub(crate) triplets: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchTerm {
    pub raw: String,
    pub normalized: String,
    pub stop_words: Vec<Ustr>,
    pub codes: Vec<Ustr>,
    pub exact_matches: Vec<Ustr>,
    pub not_exact_matches: NGrams,
}

impl SearchTerm {
    pub fn from_raw_query(raw: String, db: &LocationsDb) -> Self {
        let normalized = crate::normalize(&raw);
        let mut codes: Vec<Ustr> = vec![];
        let mut exact_matches: Vec<Ustr> = Vec::default();
        let mut not_exact_matches: NGrams = NGrams::default();
        let split_words = normalized.split(" ").collect::<Vec<_>>();
        let stop_words = split_words
            .iter()
            .filter_map(|w| Ustr::from_existing(w).filter(|w| STOP_WORDS.contains(&w.as_str())))
            .collect();
        let stop_words = dedup(stop_words);
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
                    0 | 1 => {}                                 // ignore
                    _ if stop_words.contains(&known_ustr) => {} // ignore stop words
                    2 | 3 => {
                        codes.push(known_ustr);
                        exact_matches.push(known_ustr)
                    }
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
            stop_words,
            codes: dedup(codes),
            exact_matches: dedup(exact_matches),
            not_exact_matches,
        }
    }
    pub fn match_str(&self, subject: &str) -> i64 {
        let exact = self
            .exact_matches
            .iter()
            .filter_map(|m| match m == &subject {
                true => Some(SCORE_SOFT_MAX + m.len() as i64),
                false => None,
            })
            .max();
        match exact {
            Some(s) => s,
            None => {
                let words_count = subject.split(" ").count();
                match words_count {
                    0 => 0,
                    1 => self
                        .not_exact_matches
                        .words
                        .iter()
                        .map(
                            |w| match w.len() > subject.len() - 2 && w.len() < subject.len() + 2 {
                                true => {
                                    (similarity_algo(subject, &w) * SCORE_SOFT_MAX as f64) as i64
                                }
                                false => 0,
                            },
                        )
                        .max()
                        .unwrap_or(0),
                    2 => max_match(subject, &self.not_exact_matches.doublets),
                    _ => max_match(subject, &self.not_exact_matches.triplets),
                }
            }
        }
    }
}

fn max_match(subject: &str, terms: &[String]) -> i64 {
    terms
        .iter()
        .map(|w| ((similarity_algo(subject, &w) * SCORE_SOFT_MAX as f64) as i64))
        .max()
        .unwrap_or(0)
}
