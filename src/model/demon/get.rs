use super::{Demon, DemonWithCreatorsAndRecords};
use crate::{
    error::PointercrateError,
    model::{creator::Creators, record::EmbeddedRecord},
    operation::Get,
    Result,
};
use diesel::{result::Error, PgConnection, RunQueryDsl};

impl<'a> Get<&'a str> for Demon {
    fn get(name: &'a str, connection: &PgConnection) -> Result<Self> {
        match Demon::by_name(name).first(connection) {
            Ok(demon) => Ok(demon),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Demon",
                    identified_by: name.to_owned(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i16> for Demon {
    fn get(position: i16, connection: &PgConnection) -> Result<Self> {
        match Demon::by_position(position).first(connection) {
            Ok(demon) => Ok(demon),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Demon",
                    identified_by: position.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl<T> Get<T> for DemonWithCreatorsAndRecords
where
    Demon: Get<T>,
{
    fn get(t: T, connection: &PgConnection) -> Result<Self> {
        let demon = Demon::get(t, connection)?;
        let creators = Creators::get(&demon.name, connection)?;
        let records = Vec::<EmbeddedRecord>::get(&demon, connection)?;

        Ok(DemonWithCreatorsAndRecords {
            demon,
            creators,
            records,
        })
    }
}
