use crate::{
    citext::{CiStr, CiString},
    error::PointercrateError,
    model::{By, Model},
    operation::Get,
    schema::nationalities,
    Result,
};
use derive_more::Constructor;
use diesel::{pg::PgConnection, result::Error, RunQueryDsl};
use serde_derive::Serialize;

#[derive(Queryable, Debug, PartialEq, Eq, Serialize, Hash, Constructor)]
pub struct Nationality {
    pub country_code: String,
    pub nation: CiString,
}

/// The difference between 'A', as unicode codepoint (65), and '🇦', as unicode codepoint (127462)
const MAGIC: u32 = 127397;

impl Nationality {
    pub fn to_emoji(&self) -> String {
        self.country_code
            .chars()
            .map(|c| std::char::from_u32((c as u32) + MAGIC).unwrap())
            .collect()
    }
}

impl By<nationalities::nation, &CiStr> for Nationality {}
impl By<nationalities::iso_country_code, &String> for Nationality {}

impl Model for Nationality {
    type From = nationalities::table;
    type Selection = (nationalities::iso_country_code, nationalities::nation);

    fn from() -> Self::From {
        nationalities::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Get<&str> for Nationality {
    fn get(id: &str, connection: &PgConnection) -> Result<Self> {
        match Nationality::by(&id.to_uppercase())
            .first(connection)
            .or_else(|_| Nationality::by(CiStr::from_str(id)).first(connection))
        {
            Ok(nationality) => Ok(nationality),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Nationality",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}
