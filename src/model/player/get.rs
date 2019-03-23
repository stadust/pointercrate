use super::{EmbeddedPlayer, PlayerWithDemonsAndRecords};
use crate::{
    citext::CiStr,
    context::RequestContext,
    error::PointercrateError,
    model::{creator::created_by, demon::EmbeddedDemon, player::ShortPlayer, By, Model},
    operation::Get,
    schema::demons,
    Result,
};
use diesel::{result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

impl<'a> Get<&'a CiStr> for EmbeddedPlayer {
    fn get(name: &'a CiStr, _ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        let name = CiStr::from_str(name.trim());

        match EmbeddedPlayer::by(name).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                EmbeddedPlayer::insert(&name, connection).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for EmbeddedPlayer {
    fn get(id: i32, _ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        match EmbeddedPlayer::by(id).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Player",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for ShortPlayer {
    fn get(id: i32, _ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        match ShortPlayer::by(id).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Player",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl<T> Get<T> for PlayerWithDemonsAndRecords
where
    ShortPlayer: Get<T>,
{
    fn get(t: T, ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        let player = ShortPlayer::get(t, ctx, connection)?;
        let pid = player.inner.id;

        Ok(PlayerWithDemonsAndRecords {
            records: Get::get(pid, ctx, connection)?,
            created: created_by(pid).load(connection)?,
            verified: EmbeddedDemon::all()
                .filter(demons::verifier.eq(&pid))
                .load(connection)?,
            published: EmbeddedDemon::all()
                .filter(demons::publisher.eq(&pid))
                .load(connection)?,
            player,
        })
    }
}
