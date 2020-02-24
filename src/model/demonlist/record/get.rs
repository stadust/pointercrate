use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{
        demon::MinimalDemon,
        player::DatabasePlayer,
        record::{FullRecord, MinimalRecordD, MinimalRecordP, RecordStatus},
        submitter::Submitter,
    },
    Result,
};
use futures::stream::StreamExt;
use sqlx::{Error, PgConnection};
use std::str::FromStr;

// Required until https://github.com/launchbadge/sqlx/pull/108 is merged
struct FetchedRecord {
    id: i32,
    progress: i16,
    video: Option<String>,
    status: String,
    notes: Option<String>,
    player_id: i32,
    player_name: String,
    player_banned: bool,
    demon_id: i32,
    demon_name: String,
    position: i16,
    submitter_id: i32,
    submitter_banned: bool,
}

impl FullRecord {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<FullRecord> {
        let result = sqlx::query_file_as!(FetchedRecord, "sql/record_by_id.sql", id)
            .fetch_one(connection)
            .await;

        match result {
            Ok(row) =>
                Ok(FullRecord {
                    id,
                    progress: row.progress,
                    video: row.video,
                    status: RecordStatus::from_str(&row.status)?,
                    player: DatabasePlayer {
                        id: row.player_id,
                        name: CiString(row.player_name),
                        banned: row.player_banned,
                    },
                    demon: MinimalDemon {
                        id: row.demon_id,
                        position: row.position,
                        name: CiString(row.demon_name),
                    },
                    submitter: Some(Submitter {
                        id: row.submitter_id,
                        banned: row.submitter_banned,
                    }),
                    notes: row.notes,
                }),

            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Record",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(err.into()),
        }
    }
}

pub async fn approved_records_by(player: &DatabasePlayer, connection: &mut PgConnection) -> Result<Vec<MinimalRecordD>> {
    let mut stream = sqlx::query!(
        "SELECT records.id, progress, records.video::text, demons.id AS demon_id, demons.name::text, demons.position FROM records INNER \
         JOIN demons ON records.demon = demons.id WHERE status_ = 'APPROVED' AND records.player = $1",
        player.id
    )
    .fetch(connection);

    let mut records = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        records.push(MinimalRecordD {
            id: row.id,
            progress: row.progress,
            video: if row.video.is_empty() { None } else { Some(row.video) },
            status: RecordStatus::Approved,
            demon: MinimalDemon {
                id: row.demon_id,
                position: row.position,
                name: CiString(row.name),
            },
        })
    }

    Ok(records)
}

pub async fn approved_records_on(demon: &MinimalDemon, connection: &mut PgConnection) -> Result<Vec<MinimalRecordP>> {
    let mut stream = sqlx::query!(
        "SELECT records.id, progress, video::text, players.id AS player_id, players.name::text, players.banned FROM records INNER JOIN \
         players ON records.player = players.id WHERE status_ = 'APPROVED' AND records.demon = $1",
        demon.id
    )
    .fetch(connection);

    let mut records = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        records.push(MinimalRecordP {
            id: row.id,
            progress: row.progress,
            video: if row.video.is_empty() { None } else { Some(row.video) },
            status: RecordStatus::Approved,
            player: DatabasePlayer {
                id: row.player_id,
                name: CiString(row.name),
                banned: row.banned,
            },
        })
    }

    Ok(records)
}
