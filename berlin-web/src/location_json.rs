use schemars::JsonSchema;
use ustr::Ustr;

use berlin_core::location::Location;
use berlin_core::locations_db::LocationsDb;
use berlin_core::smallvec::SmallVec;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct LocJson {
    encoding: &'static str,
    id: &'static str,
    key: &'static str,
    names: SmallVec<[&'static str; 1]>,
    codes: SmallVec<[&'static str; 1]>,
    state: (&'static str, &'static str),
    subdiv: Option<(&'static str, &'static str)>,
}

impl LocJson {
    pub fn from_location(db: &LocationsDb, l: &Location) -> Self {
        let state_code = l.get_state();
        let state_name: Ustr = *db
            .state_by_code
            .get(&state_code)
            .unwrap_or(&"#UNKNOWN COUNTRY#".into());
        let subdiv = l
            .get_subdiv()
            .map(|sd| -> Option<(&'static str, &'static str)> {
                let code_str = format!("{}:{}", state_code.as_str(), sd.as_str());
                let code = Ustr::from_existing(code_str.as_str())?;
                let name = db.subdiv_by_code.get(&code)?;
                Some((sd.as_str(), name.as_str()))
            })
            .flatten();
        Self {
            key: l.key.into(),
            encoding: l.encoding.as_str(),
            id: l.id.as_str(),
            names: l.get_names().into_iter().map(|u| u.as_str()).collect(),
            codes: l.get_codes().into_iter().map(|u| u.as_str()).collect(),
            state: (state_code.as_str(), state_name.as_str()),
            subdiv,
        }
    }
}
