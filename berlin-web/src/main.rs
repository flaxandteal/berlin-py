use std::env;
use std::sync::Arc;

use axum::routing::get;
use axum::{AddExtensionLayer, Router};
use structopt::StructOpt;
use tower_http::trace::TraceLayer;
use tracing::info;

use berlin_core::locations_db;
use berlin_web::fetch_handler::fetch_schema_handler;
use berlin_web::search_handler::search_schema_handler;
use berlin_web::{fetch_handler::fetch_handler, init_logging, search_handler::search_handler};

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(long = "log-level", case_insensitive = true, default_value = "INFO")]
    log_level: tracing::Level,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::from_args();
    init_logging(args.log_level);
    let current_dir = env::current_dir().expect("get current directory");
    let data_dir = current_dir.join("data");
    let db = locations_db::parse_data_files(data_dir);
    let db = Arc::new(db);
    let app = Router::new()
        .route("/berlin/search", get(search_handler))
        .route("/berlin/search-schema", get(search_schema_handler))
        .route("/berlin/code/:key", get(fetch_handler))
        .route("/berlin/fetch-schema", get(fetch_schema_handler))
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

async fn health_check_handler() -> &'static str {
    "OK"
}
