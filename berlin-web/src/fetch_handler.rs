use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;
use ustr::Ustr;

use axum::extract::{Extension, Query, Path};
use axum::response::IntoResponse;
use axum::http::StatusCode;

use axum::Json;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

use berlin_core::location::{Location, LocData};
use berlin_core::locations_db::LocationsDb;
use berlin_core::smallvec::SmallVec;

#[derive(Debug, Deserialize)]
pub struct FetchParams {
    extended: Option<bool>
}

#[derive(Serialize, JsonSchema)]
pub struct FetchResults {
    time: String,
    result: ResLocation,
}

#[derive(Serialize, JsonSchema)]
pub struct ResLocation {
    encoding: &'static str,
    id: &'static str,
    names: SmallVec<[&'static str; 1]>,
    codes: SmallVec<[&'static str; 1]>,
    state: Option<&'static str>,
    subdiv: Option<&'static str>
}

impl ResLocation {
    pub fn from_location(l: &Location, _extended: bool) -> Self {
        let parents = l.get_parents();
        Self {
            encoding: l.encoding.as_str(),
            id: l.id.as_str(),
            names: l.get_names().into_iter().map(|u| u.as_str()).collect(),
            codes: l.get_codes().into_iter().map(|u| u.as_str()).collect(),
            state: match (l.data, parents.0) {
                (LocData::St(_), _) => Some(l.key.as_str()),
                (_, Some(state)) => Some(state.as_str()),
                _ => None
            },
            subdiv: match (l.data, parents.1) {
                (LocData::Subdv(_), _) => Some(l.key.as_str()),
                (_, Some(subdiv)) => Some(subdiv.as_str()),
                _ => None
            }
        }
    }
}

pub async fn fetch_handler(
    Path(path_params): Path<HashMap<String, String>>,
    Query(params): Query<FetchParams>,
    Extension(state): Extension<Arc<LocationsDb>>,
) -> impl IntoResponse {
    if path_params.get("key").is_none() {
        return Err((StatusCode::NOT_FOUND, Json(json!({ "error": "Missing 'key' field" }))));
    }

    let start_time = Instant::now();
    let key = Ustr::from(&path_params.get("key").unwrap());
    let extended = params.extended.is_some() && params.extended.unwrap();
    let loc = match state.all.get(&key) {
        None => None,
        Some(loc) => Some(ResLocation::from_location(&loc, extended)),
    };

    match loc {
        None => Err((StatusCode::NOT_FOUND, Json(json!({ "key": key })))),
        _ => Ok((StatusCode::OK, Json(FetchResults {
            time: format!("{:.2?}", start_time.elapsed()),
            result: loc.unwrap(),
        })))
    }
}

pub async fn fetch_schema_handler() -> String {
    let schema = schema_for!(FetchResults);
    serde_json::to_string(&schema).expect("json schema")
}
