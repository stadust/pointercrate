use super::Creator;
use crate::{
    context::RequestContext, error::PointercrateError, operation::Delete, schema::creators, Result,
};
use diesel::{delete, ExpressionMethods, RunQueryDsl};
use log::info;

impl Delete for Creator {
    fn delete(self, ctx: RequestContext) -> Result<()> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;

        info!(
            "Removing creator {} from demon {}",
            self.creator, self.demon
        );

        delete(creators::table)
            .filter(creators::demon.eq(self.demon))
            .filter(creators::creator.eq(self.creator))
            .execute(ctx.connection())
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}
