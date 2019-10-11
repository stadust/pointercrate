use super::{Demon, FullDemon};
use crate::{
    citext::{CiStr, CiString},
    context::RequestContext,
    model::demonlist::player::DatabasePlayer,
    operation::{deserialize_non_optional, deserialize_optional, Get, Patch},
    schema::demons,
    Result,
};
use diesel::{Connection, ExpressionMethods, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

make_patch! {
    struct PatchDemon {
        name: CiString,
        position: i16,
        video: Option<String>,
        requirement: i16,
        verifier: CiString,
        publisher: CiString
    }
}

impl Patch<PatchDemon> for Demon {
    fn patch(mut self, mut patch: PatchDemon, ctx: RequestContext) -> Result<Self> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;
        ctx.check_if_match(&self)?;

        info!("Patching demon {} with {}", self.name, patch);

        let connection = ctx.connection();

        validate_db!(patch, connection: Demon::validate_name[name], Demon::validate_position[position]);
        validate_nullable!(patch: Demon::validate_video[video]);

        let map = |name: &CiStr| DatabasePlayer::get(name, ctx);

        patch!(self, patch: name, video, requirement);
        try_map_patch!(self, patch: map => verifier, map => publisher);

        // We cannot move the PatchDemon object into the closure because we already moved data out
        // of it
        let position = patch.position;

        connection.transaction(move || {
            if let Some(position) = position {
                self.mv(position, connection)?
            }

            // alright, diesel::update(self) errors out for some reason
            diesel::update(demons::table)
                .filter(demons::id.eq(self.id))
                .set((
                    demons::name.eq(&self.name),
                    demons::video.eq(&self.video),
                    /* demons::requirement.eq(&self.requirement), */
                    demons::verifier.eq(&self.verifier.id),
                    demons::publisher.eq(&self.publisher.id),
                ))
                .execute(connection)?;

            Ok(self)
        })
    }
}

impl Patch<PatchDemon> for FullDemon {
    fn patch(self, patch: PatchDemon, ctx: RequestContext) -> Result<Self> {
        Ok(FullDemon {
            demon: self.demon.patch(patch, ctx)?,
            ..self
        })
    }
}
