use super::{Demon, FullDemon};
use crate::{
    citext::CiStr,
    context::RequestContext,
    error::PointercrateError,
    model::demonlist::{creator::Creators, record::MinimalRecordP},
    operation::Get,
    Result,
};
use diesel::{result::Error, RunQueryDsl};

impl<'a> Get<&'a CiStr> for Demon {
    fn get(name: &'a CiStr, ctx: RequestContext) -> Result<Self> {
        match Demon::by_name(name).first(ctx.connection()) {
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

// Now obviously, depending on the differently sized integer types here is very fragile.
// Once /api/v1/ is deprecated and removed, it wont be necessary anymore (or we could use strong
// types)

impl Get<i16> for Demon {
    fn get(position: i16, ctx: RequestContext) -> Result<Self> {
        match Demon::by_position(position).first(ctx.connection()) {
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

impl Get<i32> for Demon {
    fn get(id: i32, ctx: RequestContext) -> Result<Self> {
        // FIXME: figure out why Demon::find doesn't work
        match Demon::by_id(id).first(ctx.connection()) {
            Ok(demon) => Ok(demon),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Demon",
                    identified_by: id.to_string(),
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
