use std::collections::HashMap;

use deunicode::deunicode;
use regex::Regex;
use serde::de::Error;
use serde::Deserialize;
use strum_macros;
use ustr::Ustr;

use crate::{normalize, search_in_string};

#[derive(Debug, Deserialize)]
pub struct AnyLocationCode {
    #[serde(rename = "<c>")]
    pub c: String,
    i: String,
    d: serde_json::Value,
}

#[derive(Debug)]
pub struct Location {
    pub key: Ustr,
    // Unified encoding+id Ustr for convenient usage as a key in hashmaps etc.
    encoding: Ustr,
    id: Ustr,
    pub data: LocData,
}

impl Location {
    pub fn from_raw(r: AnyLocationCode) -> serde_json::Result<Self> {
        let encoding: Ustr = r.c.as_str().into();
        let data = match r.c.as_str() {
            "ISO-3166-1" => LocData::St(State::from_raw(r.d)?),
            "ISO-3166-2" => LocData::Subdv(Subdivision::from_raw(r.d)?),
            "UN-LOCODE" => LocData::Locd(Locode::from_raw(r.d)?),
            "IATA" => LocData::Airp(Airport::from_raw(r.d)?),
            standard => {
                panic!("Unexpected location standard {}", standard)
            }
        };
        let id: Ustr = r.i.into();
        let key = format!("{}#{}", encoding.as_str(), id.as_str());
        Ok(Self {
            key: Ustr::from(&key),
            id,
            encoding,
            data,
        })
    }
    pub fn search(&self, term: &str, re: &Regex) -> u64 {
        match &self.data {
            LocData::St(d) => d.search(term, re),
            LocData::Subdv(d) => d.search(term, re),
            LocData::Locd(d) => d.search(term, re),
            LocData::Airp(d) => d.search(term, re),
        }
    }
}

#[derive(Debug, strum_macros::Display)]
pub enum LocData {
    St(State),
    Subdv(Subdivision),
    Locd(Locode),
    Airp(Airport),
}

#[derive(Debug)]
pub struct State {
    name: Ustr,
    short: Ustr,
    alpha2: Ustr,
    alpha3: Ustr,
    continent: Ustr,
}

impl State {
    fn search(&self, t: &str, re: &Regex) -> u64 {
        search_in_string(&self.name, t, re) + 3
    }
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let r = serde_json::from_value::<HashMap<String, String>>(r)?;
        Ok(Self {
            name: normalize(extract_field(&r, "name")?).into(),
            short: normalize(extract_field(&r, "short")?).into(),
            alpha2: normalize(extract_field(&r, "alpha2")?).into(),
            alpha3: normalize(extract_field(&r, "alpha3")?).into(),
            continent: normalize(extract_field(&r, "continent")?).into(),
        })
    }
}

#[derive(Debug)]
pub struct Subdivision {
    name: Ustr,
    supercode: Ustr,
    subcode: Ustr,
    level: Ustr,
}

impl Subdivision {
    fn search(&self, t: &str, re: &Regex) -> u64 {
        search_in_string(&self.name, t, re) + 2
    }
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let r = serde_json::from_value::<HashMap<String, String>>(r)?;
        Ok(Self {
            name: normalize(extract_field(&r, "name")?).into(),
            supercode: normalize(extract_field(&r, "supercode")?).into(),
            subcode: normalize(extract_field(&r, "subcode")?).into(),
            level: normalize(extract_field(&r, "level")?).into(),
        })
    }
}

#[derive(Debug)]
pub struct Locode {
    name: Ustr,
    supercode: Ustr,
    subcode: Ustr,
    subdivision_name: Option<Ustr>,
    subdivision_code: Option<Ustr>,
    function_code: Ustr,
}

impl Locode {
    fn search(&self, t: &str, re: &Regex) -> u64 {
        search_in_string(&self.name, t, re)
    }
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let r = serde_json::from_value::<HashMap<String, String>>(r)?;
        Ok(Self {
            name: crate::normalize(extract_field(&r, "name")?).into(),
            supercode: normalize(extract_field(&r, "supercode")?).into(),
            subcode: normalize(extract_field(&r, "subcode")?).into(),
            subdivision_name: r
                .get("subdivision_name")
                .map(|sd| crate::normalize(sd).into()),
            subdivision_code: r.get("subdivision_code").map(|sd| normalize(sd).into()),
            function_code: normalize(extract_field(&r, "function_code")?).into(),
        })
    }
}

#[derive(Deserialize)]
pub struct AirportRaw {
    name: String,
    #[serde(rename = "type")]
    airport_type: String,
    city: Option<String>,
    country: String,
    region: String,
    y: f64,
    x: f64,
    elevation: Option<String>,
}

#[derive(Debug)]
pub struct Airport {
    name: Ustr,
    airport_type: Ustr,
    city: Option<Ustr>,
    country: Ustr,
    region: Ustr,
    x: f64,
    y: f64,
    elevation: Option<String>,
}

impl Airport {
    fn search(&self, t: &str, re: &Regex) -> u64 {
        search_in_string(&self.name, t, &re) + 1
    }
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let raw = serde_json::from_value::<AirportRaw>(r)?;
        Ok(Self {
            name: normalize(&raw.name).into(),
            city: raw.city.map(|c| normalize(&c).into()),
            airport_type: raw.airport_type.into(),
            country: normalize(&raw.country).into(),
            region: normalize(&raw.region).into(),
            x: raw.x,
            y: raw.y,
            elevation: raw.elevation,
        })
    }
}

fn extract_field<'a>(hm: &'a HashMap<String, String>, field: &str) -> serde_json::Result<&'a str> {
    let val = hm.get(field);
    match val {
        Some(fl) => Ok(fl),
        None => {
            let err = format!("Missing field {}", field);
            Err(serde_json::error::Error::custom(err))
        }
    }
}
