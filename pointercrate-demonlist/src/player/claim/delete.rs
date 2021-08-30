use crate::{error::Result, player::claim::PlayerClaim};
use sqlx::PgConnection;

impl PlayerClaim {
    pub async fn delete(mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!(
            "DELETE FROM player_claims WHERE player_id = $1 AND member_id = $2",
            self.player_id,
            self.user_id
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}
