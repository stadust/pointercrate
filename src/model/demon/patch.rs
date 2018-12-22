use super::Demon;
use crate::{
    error::PointercrateError,
    model::user::Permissions,
    operation::{deserialize_patch, Hotfix, Patch, PatchField},
    schema::demons,
    video, Result,
};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use serde_derive::Deserialize;

make_patch! {
    struct PatchDemon {
        name: String,
        position: i16,
        video: String,
        requirement: i16,
        verifier: i32,
        publisher: i32
    }
}

impl Hotfix for PatchDemon {
    fn required_permissions(&self) -> Permissions {
        Permissions::ListModerator
    }
}

impl Patch<PatchDemon> for Demon {
    fn patch(mut self, mut patch: PatchDemon, connection: &PgConnection) -> Result<Self> {
        patch
            .name
            .validate_against_database(Demon::validate_name, connection)?;
        patch
            .position
            .validate_against_database(Demon::validate_position, connection)?;
        patch.video.validate(Demon::validate_video)?;

        unimplemented!()
    }
}
