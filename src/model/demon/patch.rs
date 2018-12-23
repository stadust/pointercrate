use super::Demon;
use crate::{
    error::PointercrateError,
    model::{player::Player, user::Permissions},
    operation::{deserialize_non_optional, deserialize_optional, Get, Hotfix, Patch},
    schema::demons,
    video, Result,
};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
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

        let map = |name| Player::name_to_id(name, connection);

        patch!(self, patch: name, video, requirement);
        try_map_patch!(self, patch: map => verifier, map => publisher);

        unimplemented!()
    }
}
