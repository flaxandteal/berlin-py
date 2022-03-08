use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use schemars::{schema_for, JsonSchema};
use serde::Serialize;
use serde_json::json;
use ustr::Ustr;

use berlin_core::locations_db::LocationsDb;

use crate::location_json::LocJson;

#[derive(Serialize, JsonSchema)]
pub struct FetchResults {
    time: String,
    result: LocJson,
}

fn get_loc(db: &LocationsDb, key: &str) -> Option<LocJson> {
    let key = Ustr::from_existing(key)?;
    let loc = db.all.get(&key)?;
    Some(LocJson::from_location(db, loc))
}

pub async fn fetch_handler(
    Path(key): Path<String>,
    Extension(db): Extension<Arc<LocationsDb>>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    match get_loc(&db, &key) {
        None => {
            let err_msg = format!("location not found '{}' ", &key);
            Err((StatusCode::NOT_FOUND, Json(json!({ "error": err_msg }))))
        }
        Some(loc) => Ok((
            StatusCode::OK,
            Json(FetchResults {
                time: format!("{:.2?}", start_time.elapsed()),
                result: loc,
            }),
        )),
    }
}

pub async fn fetch_schema_handler() -> String {
    let schema = schema_for!(FetchResults);
    serde_json::to_string(&schema).expect("json schema")
}
