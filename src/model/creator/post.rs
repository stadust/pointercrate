use super::Creator;
use crate::{
    citext::{CiStr, CiString},
    error::PointercrateError,
    model::{Demon, EmbeddedPlayer},
    operation::{Get, Post, PostData},
    permissions::PermissionsSet,
    schema::creators,
    Result,
};
use diesel::{insert_into, Connection, PgConnection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PostCreator {
    pub creator: CiString,
}

#[derive(Debug, Insertable)]
#[table_name = "creators"]
struct NewCreator<'a> {
    demon: &'a CiStr,
    creator: i32,
}

impl<'a> Post<(&'a CiStr, &'a CiStr)> for Creator {
    fn create_from(
        (demon, player): (&'a CiStr, &'a CiStr), connection: &PgConnection,
    ) -> Result<Creator> {
        info!("Adding '{}' as creator of demon '{}'", player, demon);

        connection.transaction(|| {
            let demon = Demon::get(demon, connection)?;
            let player = EmbeddedPlayer::get(player, connection)?;

            insert_into(creators::table)
                .values(&NewCreator {
                    demon: demon.name.as_ref(),
                    creator: player.id,
                })
                .get_result(connection)
                .map_err(PointercrateError::database)
        })
    }
}

impl Post<(CiString, CiString)> for Creator {
    fn create_from(
        (demon, player): (CiString, CiString), connection: &PgConnection,
    ) -> Result<Creator> {
        Creator::create_from((demon.as_ref(), player.as_ref()), connection)
    }
}

impl<'a> Post<(i16, &'a CiStr)> for Creator {
    fn create_from(
        (position, player): (i16, &'a CiStr), connection: &PgConnection,
    ) -> Result<Self> {
        let demon = Demon::get(position, connection)?;

        Creator::create_from((demon.name.as_ref(), player), connection)
    }
}

impl Post<(i16, CiString)> for Creator {
    fn create_from((position, player): (i16, CiString), connection: &PgConnection) -> Result<Self> {
        Creator::create_from((position, player.as_ref()), connection)
    }
}

impl PostData for (i16, CiString) {
    fn required_permissions(&self) -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}

impl<'a> PostData for (i16, &'a CiStr) {
    fn required_permissions(&self) -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}

impl<'a> PostData for (&'a CiStr, &'a CiStr) {
    fn required_permissions(&self) -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}

impl<'a> PostData for (CiString, CiString) {
    fn required_permissions(&self) -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}
