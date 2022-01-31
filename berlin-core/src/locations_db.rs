use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::info;
use ustr::{Ustr, UstrMap};

use crate::graph::ResultsGraph;
use crate::location::Location;
use crate::search::SearchTerm;
use crate::SEARCH_INCLUSION_THRESHOLD;

#[derive(Default)]
pub struct LocationsDb {
    pub all: UstrMap<Location>,
}

impl LocationsDb {
    pub fn insert(&mut self, l: Location) {
        self.all.insert(l.key, l);
    }
    pub fn search(&self, st: &SearchTerm, limit: usize) -> Vec<(Ustr, i64)> {
        let res = self
            .all
            .par_iter()
            .filter_map(|(key, loc)| {
                let score = loc.search(&st);
                match score > SEARCH_INCLUSION_THRESHOLD {
                    true => Some((*key, score)),
                    false => None,
                }
            })
            .collect::<UstrMap<_>>();
        info!("Found locations: {}", res.len());
        let res_graph = ResultsGraph::from_results(res, &self);
        let mut res = res_graph.scores.into_iter().collect::<Vec<_>>();
        res.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        info!("RES: {:?}", res[..10].to_vec());
        res.truncate(limit);
        res
    }
}
