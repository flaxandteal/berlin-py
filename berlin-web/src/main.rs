use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use serde_json::Value;
use structopt::StructOpt;
use tracing::log::warn;
use tracing::{error, info};

use berlin_core::json_decode::{AnyLocationCode, Location};
use berlin_core::locations_db::LocationsDb;
use berlin_core::rayon::iter::IntoParallelIterator;
use berlin_core::rayon::prelude::*;
use berlin_core::{mk_search_term, search};
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

    let mut db = LocationsDb::default();
    let start = Instant::now();
    let codes_vectors = files.into_par_iter().map(|file| {
        let path = app_cache.join(file);
        info!("Path {path:?}");
        let fo = File::open(path).expect("cannot open json file");
        let reader = BufReader::new(fo);
        let json: serde_json::Value = serde_json::from_reader(reader).expect("cannot decode json");
        info!("Decode json file {file}: {:.2?}", start.elapsed());
        match json {
            Value::Object(obj) => {
                let iter = obj.into_iter().par_bridge();
                let codes = iter.filter_map(|(id, val)| {
                    let raw_any = serde_json::from_value::<AnyLocationCode>(val)
                        .expect("Cannot decode location code");
                    let loc = Location::from_raw(raw_any);
                    match loc {
                        Ok(loc) => Some(loc),
                        Err(err) => {
                            error!("Error for: {id} {err:?}");
                            None
                        }
                    }
                });
                info!("{file} decoded to native structs: {:.2?}", start.elapsed());
                codes
            }
            other => panic!("Expected a JSON object: {other:?}"),
        }
    });
    let locs: Vec<Location> = codes_vectors.flatten().collect();
    for loc in locs {
        db.insert(loc);
    }
    info!(
        "DB of {} locations in: {:.2?}",
        db.all.len(),
        start.elapsed()
    );
    loop {
        let inp: String = promptly::prompt("Search Term").expect("Search term expected");
        let start = Instant::now();
        let term = mk_search_term(inp);
        info!("Parse query in {:.3?}", start.elapsed());
        warn!("TERM: {term:#?}");
        let start = Instant::now();
        let res = search(&db, term, 1);
        for (i, (loc_key, score)) in res.iter().enumerate() {
            info!(
                "Result #{i} {loc_key:?} score: {score} {:?}",
                &db.all.get(&loc_key).unwrap().data
            );
        }
        warn!("Search took {:.2?}", start.elapsed());
        println!("\n\n");
    }
}
