use crate::{model::demonlist::record::RecordStatus, Result};
use chrono::NaiveDateTime;
use futures::StreamExt;
use serde::Serialize;
use sqlx::PgConnection;

#[derive(Serialize)]
pub struct NamedId {
    id: i32,
    name: Option<String>,
}

#[derive(Serialize)]
pub struct RecordModificationData {
    progress: Option<i16>,
    video: Option<String>,
    status: Option<RecordStatus>,
    player: Option<NamedId>,
    demon: Option<NamedId>,
}

#[derive(Serialize)]
pub struct RecordEntry {
    time: NaiveDateTime,
    audit_id: i32,
    record_id: i32,
    user: NamedId,
    r#type: RecordEntryType,
}

#[derive(Serialize)]
pub enum RecordEntryType {
    Addition,
    Modification(RecordModificationData),
    Deletion,
}

/// Gets all audit log entries for the given record, in chronological order
pub async fn entries_for_record(record_id: i32, connection: &mut PgConnection) -> Result<Vec<RecordEntry>> {
    let mut entries = Vec::new();

    let addition_row = sqlx::query!(
        r#"SELECT time, audit_id, 
                  userid,
                  members.name AS "name?"
                  FROM record_additions LEFT OUTER JOIN members ON members.member_id = userid WHERE id = $1"#,
        record_id
    )
    .fetch_optional(&mut *connection)
    .await?;

    if let Some(addition) = addition_row {
        entries.push(RecordEntry {
            time: addition.time,
            audit_id: addition.audit_id,
            record_id,
            user: NamedId {
                name: addition.name,
                id: addition.userid,
            },
            r#type: RecordEntryType::Addition,
        });
    }

    {
        // Has to be in block because it doesn't unborrow the connection otherwise. No idea why
        let mut modification_stream = sqlx::query!(
            r#"SELECT time, 
                  audit_id,
                  members.name AS "username?",
                  userid,
                  progress,
                  record_modifications.video,
                  status_::TEXT,
                  players.name::TEXT AS player_name,
                  player AS player_id,
                  demons.name::TEXT AS demon_name,
                  demon AS demon_id
                  FROM record_modifications 
                  LEFT OUTER JOIN members ON members.member_id = userid
                  LEFT OUTER JOIN players ON players.id = player
                  LEFT OUTER JOIN demons ON demons.id = demon
                  WHERE record_modifications.id = $1"#,
            record_id
        )
        .fetch(&mut *connection);

        while let Some(modification) = modification_stream.next().await {
            let modification = modification?;

            entries.push(RecordEntry {
                time: modification.time,
                audit_id: modification.audit_id,
                record_id,
                r#type: RecordEntryType::Modification(RecordModificationData {
                    progress: modification.progress,
                    status: modification.status_.as_deref().map(RecordStatus::from_sql),
                    player: match modification.player_id {
                        Some(id) =>
                            Some(NamedId {
                                name: modification.player_name,
                                id,
                            }),
                        _ => None,
                    },
                    demon: match modification.demon_id {
                        Some(id) =>
                            Some(NamedId {
                                name: modification.demon_name,
                                id,
                            }),
                        _ => None,
                    },
                    video: modification.video,
                }),
                user: NamedId {
                    name: modification.username,
                    id: modification.userid,
                },
            })
        }
    }

    let deletion_row = sqlx::query!(
        r#"SELECT time, audit_id, 
                  userid,
                  members.name AS "name?"
                  FROM record_deletions LEFT OUTER JOIN members ON members.member_id = userid WHERE id = $1"#,
        record_id
    )
    .fetch_optional(&mut *connection)
    .await?;

    if let Some(deletion) = deletion_row {
        entries.push(RecordEntry {
            time: deletion.time,
            audit_id: deletion.audit_id,
            record_id,
            user: NamedId {
                name: deletion.name,
                id: deletion.userid,
            },
            r#type: RecordEntryType::Deletion,
        });
    }

    Ok(entries)
}
