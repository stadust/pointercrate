use super::Player;
use crate::{
    error::PointercrateError,
    operation::{deserialize_non_optional, Hotfix, Patch},
    permissions::PermissionsSet,
    schema::players,
    Result,
};
use diesel::{result::Error, Connection, ExpressionMethods, PgConnection, RunQueryDsl};
use serde_derive::Deserialize;

make_patch! {
    struct PatchPlayer {
        name: String,
        banned: bool
    }
}

impl Hotfix for PatchPlayer {
    fn required_permissions(&self) -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}

impl Patch<PatchPlayer> for Player {
    fn patch(mut self, patch: PatchPlayer, connection: &PgConnection) -> Result<Self> {
        connection.transaction(|| {
            if let Some(true) = patch.banned {
                if !self.banned {
                    self.ban(connection)?;
                }
            }

            if let Some(ref name) = patch.name {
                if *name != self.name {
                    match Player::by_name(&name).first(connection) {
                        Ok(player) => self.merge(player, connection)?,
                        Err(Error::NotFound) => (),
                        Err(err) => return Err(PointercrateError::database(err)),
                    }
                }
            }

            patch!(self, patch: name, banned);

            diesel::update(players::table)
                .filter(players::id.eq(&self.id))
                .set((
                    players::banned.eq(&self.banned),
                    players::name.eq(&self.name),
                ))
                .execute(connection)?;

            Ok(self)
        })
    }
}
