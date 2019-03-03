use super::Creator;
use crate::{
    error::PointercrateError, operation::Delete, permissions::PermissionsSet, schema::creators,
    Result,
};
use diesel::{delete, ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;

impl Delete for Creator {
    fn delete(self, connection: &PgConnection) -> Result<()> {
        info!(
            "Removing creator {} from demon {}",
            self.creator, self.demon
        );

        delete(creators::table)
            .filter(creators::demon.eq(self.demon))
            .filter(creators::creator.eq(self.creator))
            .execute(connection)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}
