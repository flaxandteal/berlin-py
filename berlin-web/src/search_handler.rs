use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Extension, Query};
use axum::Json;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use berlin_core::location::Location;
use berlin_core::locations_db::LocationsDb;
use berlin_core::search::{Offset, SearchTerm};

use crate::location_json::LocJson;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: String,
    limit: Option<usize>,
    state: Option<String>,
    ld: Option<u32>,
}

#[derive(Serialize, JsonSchema)]
pub struct SearchResults {
    time: String,
    query: SearchTermJson,
    results: Vec<SearchResult>,
}

#[derive(Serialize, JsonSchema)]
pub struct SearchResult {
    pub loc: LocJson,
    pub score: i64,
    pub offset: Offset,
}

#[derive(Serialize, JsonSchema)]
pub struct SearchTermJson {
    pub raw: String,
    pub normalized: String,
    pub stop_words: Vec<&'static str>,
    pub codes: Vec<&'static str>,
    pub exact_matches: Vec<&'static str>,
    pub not_exact_matches: Vec<String>,
    pub state_filter: Option<&'static str>,
    pub limit: usize,
    pub levenshtein_distance: usize,
}

impl SearchTermJson {
    fn from_search_term(t: SearchTerm) -> Self {
        SearchTermJson {
            raw: t.raw,
            normalized: t.normalized,
            stop_words: t.stop_words.into_iter().map(|u| u.as_str()).collect(),
            codes: t.codes.into_iter().map(|u| u.term.as_str()).collect(),
            exact_matches: t
                .exact_matches
                .into_iter()
                .map(|u| u.term.as_str())
                .collect(),
            not_exact_matches: t.not_exact_matches.into_iter().map(|ne| ne.term).collect(),
            state_filter: t.state_filter.map(|u| u.as_str()),
            limit: t.limit,
            levenshtein_distance: t.lev_dist as usize,
        }
    }
}

pub async fn search_handler(
    Query(params): Query<SearchParams>,
    Extension(state): Extension<Arc<LocationsDb>>,
) -> Json<SearchResults> {
    let start_time = Instant::now();
    let limit = params.limit.unwrap_or(1);
    let lev_distance = match params.ld {
        None => 2,
        Some(ld) if ld > 2 => 2,
        Some(ld) => ld,
    };
    let st = SearchTerm::from_raw_query(params.q, params.state, limit, lev_distance);
    let results = state
        .search(&st)
        .into_iter()
        .map(|(key, score)| {
            let loc: Location = state.all.get(&key).cloned().expect("loc should be in db");
            SearchResult {
                loc: LocJson::from_location(&state, &loc),
                score: score.score,
                offset: score.offset,
            }
        })
        .collect();
    Json(SearchResults {
        time: format!("{:.2?}", start_time.elapsed()),
        query: SearchTermJson::from_search_term(st),
        results,
    })
}

pub async fn search_schema_handler() -> String {
    let schema = schema_for!(SearchResults);
    serde_json::to_string(&schema).expect("json schema")
}
