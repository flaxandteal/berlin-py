use std::collections::HashMap;

use serde::de::Error;
use serde::Deserialize;
use ustr::Ustr;

#[derive(Debug, Deserialize)]
pub struct AnyLocationCode {
    #[serde(rename = "<c>")]
    pub c: String,
    i: String,
    d: serde_json::Value,
}

impl AnyLocationCode {
    pub fn dispatch(self) -> serde_json::Result<LocationCode> {
        match self.c.as_str() {
            "ISO-3166-1" => Ok(LocationCode::St(State::from_raw(self.d)?)),
            "ISO-3166-2" => Ok(LocationCode::Subdv(Subdivision::from_raw(self.d)?)),
            "UN-LOCODE" => Ok(LocationCode::Locd(Locode::from_raw(self.d)?)),
            "IATA" => Ok(LocationCode::Airp(Airport::from_raw(self.d)?)),
            standard => {
                panic!("Unexpected location standard {}", standard)
            }
        }
    }
}

#[derive(Debug)]
pub enum LocationCode {
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
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let r = serde_json::from_value::<HashMap<String, String>>(r)?;
        Ok(Self {
            name: extract_field(&r, "name")?.into(),
            short: extract_field(&r, "short")?.into(),
            alpha2: extract_field(&r, "alpha2")?.into(),
            alpha3: extract_field(&r, "alpha3")?.into(),
            continent: extract_field(&r, "continent")?.into(),
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
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let r = serde_json::from_value::<HashMap<String, String>>(r)?;
        Ok(Self {
            name: extract_field(&r, "name")?.into(),
            supercode: extract_field(&r, "supercode")?.into(),
            subcode: extract_field(&r, "subcode")?.into(),
            level: extract_field(&r, "level")?.into(),
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
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let r = serde_json::from_value::<HashMap<String, String>>(r)?;
        Ok(Self {
            name: extract_field(&r, "name")?.into(),
            supercode: extract_field(&r, "supercode")?.into(),
            subcode: extract_field(&r, "subcode")?.into(),
            subdivision_name: r.get("subdivision_name").map(|sd| sd.as_str().into()),
            subdivision_code: r.get("subdivision_code").map(|sd| sd.as_str().into()),
            function_code: extract_field(&r, "function_code")?.into(),
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
    name: String,
    airport_type: Ustr,
    city: Option<Ustr>,
    country: Ustr,
    region: Ustr,
    x: f64,
    y: f64,
    elevation: Option<String>,
}

impl Airport {
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let raw = serde_json::from_value::<AirportRaw>(r)?;
        Ok(Self {
            name: raw.name,
            airport_type: raw.airport_type.into(),
            city: raw.city.map(|c| c.into()),
            country: raw.country.into(),
            region: raw.region.into(),
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
