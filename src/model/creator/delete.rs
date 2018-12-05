use super::Creator;
use crate::{error::PointercrateError, operation::Delete, schema::creators, Result};
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
