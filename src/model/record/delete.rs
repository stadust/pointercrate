use super::Record;
use crate::{error::PointercrateError, operation::Delete, schema::records, Result};
use diesel::{delete, ExpressionMethods, PgConnection, RunQueryDsl};

impl Delete for Record {
    fn delete(self, connection: &PgConnection) -> Result<()> {
        delete(records::table)
            .filter(records::id.eq(self.id))
            .execute(connection)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}
