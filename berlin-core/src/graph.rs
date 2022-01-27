use std::time::Instant;

use petgraph::graphmap::DiGraphMap;
use tracing::info;
use ustr::{Ustr, UstrMap};

use crate::location::{state_key, subdiv_key, LocData};
use crate::locations_db::LocationsDb;

pub struct ResultsGraph {
    scores: UstrMap<i64>,
    locs: DiGraphMap<Ustr, i64>,
}

impl ResultsGraph {
    pub fn from_results(results: &[(Ustr, i64)], db: &LocationsDb) -> Self {
        let start = Instant::now();
        let scores: UstrMap<i64> = results.iter().map(|(key, s)| (*key, *s)).collect();
        let mut graph: DiGraphMap<Ustr, _> = DiGraphMap::new();
        results.iter().for_each(|(key, score)| {
            let loc = db.all.get(key).expect("location in db");
            graph.add_node(loc.key);
            let (state_key, subdiv_key) = match loc.data {
                LocData::St(_) => (None, None),
                LocData::Subdv(sd) => (state_key(sd.supercode), None),
                LocData::Locd(l) => (
                    state_key(l.supercode),
                    l.subdivision_code
                        .map(|c| subdiv_key(l.supercode, c))
                        .flatten(),
                ),
                LocData::Airp(a) => (state_key(a.country), None),
            };
            // info!("{:?}:{:?} for {:?}", state_key, subdiv_key, loc.get_names());
            for key in [state_key, subdiv_key] {
                if let Some(superkey) = key {
                    if let Some(superkey_score) = scores.get(&superkey) {
                        let weight = (superkey_score + score) / 2;
                        graph.add_edge(superkey, loc.key, weight);
                    }
                }
            }
        });
        info!("nodes: {}", graph.node_count());
        info!("edges: {}", graph.edge_count());
        let mut edges = graph.all_edges().collect::<Vec<_>>();
        edges.sort_unstable_by(|a, b| a.2.cmp(b.2));
        for edge in edges {
            let loc = db.all.get(&edge.1).unwrap();
            // info!("locode: {:?}", loc);
            let names = loc.get_names();
            let functions = match loc.data {
                LocData::Locd(lc) => lc.function_code.as_str(),
                _ => "",
            };
            info!("Edge: {:?} - {:?} {}", edge, names, functions);
        }
        info!("Graph analysis in {:.3?}", start.elapsed());
        ResultsGraph {
            scores,
            locs: graph,
        }
    }
}
