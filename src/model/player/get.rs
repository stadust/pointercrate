use super::{Player, PlayerWithDemonsAndRecords};
use crate::{
    citext::CiStr,
    error::PointercrateError,
    model::{
        creator::created_by, demon::EmbeddedDemon, player::PlayerWithNationality, user::User, By,
        Model,
    },
    operation::Get,
    permissions::{self, AccessRestrictions},
    schema::demons,
    Result,
};
use diesel::{result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

impl<'a> Get<&'a CiStr> for Player {
    fn get(name: &'a CiStr, connection: &PgConnection) -> Result<Self> {
        match Player::by(name).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                Player::insert(&name, connection).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for Player {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        match Player::by(id).first(connection) {
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

impl Get<i32> for PlayerWithNationality {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        match PlayerWithNationality::by(id).first(connection) {
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
    PlayerWithNationality: Get<T>,
{
    fn get(t: T, connection: &PgConnection) -> Result<Self> {
        let player = PlayerWithNationality::get(t, connection)?;
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
impl AccessRestrictions for PlayerWithNationality {
    fn pre_page_access(user: Option<&User>) -> Result<()> {
        permissions::demand(
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
            user,
        )
    }
}
impl AccessRestrictions for PlayerWithDemonsAndRecords {}
impl AccessRestrictions for Player {}
