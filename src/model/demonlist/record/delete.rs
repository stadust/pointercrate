use crate::{model::demonlist::record::FullRecord, Result};
use log::info;
use sqlx::PgConnection;

impl FullRecord {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        info!("Deleting record {}", self);

        sqlx::query!("DELETE FROM record_notes WHERE record = $1", self.id)
            .execute(connection)
            .await?;
        sqlx::query!("DELETE FROM records WHERE id = $1", self.id)
            .execute(connection)
            .await?;

        Ok(())
    }
}
