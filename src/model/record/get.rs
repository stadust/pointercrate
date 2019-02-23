use super::{EmbeddedRecordD, EmbeddedRecordP, EmbeddedRecordPD, Record};
use crate::{
    error::PointercrateError,
    model::{demon::Demon, record::RecordStatus, submitter::Submitter, Model},
    operation::Get,
    schema::records,
    Result,
};
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

impl Get<i32> for Vec<EmbeddedRecordD> {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        Ok(EmbeddedRecordD::by_player_and_status(id, RecordStatus::Approved).load(connection)?)
    }
}

impl<'a> Get<&'a Demon> for Vec<EmbeddedRecordP> {
    fn get(demon: &'a Demon, connection: &PgConnection) -> Result<Self> {
        Ok(
            EmbeddedRecordP::by_demon_and_status(&demon.name, RecordStatus::Approved)
                .order_by(records::progress.desc())
                .load(connection)?,
        )
    }
}

impl<'a> Get<&'a Submitter> for Vec<EmbeddedRecordPD> {
    fn get(submitter: &'a Submitter, connection: &PgConnection) -> Result<Self> {
        Ok(EmbeddedRecordPD::all()
            .filter(records::submitter.eq(&submitter.id))
            .load(connection)?)
    }
}
