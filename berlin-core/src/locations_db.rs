use std::time::Instant;

use fst::{Automaton, IntoStreamer, Streamer};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::info;
use ustr::{Ustr, UstrMap, UstrSet};

use crate::graph::ResultsGraph;
use crate::location::Location;
use crate::search::SearchTerm;
use crate::SEARCH_INCLUSION_THRESHOLD;

#[derive(Default)]
pub struct LocationsDb {
    pub all: UstrMap<Location>,
    pub by_word_map: UstrMap<UstrSet>,
    pub by_word_vec: Vec<(Ustr, UstrSet)>,
    pub fst: fst::Map<Vec<u8>>,
}

impl LocationsDb {
    pub fn insert(&mut self, l: Location) {
        self.all.insert(l.key, l);
    }
    pub fn mk_fst(self) -> Self {
        let mut words_map: UstrMap<UstrSet> = UstrMap::default();
        self.all.iter().for_each(|(key, loc)| {
            let codes = loc.get_codes();
            let names = loc.get_names();
            let words_iter = loc.words.iter().chain(codes.iter()).chain(names.iter());
            words_iter.for_each(|w| {
                let old = match words_map.get_mut(w) {
                    None => {
                        let new = UstrSet::default();
                        words_map.insert(*w, new);
                        words_map.get_mut(w).unwrap()
                    }
                    Some(set) => set,
                };
                old.insert(*key);
            })
        });
        let mut words_vec = words_map
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect::<Vec<_>>();
        words_vec.sort_unstable_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        let fst = fst::Map::from_iter(
            self.by_word_vec
                .iter()
                .enumerate()
                .map(|(i, (word, _))| (word.as_str(), i as u64)),
        )
        .expect("Build FST");
        LocationsDb {
            all: self.all,
            by_word_map: words_map,
            by_word_vec: words_vec,
            fst,
        }
    }
    pub fn search(&self, st: &SearchTerm, limit: usize) -> Vec<(Ustr, i64)> {
        let mut pre_filtered: UstrSet = UstrSet::default();
        st.exact_matches.iter().for_each(|term| {
            if let Some(locs) = self.by_word_map.get(term) {
                pre_filtered.extend(locs);
            };
        });
        let not_exact = st.not_exact_matches.iter().map(|ne| ne.as_str());
        not_exact.for_each(|term| {
            if term.len() > 3 {
                let prefix_matcher = fst::automaton::Str::new(term).starts_with();
                let union = fst::automaton::Levenshtein::new(term, 2)
                    .expect("build automaton")
                    .union(prefix_matcher);
                let mut stream = self.fst.search(union).into_stream();
                while let Some((_, v)) = stream.next() {
                    let (_, locs) = self.by_word_vec.get(v as usize).unwrap();
                    pre_filtered.extend(locs);
                }
            }
        });
        let res = pre_filtered
            .par_iter()
            .filter_map(|key| {
                let loc = self.all.get(key).unwrap();
                let score = loc.search(&st);
                match score > SEARCH_INCLUSION_THRESHOLD || pre_filtered.len() < 1000 {
                    true => Some((*key, score)),
                    false => None,
                }
            })
            .collect::<UstrMap<_>>();
        let res_graph = ResultsGraph::from_results(res, &self);
        let mut res = res_graph.scores.into_iter().collect::<Vec<_>>();
        res.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        res.truncate(limit);
        res
    }
}
