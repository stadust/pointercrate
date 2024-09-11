use crate::{error::Result, player::claim::PlayerClaim};
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Deserialize)]
pub struct PatchPlayerClaim {
    pub verified: Option<bool>,
    pub lock_submissions: Option<bool>,
}

impl PlayerClaim {
    pub async fn apply_patch(mut self, patch: PatchPlayerClaim, connection: &mut PgConnection) -> Result<Self> {
        if let Some(verified) = patch.verified {
            self.set_verified(verified, &mut *connection).await?;
        }

        if let Some(lock_submissions) = patch.lock_submissions {
            self.set_lock_submissions(lock_submissions, connection).await?;
        }

        Ok(self)
    }

    pub async fn set_lock_submissions(&mut self, lock_submissions: bool, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!(
            "UPDATE player_claims SET lock_submissions = $3 WHERE member_id = $1 AND player_id = $2",
            self.user_id,
            self.player_id,
            lock_submissions
        )
        .execute(connection)
        .await?;

        self.lock_submissions = lock_submissions;

        Ok(())
    }

    pub async fn set_verified(&mut self, verified: bool, connection: &mut PgConnection) -> Result<()> {
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

        Ok(())
    }
}
