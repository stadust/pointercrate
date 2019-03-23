use super::Submitter;
use crate::{
    context::RequestContext,
    operation::{deserialize_non_optional, Patch},
    schema::submitters,
    Result,
};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

make_patch! {
    struct PatchSubmitter {
        banned: bool
    }
}

impl Patch<PatchSubmitter> for Submitter {
    fn patch(mut self, patch: PatchSubmitter, ctx: RequestContext) -> Result<Self> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;

        info!("Patching player {} with {}", self.id, patch);

        patch!(self, patch: banned);

        diesel::update(submitters::table)
            .filter(submitters::submitter_id.eq(&self.id))
            .set(submitters::banned.eq(&self.banned))
            .execute(ctx.connection())?;

        Ok(self)
    }
}
