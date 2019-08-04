use super::Submitter;
use crate::{
    context::RequestContext,
    model::demonlist::submitter::SubmitterWithRecords,
    operation::{deserialize_non_optional, Patch},
    schema::submitters,
    Result,
};
use diesel::{ExpressionMethods, RunQueryDsl};
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
        ctx.check_if_match(&self)?;

        info!("Patching player {} with {}", self.id, patch);

        patch!(self, patch: banned);

        diesel::update(submitters::table)
            .filter(submitters::submitter_id.eq(&self.id))
            .set(submitters::banned.eq(&self.banned))
            .execute(ctx.connection())?;

        Ok(self)
    }
}

impl Patch<PatchSubmitter> for SubmitterWithRecords {
    fn patch(self, patch: PatchSubmitter, ctx: RequestContext) -> Result<Self> {
        Ok(SubmitterWithRecords {
            submitter: self.submitter.patch(patch, ctx)?,
            ..self
        })
    }
}
