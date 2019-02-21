use super::{EmbeddedRecord, Record};
use crate::{error::PointercrateError, model::Model, operation::Get, schema::records, Result};
use diesel::{result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

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

impl Get<i32> for Vec<EmbeddedRecord> {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        Ok(EmbeddedRecord::all()
            .filter(records::id.eq(&id))
            .load(connection)?)
    }
}
