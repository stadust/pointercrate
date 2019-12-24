use crate::{
    citext::{CiStr, CiString},
    context::RequestContext,
    error::PointercrateError,
    model::Model,
    operation::Get,
    schema::nationalities,
    Result,
};
use derive_more::Constructor;
use diesel::{result::Error, RunQueryDsl, Table};
use serde_derive::Serialize;

#[derive(Queryable, Debug, PartialEq, Eq, Serialize, Hash, Constructor, Identifiable)]
#[table_name = "nationalities"]
#[primary_key(iso_country_code)]
pub struct Nationality {
    pub iso_country_code: String,
    pub nation: CiString,
}

impl Model for Nationality {
    type Selection = <nationalities::table as Table>::AllColumns;

    fn selection() -> Self::Selection {
        nationalities::all_columns
    }
}

impl Nationality {
    by!(by_nation_name, nationalities::nation, &CiStr);
}

impl Get<&str> for Nationality {
    fn get(id: &str, ctx: RequestContext) -> Result<Self> {
        let connection = ctx.connection();

        match Nationality::find(&id.to_uppercase())
            .first(connection)
            .or_else(|_| Nationality::by_nation_name(CiStr::from_str(id)).first(connection))
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
