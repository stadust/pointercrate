use crate::{error::Result, User};
use log::warn;
use sqlx::PgConnection;

impl User {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        warn!("Deleting user account {}", self);

        sqlx::query!("DELETE FROM members WHERE member_id = $1", self.id)
            .execute(connection)
            .await?;

        Ok(())
    }
}
