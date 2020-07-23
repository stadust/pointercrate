use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{
        demon::MinimalDemon,
        player::DatabasePlayer,
        record::{note::notes_on, FullRecord, MinimalRecordD, MinimalRecordP, RecordStatus},
        submitter::Submitter,
    },
    Result,
};
use futures::stream::StreamExt;
use sqlx::{Error, PgConnection};

// Required until https://github.com/launchbadge/sqlx/pull/108 is merged
struct FetchedRecord {
    progress: i16,
    video: Option<String>,
    status: String,
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
                    status: RecordStatus::from_sql(&row.status),
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
                    notes: notes_on(id, connection).await?,
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
    struct Fetched {
        id: i32,
        progress: i16,
        video: Option<String>,
        demon_id: i32,
        name: String,
        position: i16,
    }

    let mut stream = sqlx::query_as!(
        Fetched,
        "SELECT records.id, progress, CASE WHEN players.link_banned THEN NULL ELSE records.video::text END, demons.id AS demon_id, \
         demons.name::text, demons.position FROM records INNER JOIN demons ON records.demon = demons.id INNER JOIN players ON players.id \
         = $1 WHERE status_ = 'APPROVED' AND records.player = $1",
        player.id
    )
    .fetch(connection);

    let mut records = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        records.push(MinimalRecordD {
            id: row.id,
            progress: row.progress,
            video: row.video,
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
    struct Fetched {
        id: i32,
        progress: i16,
        video: Option<String>,
        player_id: i32,
        name: String,
        banned: bool,
    }

    let mut stream = sqlx::query_as!(
        Fetched,
        "SELECT records.id, progress, CASE WHEN players.link_banned THEN NULL ELSE video::text END, players.id AS player_id, \
         players.name::text, players.banned FROM records INNER JOIN players ON records.player = players.id WHERE status_ = 'APPROVED' AND \
         records.demon = $1 ORDER BY progress DESC, id ASC",
        demon.id
    )
    .fetch(connection);

    let mut records = Vec::new();

    while let Some(row) = stream.next().await {
        let row = row?;

        records.push(MinimalRecordP {
            id: row.id,
            progress: row.progress,
            video: row.video,
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
