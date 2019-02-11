use super::User;
use crate::{
    error::PointercrateError,
    operation::{Delete, DeletePermissions},
    permissions::PermissionsSet,
    schema::members,
    Result,
};
use diesel::{delete, ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;

impl Delete for User {
    fn delete(self, connection: &PgConnection) -> Result<()> {
        info!("Deleting user {}", self);

        delete(members::table)
            .filter(members::member_id.eq(self.id))
            .execute(connection)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}

impl DeletePermissions for User {
    fn permissions() -> PermissionsSet {
        perms!(Administrator)
    }
}
