use crate::{
    creator::Creator,
    demon::MinimalDemon,
    error::{DemonlistError, Result},
    player::DatabasePlayer,
};
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize)]
pub struct PostCreator {
    pub creator: String,
}

impl Creator {
    pub async fn insert(demon: &MinimalDemon, player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Creator> {
        match Creator::get(demon, player, connection).await {
            Ok(_) => return Err(DemonlistError::CreatorExists),
            Err(DemonlistError::CreatorNotFound { .. }) => (),
            Err(err) => return Err(err),
        }

        let _ = sqlx::query!("INSERT INTO creators (creator, demon) VALUES ($1, $2)", player.id, demon.id)
            .execute(connection)
            .await?;

        Ok(Creator {
            demon: demon.id,
            creator: player.id,
        })
    }
}
