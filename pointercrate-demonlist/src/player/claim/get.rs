use crate::{
    error::{DemonlistError, Result},
    player::{claim::PlayerClaim, DatabasePlayer},
};
use sqlx::PgConnection;

impl PlayerClaim {
    pub async fn by_user(user_id: i32, connection: &mut PgConnection) -> Result<Option<PlayerClaim>> {
        match sqlx::query!("SELECT verified, player_id FROM player_claims WHERE member_id = $1", user_id)
            .fetch_one(connection)
            .await
        {
            Ok(row) =>
                Ok(Some(PlayerClaim {
                    user_id,
                    player_id: row.player_id,
                    verified: row.verified,
                })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn get(member_id: i32, player_id: i32, connection: &mut PgConnection) -> Result<PlayerClaim> {
        match PlayerClaim::by_user(member_id, connection).await? {
            Some(claim) if claim.player_id == player_id => Ok(claim),
            _ => Err(DemonlistError::ClaimNotFound { member_id, player_id }),
        }
    }
}
