use super::Player;
use crate::{error::PointercrateError, operation::Delete, schema::players, Result};
use diesel::{delete, ExpressionMethods, PgConnection, RunQueryDsl};

impl Delete for Player {
    fn delete(self, connection: &PgConnection) -> Result<()> {
        delete(players::table)
            .filter(players::id.eq(self.id))
            .execute(connection)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}
