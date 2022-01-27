use std::cmp::min;
use std::time::Instant;

use petgraph::graphmap::DiGraphMap;
use tracing::info;
use ustr::{Ustr, UstrMap};

use crate::location::LocData;
use crate::locations_db::LocationsDb;
use crate::GRAPH_EDGE_THRESHOLD;

pub struct ResultsGraph {
    pub(crate) scores: UstrMap<i64>,
    locs: DiGraphMap<Ustr, (i64, i64)>,
}

impl ResultsGraph {
    pub fn from_results(results: &[(Ustr, i64)], db: &LocationsDb) -> Self {
        let start = Instant::now();
        let mut scores: UstrMap<i64> = results.iter().map(|(key, s)| (*key, *s)).collect();
        let mut graph: DiGraphMap<Ustr, _> = DiGraphMap::new();
        results.iter().for_each(|(key, score)| {
            let loc = db.all.get(key).expect("location in db");
            graph.add_node(loc.key);
            let (state_key, subdiv_key) = loc.get_parents();
            // info!("{:?}:{:?} for {:?}", state_key, subdiv_key, loc.get_names());
            for key in [state_key, subdiv_key] {
                if let Some(superkey) = key {
                    if let Some(superkey_score) = scores.get(&superkey) {
                        if min(*superkey_score, *score) > GRAPH_EDGE_THRESHOLD {
                            let weight = (*superkey_score, *score);
                            graph.add_edge(superkey, loc.key, weight);
                        }
                    }
                }
            }
        });
        info!("nodes: {}", graph.node_count());
        info!("edges: {}", graph.edge_count());
        let mut edges = graph.all_edges().collect::<Vec<_>>();
        edges.sort_unstable_by(|a, b| b.2.cmp(a.2));
        for (i, edge) in edges.iter().enumerate() {
            let loc = db.all.get(&edge.1).unwrap();
            let parent = db.all.get(&edge.0).unwrap();
            scores.insert(loc.key, edge.2 .0 + edge.2 .1);
            // info!("locode: {:?}", loc);
            let loc_names = loc.get_names();
            let parent_names = parent.get_names();
            let functions = match loc.data {
                LocData::Locd(lc) => lc.function_code.as_str(),
                _ => "",
            };
            if i < 10 {
                info!(
                    "Edge: {:?} - {:?}-{:?} {}",
                    edge, parent_names, loc_names, functions
                );
            }
        }
        info!("Graph analysis in {:.3?}", start.elapsed());
        ResultsGraph {
            scores,
            locs: graph,
        }
    }
}
