use crate::cistring::CiString;
use derive_more::Constructor;
use serde::Serialize;

mod get;

#[derive(Debug, PartialEq, Eq, Serialize, Hash, Constructor)]
pub struct Nationality {
    #[serde(rename = "country_code")]
    pub iso_country_code: String,
    pub nation: CiString,
}
