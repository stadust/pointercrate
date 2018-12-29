use super::{super::user::Permissions, Submitter};
use crate::{
    operation::{deserialize_non_optional, Hotfix, Patch},
    schema::submitters,
    Result,
};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use serde_derive::Deserialize;

make_patch! {
    struct PatchSubmitter {
        banned: bool
    }
}

impl Hotfix for PatchSubmitter {
    fn required_permissions(&self) -> Permissions {
        Permissions::ListModerator
    }
}

impl Patch<PatchSubmitter> for Submitter {
    fn patch(mut self, patch: PatchSubmitter, connection: &PgConnection) -> Result<Self> {
        patch!(self, patch: banned);

        diesel::update(submitters::table)
            .filter(submitters::submitter_id.eq(&self.id))
            .set(submitters::banned.eq(&self.banned))
            .execute(connection)?;

        Ok(self)
    }
}
