use super::Record;
use crate::{
    context::RequestContext, error::PointercrateError, model::demonlist::record::DatabaseRecord,
    operation::Delete, schema::records, Result,
};
use diesel::{delete, ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;

impl Delete for Record {
    fn delete(self, ctx: RequestContext) -> Result<()> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;
        ctx.check_if_match(&self)?;

        delete_by_id(self.id, ctx.connection())
    }
}

impl Delete for DatabaseRecord {
    fn delete(self, ctx: RequestContext) -> Result<()> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;
        ctx.check_if_match(&self)?;

        delete_by_id(self.id, ctx.connection())
    }
}

fn delete_by_id(id: i32, connection: &PgConnection) -> Result<()> {
    info!("Deleting record with id {}", id);

    delete(records::table)
        .filter(records::id.eq(id))
        .execute(connection)
        .map(|_| ())
        .map_err(PointercrateError::database)
}
