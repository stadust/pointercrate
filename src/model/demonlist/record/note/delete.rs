use crate::{model::demonlist::record::note::Note, Result};
use sqlx::PgConnection;

impl Note {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("DELETE from record_notes WHERE id = $1", self.id)
            .execute(connection)
            .await?;

        Ok(())
    }
}
