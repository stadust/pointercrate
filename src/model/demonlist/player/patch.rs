use super::{EmbeddedPlayer, PlayerWithDemonsAndRecords};
use crate::{
    citext::CiString,
    context::RequestContext,
    error::PointercrateError,
    model::{demonlist::player::ShortPlayer, nationality::Nationality, By},
    operation::{deserialize_non_optional, deserialize_optional, Get, Patch},
    schema::players,
    Result,
};
use diesel::{result::Error, Connection, ExpressionMethods, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

make_patch! {
    struct PatchPlayer {
        name: CiString,
        banned: bool,
        nationality: Option<String>,
    }
}

impl Patch<PatchPlayer> for ShortPlayer {
    fn patch(mut self, patch: PatchPlayer, ctx: RequestContext) -> Result<Self> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;
        ctx.check_if_match(&self)?;

        let connection = ctx.connection();

        info!("Patching player {} with {}", self, patch);

        connection.transaction(|| {
            if let Some(true) = patch.banned {
                if !self.inner.banned {
                    self.inner.ban(connection)?;
                }
            }

            if let Some(ref name) = patch.name {
                if *name != self.inner.name {
                    match EmbeddedPlayer::by(name.as_ref()).first(connection) {
                        Ok(player) => self.inner.merge(player, connection)?,
                        Err(Error::NotFound) => (),
                        Err(err) => return Err(PointercrateError::database(err)),
                    }
                }
            }

            if let Some(nationality) = patch.nationality {
                self.nationality = nationality
                    .map(|nation| Nationality::get(nation.as_ref(), ctx))
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
}

impl Patch<PatchPlayer> for PlayerWithDemonsAndRecords {
    fn patch(self, patch: PatchPlayer, ctx: RequestContext) -> Result<Self> {
        Ok(PlayerWithDemonsAndRecords {
            player: self.player.patch(patch, ctx)?,
            ..self
        })
    }
}
