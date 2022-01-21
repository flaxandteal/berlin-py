use std::cmp::max;
use std::collections::HashMap;

use serde::de::Error;
use serde::Deserialize;
use ustr::Ustr;

use crate::normalize;
use crate::search::SearchTerm;

#[derive(Debug, Deserialize)]
pub struct AnyLocation {
    #[serde(rename = "<c>")]
    pub c: String,
    i: String,
    d: serde_json::Value,
}

#[derive(Debug)]
pub struct Location {
    pub key: Ustr, // encoding+id for usage as a key in hashmaps etc.
    pub encoding: Ustr,
    pub id: Ustr,
    pub data: LocData,
}

impl Location {
    pub fn from_raw(r: AnyLocation) -> serde_json::Result<Self> {
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
    pub fn search(&self, term: &SearchTerm) -> u64 {
        match &self.data {
            LocData::St(d) => d.search(term),
            LocData::Subdv(d) => d.search(term),
            LocData::Locd(d) => d.search(term),
            LocData::Airp(d) => d.search(term),
        }
    }
    pub fn get_names(&self) -> Vec<Ustr> {
        match &self.data {
            LocData::St(st) => st.get_names(),
            LocData::Subdv(sd) => sd.get_names(),
            LocData::Locd(locd) => locd.get_names(),
            LocData::Airp(ap) => ap.get_names(),
        }
    }
    pub fn get_codes(&self) -> Vec<Ustr> {
        match &self.data {
            LocData::St(st) => st.get_codes(),
            LocData::Subdv(sd) => sd.get_codes(),
            LocData::Locd(lc) => lc.get_codes(),
            LocData::Airp(ap) => ap.get_codes(),
        }
    }
    pub fn code_match(&self, code: Ustr) -> bool {
        match &self.data {
            LocData::St(st) => st.code_match(code),
            LocData::Subdv(sd) => sd.code_match(code),
            LocData::Locd(locd) => locd.code_match(code),
            LocData::Airp(ap) => ap.code_match(code),
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
    fn get_names(&self) -> Vec<Ustr> {
        vec![self.name, self.short, self.alpha2, self.alpha3]
    }
    fn get_codes(&self) -> Vec<Ustr> {
        vec![self.short, self.alpha2, self.alpha3]
    }
    fn code_match(&self, code: Ustr) -> bool {
        [self.short, self.alpha2, self.alpha3]
            .iter()
            .any(|f| f == &code)
    }
    fn search(&self, t: &SearchTerm) -> u64 {
        max(t.match_str(&self.name), t.match_str(&self.short))
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
    fn get_names(&self) -> Vec<Ustr> {
        vec![self.name, self.subcode]
    }
    fn get_codes(&self) -> Vec<Ustr> {
        vec![self.subcode]
    }
    fn code_match(&self, code: Ustr) -> bool {
        [self.supercode, self.subcode].iter().any(|f| f == &code)
    }
    fn search(&self, t: &SearchTerm) -> u64 {
        t.match_str(&self.name)
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
    fn get_names(&self) -> Vec<Ustr> {
        vec![self.name]
    }
    fn get_codes(&self) -> Vec<Ustr> {
        vec![]
    }
    fn code_match(&self, code: Ustr) -> bool {
        let mut codes = vec![];
        if let Some(sd) = self.subdivision_code {
            codes.push(sd);
        }
        codes.push(self.subcode);
        codes.push(self.supercode);
        codes.iter().any(|f| f == &code)
    }
    fn search(&self, t: &SearchTerm) -> u64 {
        t.match_str(&self.name)
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
    fn get_names(&self) -> Vec<Ustr> {
        vec![self.name]
    }
    fn get_codes(&self) -> Vec<Ustr> {
        vec![]
    }
    fn code_match(&self, code: Ustr) -> bool {
        self.country == code
    }
    fn search(&self, t: &SearchTerm) -> u64 {
        max(
            t.match_str(&self.name),
            self.city.map(|c| t.match_str(&c)).unwrap_or(0),
        )
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
