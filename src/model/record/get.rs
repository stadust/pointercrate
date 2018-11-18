use super::Record;
use crate::{error::PointercrateError, operation::Get, Result};
use diesel::{result::Error, PgConnection, RunQueryDsl};

impl Get<i32> for Record {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        match Record::by_id(id).first(connection) {
            Ok(record) => Ok(record),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Record",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}
