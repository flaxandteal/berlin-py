use std::path::PathBuf;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use berlin_core::location::Location;
use berlin_core::locations_db::{parse_data_files, LocationsDb};
use berlin_core::search::SearchTerm;

#[pyclass]
struct LocationsDbProxy {
    _db: LocationsDb,
}

#[pyclass]
struct LocationProxy {
    _loc: Location,
}

#[pymethods]
impl LocationsDbProxy {
    fn query(
        &self,
        query: String,
        state: Option<String>,
        limit: usize,
        lev_distance: u32,
    ) -> PyResult<Vec<LocationProxy>> {
        let gil = Python::acquire_gil();
        let _py = gil.python();
        let st = SearchTerm::from_raw_query(query, state, limit, lev_distance);
        let results = self
            ._db
            .search(&st)
            .into_iter()
            .map(|(key, _score)| {
                let loc = self
                    ._db
                    .all
                    .get(&key)
                    .cloned()
                    .expect("loc should be in db");
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
            "words" => self
                ._loc
                .words
                .iter()
                .map(|word| word.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            _ => {
                let err = PyTypeError::new_err("AttributeError");
                return Err(err);
            }
        };
        Ok(val.to_object(py))
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn load(data_dir: String) -> PyResult<LocationsDbProxy> {
    let data_path = PathBuf::from(data_dir);
    let db = parse_data_files(data_path);
    let db_proxy = LocationsDbProxy { _db: db };
    Ok(db_proxy)
}

/// A Python module implemented in Rust.
#[pymodule]
fn berlin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load, m)?)?;
    Ok(())
}
