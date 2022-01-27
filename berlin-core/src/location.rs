use std::cmp::max;
use std::collections::HashMap;

use serde::de::Error;
use serde::{Deserialize, Serialize};
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

const STATE_ENCODING: &str = "ISO-3166-1";

pub fn state_key(state_code: Ustr) -> Option<Ustr> {
    let str = format!("{}#{}", STATE_ENCODING, state_code.as_str());
    Ustr::from_existing(str.as_str())
}

const SUBDIV_ENCODING: &str = "ISO-3166-2";

pub fn subdiv_key(state_code: Ustr, subdiv_code: Ustr) -> Option<Ustr> {
    let str = format!(
        "{}#{}:{}",
        SUBDIV_ENCODING,
        state_code.as_str(),
        subdiv_code.as_str()
    );
    Ustr::from_existing(str.as_str())
}

const LOCODE_ENCODING: &str = "UN-LOCODE";
const IATA_ENCODING: &str = "IATA";

#[derive(Debug, Serialize, Clone, Copy)]
pub struct Location {
    pub key: Ustr,
    pub encoding: Ustr,
    pub id: Ustr,
    pub data: LocData,
}

impl Location {
    pub fn from_raw(r: AnyLocation) -> serde_json::Result<Self> {
        let encoding: Ustr = r.c.as_str().into();
        let data = match encoding.as_str() {
            STATE_ENCODING => LocData::St(State::from_raw(r.d)?),
            SUBDIV_ENCODING => LocData::Subdv(Subdivision::from_raw(r.d)?),
            LOCODE_ENCODING => LocData::Locd(Locode::from_raw(r.d)?),
            IATA_ENCODING => LocData::Airp(Airport::from_raw(r.d)?),
            other => {
                panic!("Unexpected location standard {}", other)
            }
        };
        let id: Ustr = r.i.into();
        let key = format!("{}#{}", encoding.as_str(), normalize(id.as_str()));
        Ok(Self {
            key: Ustr::from(&key),
            id,
            encoding,
            data,
        })
    }
    pub fn search(&self, term: &SearchTerm) -> i64 {
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
    pub fn get_parents(&self) -> (Option<Ustr>, Option<Ustr>) {
        match self.data {
            LocData::St(_) => (None, None),
            LocData::Subdv(sd) => (state_key(sd.supercode), None),
            LocData::Locd(l) => (
                state_key(l.supercode),
                l.subdivision_code
                    .map(|c| subdiv_key(l.supercode, c))
                    .flatten(),
            ),
            LocData::Airp(a) => (state_key(a.country), None),
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
pub enum LocData {
    St(State),
    Subdv(Subdivision),
    Locd(Locode),
    Airp(Airport),
}

impl LocData {
    pub fn get_state(&self) -> Ustr {
        match self {
            LocData::St(s) => s.alpha2,
            LocData::Subdv(sd) => sd.supercode,
            LocData::Locd(l) => l.supercode,
            LocData::Airp(a) => a.country,
        }
    }
    pub fn get_subdiv(&self) -> Option<Ustr> {
        match self {
            LocData::St(_) => None,
            LocData::Subdv(sd) => Some(sd.subcode),
            LocData::Locd(l) => l.subdivision_code,
            LocData::Airp(_) => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct State {
    name: Ustr,
    short: Ustr,
    pub(crate) alpha2: Ustr,
    alpha3: Ustr,
    continent: Ustr,
}

impl State {
    fn get_names(&self) -> Vec<Ustr> {
        vec![self.name, self.short, self.alpha2, self.alpha3]
    }
    fn get_codes(&self) -> Vec<Ustr> {
        vec![self.alpha2, self.alpha3]
    }
    fn code_match(&self, code: Ustr) -> bool {
        [self.short, self.alpha2, self.alpha3]
            .iter()
            .any(|f| f == &code)
    }
    fn search(&self, t: &SearchTerm) -> i64 {
        self.get_names()
            .iter()
            .map(|n| t.match_str(n))
            .max()
            .unwrap_or(0)
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

#[derive(Hash, Eq, PartialEq)]
pub struct SubDivKey {
    pub(crate) state: Ustr,
    pub(crate) subcode: Ustr,
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct Subdivision {
    name: Ustr,
    pub(crate) supercode: Ustr,
    pub(crate) subcode: Ustr,
    level: Ustr,
}

impl Subdivision {
    pub fn subdiv_key(&self) -> SubDivKey {
        SubDivKey {
            state: self.supercode,
            subcode: self.subcode,
        }
    }
    fn get_names(&self) -> Vec<Ustr> {
        vec![self.name, self.subcode]
    }
    fn get_codes(&self) -> Vec<Ustr> {
        vec![self.subcode]
    }
    fn code_match(&self, code: Ustr) -> bool {
        [self.supercode, self.subcode].iter().any(|f| f == &code)
    }
    fn search(&self, t: &SearchTerm) -> i64 {
        max(t.match_str(&self.name), t.match_str(&self.subcode))
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

#[derive(Debug, Copy, Clone, Serialize)]
pub struct Locode {
    name: Ustr,
    pub(crate) supercode: Ustr,
    pub(crate) subcode: Ustr,
    subdivision_name: Option<Ustr>,
    pub(crate) subdivision_code: Option<Ustr>,
    pub(crate) function_code: Ustr,
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
    fn search(&self, t: &SearchTerm) -> i64 {
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

#[derive(Deserialize, Debug, Clone)]
pub struct AirportRaw {
    name: String,
    iata: String,
    #[serde(rename = "type")]
    airport_type: String,
    city: Option<String>,
    country: String,
    region: String,
    y: f64,
    x: f64,
    elevation: Option<String>,
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct Airport {
    name: Ustr,
    iata: Ustr,
    airport_type: Ustr,
    city: Option<Ustr>,
    pub(crate) country: Ustr,
    region: Ustr,
    x: f64,
    y: f64,
    elevation: Option<i16>,
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
    fn search(&self, t: &SearchTerm) -> i64 {
        t.match_str(&self.name)
    }
    fn from_raw(r: serde_json::Value) -> serde_json::Result<Self> {
        let raw = serde_json::from_value::<AirportRaw>(r)?;
        let airport_type = Ustr::from(&raw.airport_type);
        let elevation = raw
            .elevation
            .as_ref()
            .map(|e| e.parse::<i16>().expect("parse elevation"));
        Ok(Self {
            name: normalize(&raw.name).into(),
            iata: normalize(&raw.iata).into(),
            city: raw.city.map(|c| normalize(&c).into()),
            airport_type,
            country: normalize(&raw.country).into(),
            region: normalize(&raw.region).into(),
            x: raw.x,
            y: raw.y,
            elevation,
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
