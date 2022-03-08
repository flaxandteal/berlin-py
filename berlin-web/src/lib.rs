pub mod fetch_handler;
mod location_json;
pub mod search_handler;

use tracing::level_filters::LevelFilter;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_log::LogTracer;

/// Register a subscriber as global default to process span data.
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn init_logging(log_level: tracing::Level) {
    let subscriber = tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(LevelFilter::from_level(log_level))
        .finish();
    init_subscriber(subscriber);
}
