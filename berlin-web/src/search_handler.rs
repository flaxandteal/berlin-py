use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Extension, Query};
use axum::Json;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use berlin_core::location::{Location, LocData};
use berlin_core::locations_db::LocationsDb;
use berlin_core::search::SearchTerm;
use berlin_core::smallvec::SmallVec;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: String,
    limit: Option<usize>,
    state: Option<String>,
    extended: Option<bool>
}

#[derive(Serialize, JsonSchema)]
pub struct SearchResults {
    time: String,
    query: SearchTermJson,
    results: Vec<SearchResult>,
}

#[derive(Serialize, JsonSchema)]
pub struct SearchResult {
    pub loc: ResLocation,
    pub score: i64,
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
}

impl SearchTermJson {
    fn from_search_term(t: SearchTerm) -> Self {
        SearchTermJson {
            raw: t.raw,
            normalized: t.normalized,
            stop_words: t.stop_words.into_iter().map(|u| u.as_str()).collect(),
            codes: t.codes.into_iter().map(|u| u.as_str()).collect(),
            exact_matches: t.exact_matches.into_iter().map(|u| u.as_str()).collect(),
            not_exact_matches: t.not_exact_matches,
            state_filter: t.state_filter.map(|u| u.as_str()),
        }
    }
}

#[derive(Serialize, JsonSchema)]
pub struct ResLocation {
    encoding: &'static str,
    id: &'static str,
    names: SmallVec<[&'static str; 1]>,
    codes: SmallVec<[&'static str; 1]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    subdiv: Option<&'static str>
}

impl ResLocation {
    pub fn from_location(l: &Location, extended: bool) -> Self {
        let parents = l.get_parents();
        Self {
            encoding: l.encoding.as_str(),
            id: l.id.as_str(),
            names: l.get_names().into_iter().map(|u| u.as_str()).collect(),
            codes: l.get_codes().into_iter().map(|u| u.as_str()).collect(),
            state: match (l.data, parents.0, extended) {
                (LocData::St(_), _, true) => Some(l.key.as_str()),
                (_, Some(state), true) => Some(state.as_str()),
                _ => None
            },
            subdiv: match (l.data, parents.1, extended) {
                (LocData::Subdv(_), _, true) => Some(l.key.as_str()),
                (_, Some(subdiv), true) => Some(subdiv.as_str()),
                _ => None
            }
        }
    }
}

pub async fn search_handler(
    Query(params): Query<SearchParams>,
    Extension(state): Extension<Arc<LocationsDb>>,
) -> Json<SearchResults> {
    let start_time = Instant::now();
    let limit = params.limit.unwrap_or(1);
    let st = SearchTerm::from_raw_query(params.q, params.state);
    let extended = params.extended.is_some() && params.extended.unwrap();
    let results = state
        .search(&st, limit)
        .into_iter()
        .map(|(key, score)| {
            let loc: Location = state.all.get(&key).cloned().expect("loc should be in db");
            SearchResult {
                loc: ResLocation::from_location(&loc, extended),
                score: score,
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
