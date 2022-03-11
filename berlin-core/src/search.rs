use serde::Serialize;
use strsim::normalized_levenshtein as similarity_algo;
use unicode_segmentation::UnicodeSegmentation;
use ustr::Ustr;

use crate::{dedup, SCORE_SOFT_MAX};

const STOP_WORDS: [&str; 11] = [
    "at", "to", "in", "on", "of", "for", "by", "and", "was", "did", "the",
];

#[derive(Debug, Serialize)]
pub struct SearchTerm {
    pub raw: String,
    pub normalized: String,
    pub stop_words: Vec<Ustr>,
    pub codes: Vec<Ustr>,
    pub exact_matches: Vec<Ustr>,
    pub not_exact_matches: Vec<String>,
    pub state_filter: Option<Ustr>,
    pub limit: usize,
    pub lev_dist: u32,
}

impl SearchTerm {
    pub fn from_raw_query(
        raw: String,
        state_filter: Option<String>,
        limit: usize,
        lev_dist: u32,
    ) -> Self {
        let normalized = crate::normalize(&raw);
        let mut codes: Vec<Ustr> = vec![];
        let mut exact_matches: Vec<Ustr> = Vec::default();
        let mut not_exact_matches = vec![];
        let split_words: Vec<&str> = normalized.unicode_words().collect();
        let stop_words = split_words
            .iter()
            .filter_map(|w| Ustr::from_existing(w).filter(|w| STOP_WORDS.contains(&w.as_str())))
            .collect();
        let stop_words = dedup(stop_words);
        // info!("Split words: {:?}", split_words);
        for (i, w) in split_words.iter().enumerate() {
            if split_words.len() > i + 1 {
                let doublet: String = [w, split_words[i + 1]].join(" ");
                match Ustr::from_existing(&doublet) {
                    Some(u) => exact_matches.push(u),
                    None => not_exact_matches.push(doublet.clone()),
                }
                if split_words.len() > i + 2 {
                    let triplet = [&doublet, split_words[i + 2]].join(" ");
                    match Ustr::from_existing(&triplet) {
                        Some(u) => exact_matches.push(u),
                        None => {}
                    }
                }
            }
            match Ustr::from_existing(w) {
                None => not_exact_matches.push(w.to_string()),
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
            state_filter: state_filter.map(|s| Ustr::from_existing(&s)).flatten(),
            limit,
            lev_dist,
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
            None => self
                .not_exact_matches
                .iter()
                .map(|w| {
                    if w.len() > 3 && subject.starts_with(w) {
                        SCORE_SOFT_MAX + (2 * w.len() as i64)
                    } else {
                        match w.len() > subject.len() - 2 && w.len() < subject.len() + 2 {
                            true => (similarity_algo(subject, &w) * SCORE_SOFT_MAX as f64) as i64,
                            false => 0,
                        }
                    }
                })
                .max()
                .unwrap_or(0),
        }
    }
}
