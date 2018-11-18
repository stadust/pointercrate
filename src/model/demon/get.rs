use super::Demon;
use crate::{error::PointercrateError, operation::Get, Result};
use diesel::{result::Error, PgConnection, RunQueryDsl};

impl Get<String> for Demon {
    fn get(name: String, connection: &PgConnection) -> Result<Self> {
        match Demon::by_name(&name).first(connection) {
            Ok(demon) => Ok(demon),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Demon",
                    identified_by: name,
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}
