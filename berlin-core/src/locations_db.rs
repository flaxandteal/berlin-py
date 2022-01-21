use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use smallvec::{smallvec, SmallVec};
use ustr::{Ustr, UstrMap, UstrSet};

use crate::location::Location;
use crate::search::SearchTerm;
use crate::{LOC_CODE_BOOST, SCORE_SOFT_MAX, SEARCH_INCLUSION_THRESHOLD};

const MAX_RESULTS_COUNT: usize = 1000;

#[derive(Default)]
pub struct LocationsDb {
    pub all: UstrMap<Location>,
    pub names_registry: UstrMap<SmallVec<[Ustr; 4]>>, // the values are references to self.all keys
}

impl LocationsDb {
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
    pub fn boost_by_codes(
        &self,
        search_results: Vec<(Ustr, u64)>,
        codes: &UstrSet,
    ) -> Vec<(Ustr, u64)> {
        let mut boosted: Vec<(Ustr, u64)> = search_results
            .into_iter()
            .map(|(key, score)| {
                let loc = self.all.get(&key).expect("should be in the db");
                let matches: u64 = codes
                    .iter()
                    .map(|code| match loc.code_match(*code) {
                        true => LOC_CODE_BOOST,
                        false => 0,
                    })
                    .sum();
                (key, score + matches)
            })
            .collect();
        boosted.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        boosted
    }
    pub fn search(&self, st: SearchTerm, limit: usize) -> Vec<(Ustr, u64)> {
        let locations_by_code = self.find_by_names(&st.codes);
        let codes = locations_by_code
            .iter()
            .map(|l| self.get_codes(l))
            .flatten()
            .collect::<UstrSet>();
        let exact_locations = self
            .find_by_names(&st.exact_matches)
            .into_iter()
            .map(|l| (l, SCORE_SOFT_MAX))
            .collect::<Vec<_>>();
        let mut exact_locations = self.boost_by_codes(exact_locations, &codes);
        if exact_locations.len() >= limit {
            exact_locations.truncate(limit);
            return exact_locations;
        }
        let res = self.all.par_iter().filter_map(|(key, loc)| {
            let score = loc.search(&st);
            match score > SEARCH_INCLUSION_THRESHOLD {
                true => Some((*key, score)),
                false => None,
            }
        });
        let mut res = res.collect::<Vec<_>>();
        res.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        res.truncate(MAX_RESULTS_COUNT);
        let res = self.boost_by_codes(res, &codes);
        exact_locations.extend(res);
        exact_locations.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        exact_locations.truncate(limit);
        exact_locations
    }
}
