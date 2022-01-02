use crate::{
    error::{DemonlistError, Result},
    player::{claim::PlayerClaim, DatabasePlayer},
};
use sqlx::PgConnection;

pub struct ClaimBy {
    pub player: DatabasePlayer,
    pub verified: bool,
}

impl PlayerClaim {
    pub async fn verified_claim_on(player_id: i32, connection: &mut PgConnection) -> Result<Option<PlayerClaim>> {
        match sqlx::query!("SELECT member_id FROM player_claims WHERE player_id = $1 AND verified", player_id)
            .fetch_one(connection)
            .await
        {
            Ok(row) =>
                Ok(Some(PlayerClaim {
                    user_id: row.member_id,
                    player_id,
                    verified: true,
                })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn by_user(user_id: i32, connection: &mut PgConnection) -> Result<Option<ClaimBy>> {
        match sqlx::query!(
            r#"SELECT verified, player_id, players.name::text as "name!", players.banned FROM player_claims INNER JOIN players ON player_id=players.id
             WHERE member_id = $1"#,
            user_id
        )
        .fetch_one(connection)
        .await
        {
            Ok(row) =>
                Ok(Some(ClaimBy {
                    player: DatabasePlayer {
                        id: row.player_id,
                        name: row.name,
                        banned: row.banned,
                    },
                    verified: row.verified,
                })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn get(member_id: i32, player_id: i32, connection: &mut PgConnection) -> Result<PlayerClaim> {
        match PlayerClaim::by_user(member_id, connection).await? {
            Some(claim) if claim.player.id == player_id =>
                Ok(PlayerClaim {
                    user_id: member_id,
                    player_id,
                    verified: claim.verified,
                }),
            _ => Err(DemonlistError::ClaimNotFound { member_id, player_id }),
        }
    }
}
