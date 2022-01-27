use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use smallvec::{smallvec, SmallVec};
use tracing::info;
use ustr::{Ustr, UstrMap, UstrSet};

use crate::graph::ResultsGraph;
use crate::location::Location;
use crate::search::SearchTerm;
use crate::{SCORE_SOFT_MAX, SEARCH_INCLUSION_THRESHOLD};

pub struct LocationsDb {
    pub stop_words_english: UstrSet,
    pub all: UstrMap<Location>,
    // name to loc key
    pub names_registry: UstrMap<SmallVec<[Ustr; 4]>>,
}

impl LocationsDb {
    pub fn new() -> Self {
        let sw = stop_words::get(stop_words::LANGUAGE::English);
        let sw = sw.iter().map(|w| w.as_str().into()).collect();
        LocationsDb {
            stop_words_english: sw,
            all: Default::default(),
            names_registry: Default::default(),
        }
    }
    pub fn insert(&mut self, l: Location) {
        let mut loc_names = l.get_names();
        let loc_words: Vec<Ustr> = loc_names
            .iter()
            .map(|n| {
                let words = n.split(" ").collect::<Vec<_>>();
                words.into_iter().filter_map(|w| match w.len() > 3 {
                    true => Some(w.into()),
                    false => None,
                })
            })
            .flatten()
            .collect();
        loc_names.extend(loc_words);
        loc_names.iter().for_each(|n| {
            match self.names_registry.get_mut(n) {
                None => {
                    self.names_registry.insert(*n, smallvec![l.key]);
                }
                Some(names) => names.push(l.key),
            };
        });
        self.all.insert(l.key, l);
    }
    pub fn find_by_name(&self, name: &Ustr) -> SmallVec<[Ustr; 4]> {
        match self.names_registry.get(name) {
            None => smallvec![],
            Some(s) => s.clone(),
        }
    }
    pub fn find_by_names(&self, names: &[Ustr]) -> Vec<Ustr> {
        names
            .iter()
            .map(|c| self.find_by_name(c))
            .flatten()
            .collect::<Vec<_>>()
    }
    pub fn get_codes(&self, key: &Ustr) -> Vec<Ustr> {
        match self.all.get(&key) {
            None => vec![],
            Some(loc) => loc.get_codes(),
        }
    }
    pub fn search(&self, st: &SearchTerm, limit: usize) -> Vec<(Ustr, i64)> {
        let exact_locations = self
            .find_by_names(&st.exact_matches)
            .into_iter()
            .map(|l| (l, SCORE_SOFT_MAX))
            .collect::<Vec<_>>();
        info!("Exact locations: {}", exact_locations.len());
        let mut res = if exact_locations.len() >= limit {
            exact_locations
        } else {
            let mut r = self
                .all
                .par_iter()
                .filter_map(|(key, loc)| {
                    let score = loc.search(&st);
                    match score > SEARCH_INCLUSION_THRESHOLD {
                        true => Some((*key, score)),
                        false => None,
                    }
                })
                .collect::<Vec<_>>();
            // r.truncate(10000);
            r.extend(exact_locations);
            r.into_iter().collect::<UstrMap<_>>().into_iter().collect()
        };
        let res_graph = ResultsGraph::from_results(&res, &self);
        res.extend(res_graph.scores.into_iter().collect::<Vec<_>>());
        res.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        res.truncate(limit);
        res
    }
}
