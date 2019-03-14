use super::{EmbeddedPlayer, PlayerWithDemonsAndRecords};
use crate::{
    citext::CiStr,
    error::PointercrateError,
    model::{
        creator::created_by, demon::EmbeddedDemon, player::ShortPlayer, user::User, By, Model,
    },
    operation::Get,
    permissions::{self, AccessRestrictions},
    schema::demons,
    Result,
};
use diesel::{result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use crate::model::player::RankedPlayer2;

impl<'a> Get<&'a CiStr> for EmbeddedPlayer {
    fn get(name: &'a CiStr, connection: &PgConnection) -> Result<Self> {
        match EmbeddedPlayer::by(name).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                EmbeddedPlayer::insert(&name, connection).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for EmbeddedPlayer {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
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
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
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
    fn get(t: T, connection: &PgConnection) -> Result<Self> {
        let player = ShortPlayer::get(t, connection)?;
        let pid = player.inner.id;

        Ok(PlayerWithDemonsAndRecords {
            records: Get::get(pid, connection)?,
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

// Everyone can access player objects (through the stats viewer)
impl AccessRestrictions for ShortPlayer {
    fn pre_page_access(user: Option<&User>) -> Result<()> {
        permissions::demand(
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
            user,
        )
    }
}
impl AccessRestrictions for PlayerWithDemonsAndRecords {}
impl AccessRestrictions for EmbeddedPlayer {}
impl AccessRestrictions for RankedPlayer2 {}
