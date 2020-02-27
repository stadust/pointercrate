use super::Creator;
use crate::Result;
use log::info;
use sqlx::PgConnection;

impl Creator {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        info!("Removing creator {} from demon {}", self.creator, self.demon);

        Ok(
            sqlx::query!("DELETE FROM creators WHERE creator = $1 AND demon = $2", self.creator, self.demon)
                .execute(connection)
                .await
                .map(|how_many| info!("Deletion of effected {} rows", how_many))?,
        )
    }
}
