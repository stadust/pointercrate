use super::Creator;
use crate::{
    error::PointercrateError,
    operation::{Delete, DeletePermissions},
    schema::creators,
    Result, model::user::PermissionsSet
};
use diesel::{delete, ExpressionMethods, PgConnection, RunQueryDsl};

impl Delete for Creator {
    fn delete(self, connection: &PgConnection) -> Result<()> {
        delete(creators::table)
            .filter(creators::demon.eq(self.demon))
            .filter(creators::creator.eq(self.creator))
            .execute(connection)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}

impl DeletePermissions for Creator {
    fn permissions() -> PermissionsSet {
        perms!(ListModerator or ListAdministrator)
    }
}