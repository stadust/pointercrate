use crate::{error::Result, player::claim::PlayerClaim};
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Deserialize)]
pub struct PatchVerified {
    pub verified: bool,
}

impl PlayerClaim {
    pub async fn set_verified(mut self, verified: bool, connection: &mut PgConnection) -> Result<PlayerClaim> {
        sqlx::query!(
            "UPDATE player_claims SET verified = $3 WHERE member_id = $1 AND player_id = $2",
            self.user_id,
            self.player_id,
            verified
        )
        .execute(&mut *connection)
        .await?;

        self.verified = verified;

        if verified {
            // remove all other claims (verified or not) on that player
            sqlx::query!(
                "DELETE FROM player_claims WHERE player_id = $1 AND member_id <> $2",
                self.player_id,
                self.user_id
            )
            .execute(connection)
            .await?;
        }

        Ok(self)
    }
}
