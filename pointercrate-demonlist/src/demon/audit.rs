use crate::error::Result;

use crate::demon::MinimalDemon;
use chrono::NaiveDateTime;
use futures::StreamExt;
use pointercrate_core::audit::{AuditLogEntry, AuditLogEntryType, NamedId};
use serde::Serialize;
use sqlx::PgConnection;

#[derive(Serialize)]
pub struct DemonModificationData {
    pub name: Option<String>,
    pub position: Option<i16>,
    pub requirement: Option<i16>,
    pub video: Option<String>,
    pub verifier: Option<NamedId>,
    pub publisher: Option<NamedId>,
}

#[derive(Serialize, Debug)]
pub enum MovementReason {
    Added,
    Moved,
    OtherAddedAbove { other: NamedId },
    OtherMoved { other: NamedId },
    Unknown,
}

#[derive(Serialize, Debug)]
pub struct MovementLogEntry {
    reason: MovementReason,
    time: NaiveDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    old_position: Option<i16>,
}

pub async fn movement_log_for_demon(demon_id: i32, connection: &mut PgConnection) -> Result<Vec<MovementLogEntry>> {
    let audit_log = audit_log_for_demon(demon_id, connection).await?;

    let mut movement_log = Vec::new();

    for log_entry in audit_log {
        let time = log_entry.time;

        match log_entry.r#type {
            AuditLogEntryType::Addition =>
                movement_log.push(MovementLogEntry {
                    time,
                    old_position: None,
                    reason: MovementReason::Added,
                }),
            AuditLogEntryType::Modification(data) =>
                if let Some(old_position) = data.position {
                    // whenever a demon is moved, its position is first set to -1, all other demons are shifted, and
                    // then it is moved to its final position however since audit log entries with
                    // the same timestamp are not ordered in any way, trying to use this entry to draw conclusions about
                    // whether the demon we're looking at was moved leads to some very convoluted code .
                    if old_position == -1 {
                        continue
                    }

                    let id_of_moved = sqlx::query!("SELECT id FROM demon_modifications WHERE time = $1 AND position = -1", time)
                        .fetch_optional(&mut *connection)
                        .await?;

                    match id_of_moved {
                        Some(id) if id.id == demon_id =>
                            movement_log.push(MovementLogEntry {
                                reason: MovementReason::Moved,
                                time,
                                old_position: Some(old_position),
                            }),
                        Some(id) =>
                            match MinimalDemon::by_id(id.id, &mut *connection).await {
                                Ok(moved_demon) =>
                                    movement_log.push(MovementLogEntry {
                                        reason: MovementReason::OtherMoved {
                                            other: NamedId {
                                                id: id.id,
                                                name: Some(moved_demon.name),
                                            },
                                        },
                                        old_position: Some(old_position),
                                        time,
                                    }),
                                Err(_) =>
                                    movement_log.push(MovementLogEntry {
                                        reason: MovementReason::OtherMoved {
                                            other: NamedId { id: id.id, name: None },
                                        },
                                        old_position: Some(old_position),
                                        time,
                                    }),
                            },
                        None => {
                            // inner join because apparently some demons got deleted from the database >.>
                            let added_demon = sqlx::query!(
                                "SELECT demons.id, name::text FROM demon_additions INNER JOIN demons ON demon_additions.id = demons.id \
                                 WHERE time=$1",
                                time
                            )
                            .fetch_optional(&mut *connection)
                            .await?;

                            match added_demon {
                                Some(added_demon) =>
                                    movement_log.push(MovementLogEntry {
                                        reason: MovementReason::OtherAddedAbove {
                                            other: NamedId {
                                                name: added_demon.name, /* for once the sqlx fuckup of interpreting cases as optionals
                                                                         * works in our favor */
                                                id: added_demon.id,
                                            },
                                        },
                                        old_position: Some(old_position),
                                        time,
                                    }),
                                None =>
                                    movement_log.push(MovementLogEntry {
                                        reason: MovementReason::Unknown,
                                        old_position: Some(old_position),
                                        time,
                                    }),
                            }
                        },
                    }

                    // if old position = -1 there exists another modification entry with the same
                    // timestamp (due to transactional nature of movements). Means that _this_ demon
                    // was moved

                    // if there exists an entry with position = -1 from another demon, then this
                    // movement is the shift induced by that other demon being moved
                    // if there exists an addition entry for another demon with the same timestamp,
                    // then this movement is the shift induced by that addition
                    // otherwise, we do not know (the log entry is from before we kept track of
                    // audit logs accurately) :(
                },
            AuditLogEntryType::Deletion => unreachable!(),
        }
    }

    Ok(movement_log)
}

pub async fn audit_log_for_demon(demon_id: i32, connection: &mut PgConnection) -> Result<Vec<AuditLogEntry<DemonModificationData>>> {
    let mut entries = Vec::new();

    let addition_row = sqlx::query!(
        r#"SELECT time, audit_id, 
                  userid,
                  members.name AS "name?"
           FROM demon_additions LEFT OUTER JOIN members ON members.member_id = userid WHERE id = $1"#,
        demon_id
    )
    .fetch_optional(&mut *connection)
    .await?;

    if let Some(addition) = addition_row {
        entries.push(AuditLogEntry {
            time: addition.time,
            entry_id: addition.audit_id,
            id: demon_id,
            user: NamedId {
                name: addition.name,
                id: addition.userid,
            },
            r#type: AuditLogEntryType::Addition,
        });
    }

    let mut modification_stream = sqlx::query!(
        r#"SELECT time,
                audit_id,
                members.name as "username?",
                userid,
                demon_modifications.name::text,
                position,
                requirement,
                video,
                verifier,
                verifiers.name::text as verifier_name,
                publisher,
                publishers.name::text as publisher_name
           FROM demon_modifications
           LEFT OUTER JOIN members ON members.member_id = userid
           LEFT OUTER JOIN players AS verifiers ON verifier=verifiers.id
           LEFT OUTER JOIN players AS publishers ON publisher=publishers.id
           WHERE demon_modifications.id = $1
           ORDER BY time
                "#,
        demon_id
    )
    .fetch(connection);

    while let Some(modification) = modification_stream.next().await {
        let row = modification?;

        entries.push(AuditLogEntry {
            time: row.time,
            entry_id: row.audit_id,
            id: demon_id,
            r#type: AuditLogEntryType::Modification(DemonModificationData {
                name: row.name,
                position: row.position,
                requirement: row.requirement,
                video: row.video,
                verifier: match row.verifier {
                    Some(id) =>
                        Some(NamedId {
                            name: row.verifier_name,
                            id,
                        }),
                    None => None,
                },
                publisher: match row.publisher {
                    Some(id) =>
                        Some(NamedId {
                            name: row.publisher_name,
                            id,
                        }),
                    None => None,
                },
            }),
            user: NamedId {
                name: row.username,
                id: row.userid,
            },
        })
    }

    Ok(entries)
}
