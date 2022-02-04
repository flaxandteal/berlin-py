use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use axum::extract::{Extension, Query};
use axum::routing::get;
use axum::{AddExtensionLayer, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use structopt::StructOpt;
use tower_http::trace::TraceLayer;
use tracing::log::warn;
use tracing::{error, info};

use berlin_core::location::{AnyLocation, Location};
use berlin_core::locations_db::LocationsDb;
use berlin_core::rayon::iter::IntoParallelIterator;
use berlin_core::rayon::prelude::*;
use berlin_core::search::SearchTerm;
use berlin_web::init_logging;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(long = "log-level", case_insensitive = true, default_value = "INFO")]
    log_level: tracing::Level,
    #[structopt(long = "interactive", short = "i")]
    interactive: bool,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::from_args();
    init_logging(args.log_level);

    let current_dir = env::current_dir().expect("get current directory");
    let data_dir = current_dir.join("data");
    let db = parse_json_files(data_dir);

    if args.interactive {
        loop {
            cli_search_query(&db)
        }
    } else {
        let db = Arc::new(db);
        let app = Router::new()
            .route("/search", get(search_handler))
            .route("/health", get(health_check_handler))
            .layer(AddExtensionLayer::new(db))
            .layer(TraceLayer::new_for_http());
        let addr = "0.0.0.0:3000";
        info!("Will listen on {addr}");
        axum::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

async fn health_check_handler() -> &'static str {
    "OK"
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: String,
    limit: Option<usize>,
    state: Option<String>,
}

#[derive(Serialize)]
struct SearchResults {
    time: String,
    query: SearchTerm,
    results: Vec<SearchResult>,
}

#[derive(Serialize)]
struct SearchResult {
    loc: Location,
    score: i64,
}

async fn search_handler(
    Query(params): Query<SearchParams>,
    Extension(state): Extension<Arc<LocationsDb>>,
) -> Json<SearchResults> {
    let start_time = Instant::now();
    let limit = params.limit.unwrap_or(1);
    let st = SearchTerm::from_raw_query(params.q, params.state);
    let results = state
        .search(&st, limit)
        .into_iter()
        .map(|(key, score)| {
            let loc: Location = state.all.get(&key).cloned().expect("loc should be in db");
            SearchResult { loc, score }
        })
        .collect();
    Json(SearchResults {
        time: format!("{:.3?}", start_time.elapsed()),
        query: st,
        results,
    })
}

fn cli_search_query(db: &LocationsDb) {
    let inp: String = promptly::prompt("Search Term").expect("Search term expected");
    let start = Instant::now();
    let term = SearchTerm::from_raw_query(inp, None);
    info!("Parse query in {:.3?}", start.elapsed());
    warn!("TERM: {term:#?}");
    let start = Instant::now();
    let res = db.search(&term, 5);
    for (i, (loc_key, score)) in res.iter().enumerate() {
        info!(
            "Result #{i} {loc_key:?} score: {score} {:?}",
            &db.all.get(&loc_key).unwrap().data
        );
    }
    warn!("Search took {:.2?}", start.elapsed());
    println!("\n\n");
}

fn parse_json_files(data_dir: PathBuf) -> LocationsDb {
    let files = vec!["state.json", "subdivision.json", "locode.json", "iata.json"];
    let start = Instant::now();
    let db = LocationsDb::default();
    let db = RwLock::new(db);
    files.into_par_iter().for_each(|file| {
        let path = data_dir.join(file);
        info!("Path {path:?}");
        let fo = File::open(path).expect("cannot open json file");
        let reader = BufReader::new(fo);
        let json: serde_json::Value = serde_json::from_reader(reader).expect("cannot decode json");
        info!("Decode json file {file}: {:.2?}", start.elapsed());
        match json {
            Value::Object(obj) => {
                let iter = obj.into_iter().par_bridge();
                let codes = iter
                    .filter_map(|(id, val)| {
                        let raw_any = serde_json::from_value::<AnyLocation>(val)
                            .expect("Cannot decode location code");
                        let loc = Location::from_raw(raw_any);
                        match loc {
                            Ok(loc) => Some(loc),
                            Err(err) => {
                                error!("Error for: {id} {err:?}");
                                None
                            }
                        }
                    })
                    .for_each(|l| {
                        let mut db = db.write().expect("cannot aquire lock");
                        db.insert(l);
                    });
                info!("{file} decoded to native structs: {:.2?}", start.elapsed());
                codes
            }
            other => panic!("Expected a JSON object: {other:?}"),
        }
    });
    let db = db.into_inner().expect("rw lock extract");
    info!(
        "parsed {} locations in: {:.2?}",
        db.all.len(),
        start.elapsed()
    );
    db
}
