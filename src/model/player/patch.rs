use super::{Player, PlayerWithDemonsAndRecords};
use crate::{
    citext::CiString,
    error::PointercrateError,
    model::{nationality::Nationality, player::PlayerWithNationality, By},
    operation::{deserialize_non_optional, deserialize_optional, Get, Patch},
    permissions::PermissionsSet,
    schema::players,
    Result,
};
use diesel::{result::Error, Connection, ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

make_patch! {
    struct PatchPlayer {
        name: CiString,
        banned: bool,
        nationality: Option<String>,
    }
}

impl Patch<PatchPlayer> for PlayerWithNationality {
    fn patch(mut self, patch: PatchPlayer, connection: &PgConnection) -> Result<Self> {
        info!("Patching player {} with {}", self, patch);

        connection.transaction(|| {
            if let Some(true) = patch.banned {
                if !self.inner.banned {
                    self.inner.ban(connection)?;
                }
            }

            if let Some(ref name) = patch.name {
                if *name != self.inner.name {
                    match Player::by(name.as_ref()).first(connection) {
                        Ok(player) => self.inner.merge(player, connection)?,
                        Err(Error::NotFound) => (),
                        Err(err) => return Err(PointercrateError::database(err)),
                    }
                }
            }

            if let Some(nationality) = patch.nationality {
                self.nationality = nationality
                    .map(|nation| Nationality::get(nation.as_ref(), connection))
                    .transpose()?;
            }

            patch!(self.inner, patch: name, banned);

            diesel::update(players::table)
                .filter(players::id.eq(&self.inner.id))
                .set((
                    players::banned.eq(&self.inner.banned),
                    players::name.eq(&self.inner.name),
                    players::nationality.eq(&self.nationality.as_ref().map(|n| &n.country_code)),
                ))
                .execute(connection)?;

            Ok(self)
        })
    }

    fn permissions_for(&self, _: &PatchPlayer) -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}

impl Patch<PatchPlayer> for PlayerWithDemonsAndRecords {
    fn patch(self, patch: PatchPlayer, connection: &PgConnection) -> Result<Self> {
        let PlayerWithDemonsAndRecords {
            player,
            records,
            created,
            verified,
            published,
        } = self;

        let player = player.patch(patch, connection)?;

        Ok(PlayerWithDemonsAndRecords {
            player,
            records,
            created,
            verified,
            published,
        })
    }

    fn permissions_for(&self, _: &PatchPlayer) -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}
