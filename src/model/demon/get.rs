use super::{Demon, DemonWithCreatorsAndRecords};
use crate::{
    citext::CiStr,
    context::RequestContext,
    error::PointercrateError,
    model::{creator::Creators, record::EmbeddedRecordP, By},
    operation::Get,
    Result,
};
use diesel::{result::Error, PgConnection, RunQueryDsl};

impl<'a> Get<&'a CiStr> for Demon {
    fn get(name: &'a CiStr, _ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        match Demon::by(name).first(connection) {
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
    fn get(position: i16, _ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        match Demon::by(position).first(connection) {
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
    fn get(demon: Demon, ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        let creators = Creators::get(demon.name.as_ref(), ctx, connection)?;
        let records = Vec::<EmbeddedRecordP>::get(&demon, ctx, connection)?;

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
    fn get(t: T, ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        DemonWithCreatorsAndRecords::get(Demon::get(t, ctx, connection)?, ctx, connection)
    }
}
