use super::Demon;
use crate::{
    error::PointercrateError,
    model::user::Permissions,
    operation::{deserialize_patch, Hotfix, Patch, PatchField},
    schema::demons,
    Result,
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
    fn patch(mut self, patch: PatchDemon, connection: &PgConnection) -> Result<Self> {
        unimplemented!()
    }
}
