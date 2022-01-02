use crate::{
    creator::Creator,
    demon::MinimalDemon,
    error::{DemonlistError, Result},
    player::DatabasePlayer,
};
use futures::stream::StreamExt;
use sqlx::PgConnection;

impl Creator {
    pub async fn get(demon: &MinimalDemon, player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Creator> {
        let exists = sqlx::query!(
            r#"SELECT EXISTS (SELECT FROM creators WHERE creator=$1 AND demon = $2) AS "result!: bool""#,
            player.id,
            demon.id
        )
        .fetch_one(connection)
        .await?
        .result;

        if exists {
            Ok(Creator {
                demon: demon.id,
                creator: player.id,
            })
        } else {
            Err(DemonlistError::CreatorNotFound {
                player_id: player.id,
                demon_id: demon.id,
            })
        }
    }
}

pub async fn creators_of(demon: &MinimalDemon, connection: &mut PgConnection) -> Result<Vec<DatabasePlayer>> {
    let mut stream = sqlx::query!(
        r#"SELECT players.id, players.name AS "name: String", players.banned FROM players INNER JOIN creators ON players.id = creators.creator WHERE 
         creators.demon = $1"#,
        demon.id
    )
    .fetch(connection);
    let mut players = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        players.push(DatabasePlayer {
            id: row.id,
            name: row.name,
            banned: row.banned,
        })
    }

    Ok(players)
}

pub async fn created_by(player_id: i32, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    query_many_demons!(
        connection,
        r#"SELECT demons.id, demons.name as "name: String", demons.position FROM demons INNER JOIN creators ON demons.id = creators.demon WHERE
         creators.creator=$1"#,
        player_id
    )
}
