use super::{Demon, DemonWithCreatorsAndRecords};
use crate::{
    error::PointercrateError,
    model::{creator::Creators, demon::PartialDemon, record::EmbeddedRecordP},
    operation::Get,
    permissions::AccessRestrictions,
    Result,
};
use diesel::{result::Error, PgConnection, RunQueryDsl};
use crate::citext::CiStr;

impl<'a> Get<&'a CiStr> for Demon {
    fn get(name: &'a CiStr, connection: &PgConnection) -> Result<Self> {
        match Demon::by_name(name).first(connection) {
            Ok(demon) => Ok(demon),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Demon",
                    identified_by: name.to_string(),
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

impl Get<Demon> for DemonWithCreatorsAndRecords {
    fn get(demon: Demon, connection: &PgConnection) -> Result<Self> {
        let creators = Creators::get(demon.name.as_ref(), connection)?;
        let records = Vec::<EmbeddedRecordP>::get(&demon, connection)?;

        Ok(DemonWithCreatorsAndRecords {
            demon,
            creators,
            records,
        })
    }
}

impl<T> Get<T> for DemonWithCreatorsAndRecords
where
    Demon: Get<T>,
{
    fn get(t: T, connection: &PgConnection) -> Result<Self> {
        DemonWithCreatorsAndRecords::get(Demon::get(t, connection)?, connection)
    }
}

impl AccessRestrictions for Demon {}
impl AccessRestrictions for PartialDemon {}
impl AccessRestrictions for DemonWithCreatorsAndRecords {}
