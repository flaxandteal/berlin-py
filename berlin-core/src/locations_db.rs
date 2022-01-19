use ustr::{Ustr, UstrMap, UstrSet};

use crate::Location;

#[derive(Default)]
pub struct LocationsDb {
    pub all: UstrMap<Location>,
    pub names_registry: UstrMap<Vec<Ustr>>, // the values are references to self.all keys
}

impl LocationsDb {
    pub fn insert(&mut self, l: Location) {
        l.get_names()
            .iter()
            .for_each(|n| match self.names_registry.get_mut(n) {
                None => {
                    self.names_registry.insert(*n, vec![l.key]);
                }
                Some(names) => {
                    names.push(l.key);
                }
            });
        self.all.insert(l.key, l);
    }
    pub fn find_by_name(&self, name: &Ustr) -> Vec<Ustr> {
        match self.names_registry.get(name) {
            None => vec![],
            Some(s) => s.clone(),
        }
    }
    pub fn find_by_name_many(&self, names: &[Ustr]) -> Vec<Ustr> {
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
                // info!("{:#?}", loc);
                let matches: u64 = codes
                    .iter()
                    .map(|code| match loc.code_match(*code) {
                        true => 1,
                        false => 0,
                    })
                    .sum();
                (key, score + matches)
            })
            .collect();
        boosted.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        boosted
    }
}
