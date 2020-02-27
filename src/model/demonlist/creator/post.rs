use super::Creator;
use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{demon::MinimalDemon, player::DatabasePlayer},
    Result,
};
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize)]
pub struct PostCreator {
    pub creator: CiString,
}

impl Creator {
    pub async fn insert(demon: &MinimalDemon, player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Creator> {
        match Creator::get(demon, player, connection).await {
            Ok(creator) => return Ok(creator),
            Err(PointercrateError::ModelNotFound { .. }) => (),
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
