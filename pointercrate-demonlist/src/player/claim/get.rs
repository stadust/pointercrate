use crate::{
    error::{DemonlistError, Result},
    player::claim::PlayerClaim,
};
use sqlx::PgConnection;

impl PlayerClaim {
    pub async fn get(member_id: i32, player_id: i32, connection: &mut PgConnection) -> Result<PlayerClaim> {
        match sqlx::query!(
            "SELECT verified FROM player_claims WHERE member_id = $1 AND player_id = $2",
            member_id,
            player_id
        )
        .fetch_one(connection)
        .await
        {
            Ok(row) =>
                Ok(PlayerClaim {
                    user_id: member_id,
                    player_id,
                    verified: row.verified,
                }),
            Err(sqlx::Error::RowNotFound) => Err(DemonlistError::ClaimNotFound { member_id, player_id }),
            Err(err) => Err(err.into()),
        }
    }
}
