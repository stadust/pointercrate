use crate::{error::Result, record::RecordStatus};

use futures::StreamExt;
use pointercrate_core::audit::{AuditLogEntry, AuditLogEntryType, NamedId};
use serde::Serialize;
use sqlx::PgConnection;

#[derive(Serialize)]
pub struct RecordModificationData {
    progress: Option<i16>,
    video: Option<String>,
    status: Option<RecordStatus>,
    player: Option<NamedId>,
    demon: Option<NamedId>,
}

/// Gets all audit log entries for the given record, in chronological order
pub async fn audit_log_for_record(record_id: i32, connection: &mut PgConnection) -> Result<Vec<AuditLogEntry<RecordModificationData>>> {
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
        entries.push(AuditLogEntry {
            time: addition.time,
            entry_id: addition.audit_id,
            id: record_id,
            user: NamedId {
                name: addition.name,
                id: addition.userid,
            },
            r#type: AuditLogEntryType::Addition,
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
                  WHERE record_modifications.id = $1
                  ORDER BY time"#,
            record_id
        )
        .fetch(&mut *connection);

        while let Some(modification) = modification_stream.next().await {
            let modification = modification?;

            entries.push(AuditLogEntry {
                time: modification.time,
                entry_id: modification.audit_id,
                id: record_id,
                r#type: AuditLogEntryType::Modification(RecordModificationData {
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
        entries.push(AuditLogEntry {
            time: deletion.time,
            entry_id: deletion.audit_id,
            id: record_id,
            user: NamedId {
                name: deletion.name,
                id: deletion.userid,
            },
            r#type: AuditLogEntryType::Deletion,
        });
    }

    Ok(entries)
}
