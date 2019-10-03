use super::{DatabasePlayer, FullPlayer};
use crate::{
    citext::CiStr,
    context::RequestContext,
    error::PointercrateError,
    model::{
        demonlist::{creator::created_by, demon::MinimalDemon, player::Player},
        By, Model,
    },
    operation::Get,
    schema::{demons, players},
    Result,
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl, RunQueryDsl};
use log::info;

#[derive(Insertable, Debug)]
#[table_name = "players"]
struct NewPlayer<'a> {
    name: &'a CiStr,
}

impl<'a> Get<&'a CiStr> for DatabasePlayer {
    fn get(name: &'a CiStr, ctx: RequestContext) -> Result<Self> {
        let name = CiStr::from_str(name.trim());

        match DatabasePlayer::by(name).first(ctx.connection()) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) => {
                info!("Creating new player with name {}", name);

                insert_into(players::table)
                    .values(&NewPlayer { name })
                    .returning(DatabasePlayer::selection())
                    .get_result(ctx.connection())
                    .map_err(PointercrateError::database)
            },
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for DatabasePlayer {
    fn get(id: i32, ctx: RequestContext) -> Result<Self> {
        match DatabasePlayer::by(id).first(ctx.connection()) {
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

impl Get<i32> for Player {
    fn get(id: i32, ctx: RequestContext) -> Result<Self> {
        match Player::by(id).first(ctx.connection()) {
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

impl<T> Get<T> for FullPlayer
where
    Player: Get<T>,
{
    fn get(t: T, ctx: RequestContext) -> Result<Self> {
        let player = Player::get(t, ctx)?;
        let pid = player.inner.id;

        Ok(FullPlayer {
            records: Get::get(pid, ctx)?,
            created: created_by(pid).load(ctx.connection())?,
            verified: MinimalDemon::all()
                .filter(demons::verifier.eq(&pid))
                .load(ctx.connection())?,
            published: MinimalDemon::all()
                .filter(demons::publisher.eq(&pid))
                .load(ctx.connection())?,
            player,
        })
    }
}
