
use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use std::fs::File;
use std::io::BufReader;
use berlin_core::rayon::iter::IntoParallelIterator;
use berlin_core::rayon::prelude::*;
use berlin_core::locations_db::LocationsDb;
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};
use serde_json::Value;
use std::time::Instant;
use std::path::PathBuf;
use berlin_core::location::{AnyLocation, Location};
use berlin_core::search::SearchTerm;

#[pyclass]
struct LocationsDbProxy {
    _db: LocationsDb
}

#[pyclass]
struct LocationProxy {
    _loc: Location
}

#[pymethods]
impl LocationsDbProxy {
    fn query(&self, query: String, state: Option<String>, limit: usize) -> PyResult<Vec<LocationProxy>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let st = SearchTerm::from_raw_query(query, state);
        let results = self._db
            .search(&st, limit)
            .into_iter()
            .map(|(key, score)| {
                let loc = self._db.all.get(&key).cloned().expect("loc should be in db");
                LocationProxy { _loc: loc }
            })
            .collect();
        Ok(results)
    }
}

#[pymethods]
impl LocationProxy {
    fn __getattr__(&self, attr: String) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let val = match attr.as_str() {
            "key" => self._loc.key.to_string(),
            "encoding" => self._loc.encoding.to_string(),
            "id" => self._loc.id.to_string(),
            "words" => self._loc.words.iter().map(|word| {
                word.to_string()
            }).collect(),
            _ => {
                let err = PyTypeError::new_err("AttributeError");
                return Err(err)
            }
        };
        Ok(val.to_object(py))
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn load(data_dir: String) -> PyResult<LocationsDbProxy> {
    let data_path = PathBuf::from(data_dir);
    let db = parse_json_files(data_path);
    let db_proxy = LocationsDbProxy { _db: db };
    Ok(db_proxy)
}

/// A Python module implemented in Rust.
#[pymodule]
fn berlin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load, m)?)?;
    Ok(())
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
                                error!("Error for: {id} {:?}", err);
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
