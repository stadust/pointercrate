use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{creator::Creator, demon::MinimalDemon, player::DatabasePlayer},
    Result,
};
use futures::stream::StreamExt;
use sqlx::PgConnection;

impl Creator {
    pub async fn get(demon: &MinimalDemon, player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Creator> {
        let exists = sqlx::query!(
            "SELECT EXISTS (SELECT FROM creators WHERE creator=$1 AND demon = $2) AS result",
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
            Err(PointercrateError::ModelNotFound {
                model: "Creator",
                identified_by: format!("(demon, player) tuple ({},{})", demon.id, player.id),
            })
        }
    }
}

pub async fn creators_of(demon: &MinimalDemon, connection: &mut PgConnection) -> Result<Vec<DatabasePlayer>> {
    let mut stream = sqlx::query!(
        "SELECT players.id, players.name::TEXT, players.banned FROM players INNER JOIN creators ON players.id = creators.creator WHERE \
         creators.demon = $1",
        demon.id
    )
    .fetch(connection);
    let mut players = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        players.push(DatabasePlayer {
            id: row.id,
            name: CiString(row.name),
            banned: row.banned,
        })
    }

    Ok(players)
}

pub async fn created_by(player_id: i32, connection: &mut PgConnection) -> Result<Vec<MinimalDemon>> {
    let mut stream = sqlx::query!(
        "SELECT demons.id, demons.name::TEXT, demons.position FROM demons INNER JOIN creators ON demons.id = creators.demon WHERE \
         creators.creator=$1",
        player_id
    )
    .fetch(connection);
    let mut demons = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        demons.push(MinimalDemon {
            id: row.id,
            name: CiString(row.name),
            position: row.position,
        })
    }

    Ok(demons)
}
