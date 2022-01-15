use std::fs::File;
use std::io::BufReader;

use serde_json::Value;
use structopt::StructOpt;
use tracing::{error, info};

use berlin_core::json_decode::AnyLocationCode;
use berlin_web::init_logging;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(long = "log-level", case_insensitive = true, default_value = "INFO")]
    log_level: tracing::Level,
}

fn main() {
    let args = CliArgs::from_args();
    init_logging(args.log_level);

    let files = vec![
        "berlin-state.json",
        "berlin-subdivision.json",
        "berlin-locode.json",
        "berlin-iata.json",
    ];
    let caches = dirs::cache_dir().expect("caches dir not found");
    let app_cache = caches.join("berlin");
    for file in files {
        let path = app_cache.join(file);
        info!("file {path:?}");
        let fo = File::open(path).expect("cannot open json file");
        let reader = BufReader::new(fo);
        let json: serde_json::Value = serde_json::from_reader(reader).expect("cannot decode json");
        match json {
            Value::Object(obj) => {
                for (id, val) in obj {
                    let raw_any = serde_json::from_value::<AnyLocationCode>(val)
                        .expect("Cannot decode location code");
                    let loc = raw_any.dispatch();
                    match loc {
                        Ok(loc) => {} // not implemented
                        Err(err) => {
                            error!("Error for: {} {:?}", id, err);
                        }
                    }
                }
            }
            other => panic!(
                "Expected a JSON object, found other json structure: {:?}",
                other
            ),
        }
    }
}
