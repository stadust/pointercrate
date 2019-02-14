use super::Record;
use crate::{
    error::PointercrateError,
    operation::{Delete, DeletePermissions},
    permissions::PermissionsSet,
    schema::records,
    Result,
};
use diesel::{delete, ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;

impl Delete for Record {
    fn delete(self, connection: &PgConnection) -> Result<()> {
        info!("Deleting record {}", self);

        delete(records::table)
            .filter(records::id.eq(self.id))
            .execute(connection)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}

impl DeletePermissions for Record {
    fn permissions() -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}
