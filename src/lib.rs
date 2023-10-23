use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use berlin_core::ustr::Ustr;
use pyo3::exceptions::{PyAttributeError, PyKeyError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::PyList;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};

use berlin_core::location::{subdiv_key, CsvLocode, Location};
use berlin_core::locations_db::{
    parse_data_blocks, parse_data_files, parse_data_list, LocationsDb,
};
use berlin_core::search::SearchTerm;

#[pyclass]
struct LocationsDbProxy {
    _db: Arc<Mutex<LocationsDb>>,
}

#[pyclass(name = "Location")]
struct LocationProxy {
    _loc: Location,
    _db: Arc<Mutex<LocationsDb>>,
}

#[pymethods]
impl LocationsDbProxy {
    fn retrieve(&self, term: String) -> PyResult<LocationProxy> {
        match self._db.lock().unwrap().retrieve(term.as_str()) {
            Some(loc) => Python::with_gil(|_py| {
                Ok(LocationProxy {
                    _loc: loc,
                    _db: self._db.clone(),
                })
            }),
            None => {
                let err = PyKeyError::new_err(format!["{} not found", term.as_str()]);
                Err(err)
            }
        }
    }

    fn get_state_key(&self, state: &str) -> PyResult<String> {
        let code = match Ustr::from_existing(state) {
            None => {
                let err = PyKeyError::new_err(format!["{} not found as state key", state]);
                return Err(err);
            }
            Some(code) => code,
        };
        let result = {
            match self._db.lock().unwrap().state_by_code.get(&code) {
                None => {
                    let err = PyKeyError::new_err(format!["{} not found as state key", state]);
                    Err(err)
                }
                Some(name) => Ok(name.to_string()),
            }
        };
        result
    }

    fn get_subdiv_key(&self, state: &str, subdiv: &str) -> PyResult<String> {
        let code_str = format!["{}:{}", state, subdiv];
        let code = match Ustr::from_existing(code_str.as_str()) {
            None => {
                let err = PyKeyError::new_err(format!["{} not found as subdiv key", code_str]);
                return Err(err);
            }
            Some(code) => code,
        };
        let result = {
            match self._db.lock().unwrap().subdiv_by_code.get(&code) {
                None => {
                    let err = PyKeyError::new_err(format!["{} not found as subdiv key", subdiv]);
                    Err(err)
                }
                Some(name) => Ok(name.to_string()),
            }
        };
        result
    }

    fn query(
        &self,
        query: String,
        limit: usize,
        lev_distance: u32,
        state: Option<String>,
    ) -> PyResult<Vec<LocationProxy>> {
        let results = Python::with_gil(|_py| {
            let st = SearchTerm::from_raw_query(query, state, limit, lev_distance);
            let db = self._db.lock().unwrap();
            db.search(&st)
                .into_iter()
                .map(|(key, _score)| {
                    let loc = db.all.get(&key).cloned().expect("loc should be in db");
                    LocationProxy {
                        _loc: loc,
                        _db: self._db.clone(),
                    }
                })
                .collect()
        });
        Ok(results)
    }
}

#[pymethods]
impl LocationProxy {
    fn __getattr__(&self, attr: String) -> PyResult<PyObject> {
        let val = Python::with_gil(|py| {
            let val = match attr.as_str() {
                "key" => self._loc.key.to_string().to_object(py),
                "encoding" => self._loc.encoding.to_string().to_object(py),
                "id" => self._loc.id.to_string().to_object(py),
                "words" => self
                    ._loc
                    .words
                    .iter()
                    .map(|word| word.to_string())
                    .collect::<Vec<_>>()
                    .to_object(py),
                _ => {
                    let err = PyAttributeError::new_err(format!["{} not found", attr.as_str()]);
                    return Err(err);
                }
            };
            Ok(val)
        });
        Ok(val.unwrap())
    }

    fn get_names(&self) -> PyResult<Py<PyAny>> {
        let val: Result<_, PyAttributeError> = Python::with_gil(|py| {
            let names: &PyList =
                PyList::new(py, self._loc.get_names().iter().map(|name| name.as_str()));
            Ok(names.into())
        });
        Ok(val.unwrap())
    }

    fn get_codes(&self) -> PyResult<Py<PyAny>> {
        let val: Result<_, PyAttributeError> = Python::with_gil(|py| {
            let codes: &PyList =
                PyList::new(py, self._loc.get_codes().iter().map(|code| code.as_str()));
            Ok(codes.into())
        });
        Ok(val.unwrap())
    }

    fn get_state_code(&self) -> &str {
        self._loc.get_state().as_str()
    }

    fn get_subdiv_code(&self) -> Option<&str> {
        match self._loc.get_subdiv() {
            None => None,
            Some(ustr) => Some(ustr.as_str()),
        }
    }

    #[getter]
    fn children(&self) -> PyResult<Vec<LocationProxy>> {
        let db = self._db.lock().unwrap();
        let child_nodes = db.indices.get(&self._loc.key).unwrap().children(&db.arena);
        let result = Python::with_gil(|_py| {
            child_nodes
                .map(|node_id| {
                    let node = db.arena.get(node_id).unwrap();
                    let key = node.get();
                    let loc = db.retrieve(key).unwrap();
                    LocationProxy {
                        _loc: loc,
                        _db: self._db.clone(),
                    }
                })
                .collect()
        });
        Ok(result)
    }

    #[getter]
    fn state(&self) -> PyResult<LocationProxy> {
        let db = self._db.lock().unwrap();
        match db.state_by_code.get(&self._loc.get_state()) {
            Some(key) => Python::with_gil(|_py| {
                let loc = db.retrieve(key).unwrap();
                Ok(LocationProxy {
                    _loc: loc,
                    _db: self._db.clone(),
                })
            }),
            None => {
                let err = PyKeyError::new_err(format!["{} not found", self._loc.get_state()]);
                Err(err)
            }
        }
    }

    #[getter]
    fn subdiv(&self) -> PyResult<Option<LocationProxy>> {
        let db = self._db.lock().unwrap();
        match self._loc.get_subdiv() {
            Some(key) => Python::with_gil(|_py| {
                let state = self._loc.get_state();
                match subdiv_key(state, key) {
                    Some(key) => {
                        let loc = db.retrieve(&key).unwrap();
                        Ok(Some(LocationProxy {
                            _loc: loc,
                            _db: self._db.clone(),
                        }))
                    }
                    None => {
                        let err = PyKeyError::new_err(format!["{} not found", key]);
                        Err(err)
                    }
                }
            }),
            None => Ok(None),
        }
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn load_from_json(
    blocks: Vec<Vec<String>>,
    rows: Vec<HashMap<String, String>>,
) -> PyResult<LocationsDbProxy> {
    let db = {
        let mut errors: Vec<String> = vec![];
        let blocks: Vec<(String, Result<serde_json::Value, _>)> = blocks
            .par_iter()
            .enumerate()
            .map(|(m, strings)| {
                strings
                    .par_iter()
                    .enumerate()
                    .map_with(m, |m, (n, string)| {
                        (
                            format!("{m}, {n}"),
                            serde_json::from_str::<serde_json::Value>(string),
                        )
                    })
            })
            .flatten()
            .collect::<_>();

        let blocks: Vec<(String, serde_json::Value)> = blocks
            .into_iter()
            .filter_map(|(loc, value)| match value {
                Ok(value) => Some((loc, value)),
                Err(err) => {
                    errors.push(format!("Block {loc}: {}", err.to_string()));
                    None
                }
            })
            .collect::<_>();

        if errors.len() > 0 {
            return Err(PyTypeError::new_err(format!(
                "JSON parsing errors:\n{}",
                errors.join("\n")
            )));
        }

        let db = match parse_data_blocks(blocks.into_par_iter(), None) {
            Ok(db) => db,
            Err(err) => {
                return Err(PyTypeError::new_err(format!(
                    "JSON parsing errors:\n{}",
                    err.to_string()
                )));
            }
        };

        let mut errors: Vec<String> = vec![];
        let rows = rows
            .iter()
            .enumerate()
            .filter_map(|(n, row)| {
                match (|row: &HashMap<String, String>| {
                    let locode = CsvLocode {
                        country: match row.get("country") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => return Err(format!("Line {n}: No country")),
                        },
                        subcode: match row.get("subcode") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => return Err(format!("Line {n}: No subcode")),
                        },
                        name: match row.get("name") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => "".to_string(),
                        },
                        name_wo_diacritics: match row.get("name_wo_diacritics") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => "".to_string(),
                        },
                        subdivision_code: match row.get("subdivision_code") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => "".to_string(),
                        },
                        status: match row.get("status") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => "".to_string(),
                        },
                        date: match row.get("date") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => "".to_string(),
                        },
                        iata_code: match row.get("iata_code") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => "".to_string(),
                        },
                        function: match row.get("function") {
                            Some(str_ref) => (*str_ref).clone(),
                            None => "".to_string(),
                        },
                        coordinates: match row.get("coordinates") {
                            Some(coordinates) => Some((*coordinates).clone()),
                            None => None,
                        },
                    };
                    Ok(locode)
                })(row)
                {
                    Ok(locode) => Some(locode),
                    Err(err) => {
                        errors.push(err);
                        None
                    }
                }
            })
            .collect::<Vec<CsvLocode>>();
        if errors.len() > 0 {
            return Err(PyTypeError::new_err(format!(
                "LOCODE parsing errors:\n{}",
                errors.join("\n")
            )));
        }
        let db = match parse_data_list(db, rows.into_iter()) {
            Ok(db) => db,
            Err(err) => {
                return Err(PyTypeError::new_err(format!(
                    "JSON parsing errors:\n{}",
                    err.to_string()
                )));
            }
        };
        db.mk_fst()
    };
    let db_proxy = LocationsDbProxy {
        _db: Arc::new(Mutex::new(db)),
    };
    Ok(db_proxy)
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn load(data_dir: String) -> PyResult<LocationsDbProxy> {
    let data_path = PathBuf::from(data_dir);
    let db = match parse_data_files(data_path) {
        Ok(db) => db,
        Err(err) => {
            return Err(PyTypeError::new_err(format!(
                "JSON parsing errors:\n{}",
                err.to_string()
            )));
        }
    };
    let db_proxy = LocationsDbProxy {
        _db: Arc::new(Mutex::new(db)),
    };
    Ok(db_proxy)
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "_berlin")]
fn berlin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<LocationProxy>()?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_json, m)?)?;
    Ok(())
}
