use serde::Serialize;
use strsim::jaro_winkler as similarity_algo;
use ustr::Ustr;

use crate::SCORE_SOFT_MAX;

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
    pub codes: Vec<Ustr>,
    pub exact_matches: Vec<Ustr>,
    pub not_exact_matches: NGrams,
}

impl SearchTerm {
    pub fn from_raw_query(raw: String) -> Self {
        let normalized = crate::normalize(&raw);
        let mut codes: Vec<Ustr> = vec![];
        let mut exact_matches: Vec<Ustr> = Vec::default();
        let mut not_exact_matches: NGrams = NGrams::default();
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
    pub fn match_str(&self, subject: &str) -> u64 {
        let words_count = subject.split(" ").count();
        match self.exact_matches.iter().any(|m| m == &subject) {
            true => SCORE_SOFT_MAX,
            false => match words_count {
                0 => 0,
                1 => self
                    .not_exact_matches
                    .words
                    .iter()
                    .map(
                        |w| match w.len() > subject.len() - 2 && w.len() < subject.len() + 2 {
                            true => (similarity_algo(subject, &w) * SCORE_SOFT_MAX as f64) as u64,
                            false => 0,
                        },
                    )
                    .max()
                    .unwrap_or(0),
                2 => max_match(subject, &self.not_exact_matches.doublets),
                _ => max_match(subject, &self.not_exact_matches.triplets),
            },
        }
    }
}

fn max_match(subject: &str, terms: &[String]) -> u64 {
    terms
        .iter()
        .map(|w| ((similarity_algo(subject, &w) * SCORE_SOFT_MAX as f64) as u64))
        .max()
        .unwrap_or(0)
}
