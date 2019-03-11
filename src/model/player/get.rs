use super::{Player, PlayerWithDemonsAndRecords};
use crate::{
    error::PointercrateError,
    model::{creator::created_by, demon::EmbeddedDemon, user::User, Model},
    operation::Get,
    permissions::{self, AccessRestrictions},
    schema::demons,
    Result,
};
use diesel::{result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use crate::citext::CiStr;

impl<'a> Get<&'a CiStr> for Player {
    fn get(name: &'a CiStr, connection: &PgConnection) -> Result<Self> {
        match Player::by_name(name).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                Player::insert(&name, connection).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for Player {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        match Player::by_id(id).first(connection) {
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
    Player: Get<T>,
{
    fn get(t: T, connection: &PgConnection) -> Result<Self> {
        let player = Player::get(t, connection)?;

        Ok(PlayerWithDemonsAndRecords {
            records: Get::get(player.id, connection)?,
            created: created_by(player.id).load(connection)?,
            verified: EmbeddedDemon::all()
                .filter(demons::verifier.eq(&player.id))
                .load(connection)?,
            published: EmbeddedDemon::all()
                .filter(demons::publisher.eq(&player.id))
                .load(connection)?,
            player,
        })
    }
}

// Everyone can access player objects (through the stats viewer)
impl AccessRestrictions for Player {
    fn pre_page_access(user: Option<&User>) -> Result<()> {
        permissions::demand(
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
            user,
        )
    }
}
impl AccessRestrictions for PlayerWithDemonsAndRecords {}
