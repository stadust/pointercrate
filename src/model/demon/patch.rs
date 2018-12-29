use super::Demon;
use crate::{
    model::{player::Player, user::Permissions},
    operation::{deserialize_non_optional, deserialize_optional, Get, Hotfix, Patch},
    schema::demons,
    Result,
};
use diesel::{Connection, ExpressionMethods, PgConnection, RunQueryDsl};
use serde_derive::Deserialize;

make_patch! {
    struct PatchDemon {
        name: String,
        position: i16,
        video: Option<String>,
        requirement: i16,
        verifier: String,
        publisher: String
    }
}

impl Hotfix for PatchDemon {
    fn required_permissions(&self) -> Permissions {
        Permissions::ListModerator
    }
}

impl Patch<PatchDemon> for Demon {
    fn patch(mut self, mut patch: PatchDemon, connection: &PgConnection) -> Result<Self> {
        validate_db!(patch, connection: Demon::validate_name[name], Demon::validate_position[position]);
        validate_nullable!(patch: Demon::validate_video[video]);

        let map = |name: &str| Player::get(name, connection);

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
                .filter(demons::name.eq(&self.name))
                .set((
                    demons::name.eq(&self.name),
                    demons::video.eq(&self.video),
                    demons::requirement.eq(&self.requirement),
                    demons::verifier.eq(&self.verifier.id),
                    demons::publisher.eq(&self.publisher.id),
                ))
                .execute(connection)?;

            Ok(self)
        })
    }
}
