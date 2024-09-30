use crate::{demon::FullDemon, error::Result};
use log::info;
use sqlx::PgConnection;

impl FullDemon {
    pub async fn delete_demon(self, connection: &mut PgConnection) -> Result<()> {
        info!("Deleting demon {}", self);

        FullDemon::delete_all_records(self.demon.base.id, &self.demon.base.name, connection).await?;

        // creator is stored separately from demons
        FullDemon::delete_demon_data(self.demon.base.id, connection).await?;

        // prevent holes in the list of demons
        FullDemon::shift_up(self.demon.base.position, connection).await?;

        Ok(())
    }
    /// Delete all records on a demon
    pub async fn delete_all_records(demon_id: i32, demon_name: &String, connection: &mut PgConnection) -> Result<()> {
        // backup records before they're deleted
        sqlx::query!("
            INSERT INTO rec_backup (id, progress, video, status_, player, submitter, demon, raw_footage, demon_name) SELECT id, progress, video, status_, player, submitter, demon, raw_footage, $1::text FROM records WHERE demon = $2::integer;
            ", demon_name, demon_id)
            .execute(&mut *connection)
            .await?;

        // Delete records from records table
        sqlx::query!("DELETE FROM records WHERE demon = $1", demon_id)
            .execute(&mut *connection)
            .await?;

        Ok(())
    }
    /// Delete a demon from the database
    pub async fn delete_demon_data(demon_id: i32, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("DELETE FROM creators WHERE demon = $1", demon_id)
            .execute(&mut *connection)
            .await?;

        sqlx::query!("DELETE FROM demons WHERE id = $1", demon_id)
            .execute(&mut *connection)
            .await?;

        Ok(())
    }
}
