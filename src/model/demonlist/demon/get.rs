use super::{Demon, FullDemon};
use crate::{
    citext::CiStr,
    context::RequestContext,
    error::PointercrateError,
    model::{
        demonlist::{creator::Creators, record::MinimalRecordP},
        By,
    },
    operation::Get,
    Result,
};
use diesel::{result::Error, RunQueryDsl};

impl<'a> Get<&'a CiStr> for Demon {
    fn get(name: &'a CiStr, ctx: RequestContext) -> Result<Self> {
        match Demon::by(name).first(ctx.connection()) {
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
    fn get(position: i16, ctx: RequestContext) -> Result<Self> {
        match Demon::by(position).first(ctx.connection()) {
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

impl Get<Demon> for FullDemon {
    fn get(demon: Demon, ctx: RequestContext) -> Result<Self> {
        let creators = Creators::get(demon.id, ctx)?;
        let records = Vec::<MinimalRecordP>::get(&demon, ctx)?;

        Ok(FullDemon {
            demon,
            creators,
            records,
        })
    }
}

impl<T> Get<T> for FullDemon
where
    Demon: Get<T>,
{
    fn get(t: T, ctx: RequestContext) -> Result<Self> {
        FullDemon::get(Demon::get(t, ctx)?, ctx)
    }
}
