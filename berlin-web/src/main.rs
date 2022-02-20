use std::env;
use std::sync::Arc;
use std::time::Instant;

use axum::routing::get;
use axum::{AddExtensionLayer, Router};
use structopt::StructOpt;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use berlin_core::locations_db;
use berlin_core::locations_db::LocationsDb;
use berlin_core::search::SearchTerm;
use berlin_web::{init_logging, search_handler};

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
    let db = locations_db::parse_json_files(data_dir);

    if args.interactive {
        loop {
            cli_search_query(&db)
        }
    } else {
        let db = Arc::new(db);
        let app = Router::new()
            .route("/search", get(search_handler::search_handler))
            .route("/search-schema", get(search_handler::search_schema_handler))
            .route("/health", get(health_check_handler))
            .layer(AddExtensionLayer::new(db))
            .layer(TraceLayer::new_for_http());
        let addr = "0.0.0.0:3001";
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
