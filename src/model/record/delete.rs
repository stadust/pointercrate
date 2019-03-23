use super::Record;
use crate::{
    context::RequestContext, error::PointercrateError, operation::Delete, schema::records, Result,
};
use diesel::{delete, ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;

impl Delete for Record {
    fn delete(self, ctx: RequestContext, connection: &PgConnection) -> Result<()> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;
        ctx.check_if_match(&self)?;

        info!("Deleting record {}", self);

        delete(records::table)
            .filter(records::id.eq(self.id))
            .execute(connection)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}
