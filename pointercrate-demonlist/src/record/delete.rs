use crate::{error::Result, record::FullRecord};
use log::info;
use sqlx::PgConnection;

impl FullRecord {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        info!("Deleting record {}", self);

        FullRecord::delete_by_id(self.id, &mut *connection).await?;

        self.player.update_score(connection).await?;

        Ok(())
    }

    /// `FullRecord::delete` should be preferred. Only exists to delete invalid submissions
    /// in the asychronous validation (which is why no score adjustment needs to take place here)
    pub async fn delete_by_id(record_id: i32, connection: &mut PgConnection) -> Result<()> {
        // Associated notes get deleted due to the ON DELETE CASCADE on record_notes.record

        sqlx::query!("DELETE FROM records WHERE id = $1", record_id)
            .execute(connection)
            .await?;

        Ok(())
    }
}
