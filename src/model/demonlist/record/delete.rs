use super::Record;
use crate::{
    context::RequestContext, error::PointercrateError, operation::Delete, schema::records, Result,
};
use diesel::{delete, ExpressionMethods, RunQueryDsl};
use log::info;

impl Delete for Record {
    fn delete(self, ctx: RequestContext) -> Result<()> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;
        ctx.check_if_match(&self)?;

        info!("Deleting record {}", self);

        delete(records::table)
            .filter(records::id.eq(self.id))
            .execute(ctx.connection())
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}
