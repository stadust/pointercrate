use crate::error::Result;

use crate::demon::MinimalDemon;
use chrono::{NaiveDateTime, NaiveTime};
use futures::StreamExt;
use pointercrate_core::audit::{AuditLogEntry, AuditLogEntryType, NamedId};
use serde::Serialize;
use sqlx::PgConnection;
use std::collections::HashMap;

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

    // only `None` for the last entry in case the demon has been deleted
    new_position: Option<i16>,
}

pub async fn movement_log_for_demon(demon_id: i32, connection: &mut PgConnection) -> Result<Vec<MovementLogEntry>> {
    let audit_log = audit_log_for_demon(demon_id, connection).await?;

    let mut movement_log = Vec::new();
    // map time -> NamedId keeping track of all additions
    let mut additions = HashMap::new();
    // map time -> NamedId keeping track when movements to -1 happened
    let mut all_moves = HashMap::new();

    {
        // non-lexical lifetimes working amazingly I see >.>
        let mut addition_stream = sqlx::query!(
            r#"SELECT time AS "time!", demon_additions.id AS "id!", demons.name::text FROM demon_additions LEFT OUTER JOIN demons ON demons.id = demon_additions.id"#
        )
        .fetch(&mut *connection);

        while let Some(row) = addition_stream.next().await {
            let row = row?;
            additions.insert(
                row.time,
                NamedId {
                    id: row.id,
                    name: row.name,
                },
            );
        }
    }

    {
        // non-lexical lifetimes working amazingly I see >.>
        let mut move_stream = sqlx::query!(
            "SELECT time, demon_modifications.id, demons.name::TEXT FROM demon_modifications LEFT OUTER JOIN demons ON demons.id = \
             demon_modifications.id WHERE demon_modifications.position = -1"
        )
        .fetch(&mut *connection);

        while let Some(row) = move_stream.next().await {
            let row = row?;
            all_moves.insert(
                row.time,
                NamedId {
                    id: row.id,
                    name: row.name,
                },
            );
        }
    }

    for log_entry in audit_log {
        let time = log_entry.time;

        match log_entry.r#type {
            AuditLogEntryType::Addition => movement_log.push(MovementLogEntry {
                time,
                new_position: None,
                reason: MovementReason::Added,
            }),
            AuditLogEntryType::Modification(data) => {
                if let Some(old_position) = data.position {
                    // whenever a demon is moved, its position is first set to -1, all other demons are shifted, and
                    // then it is moved to its final position however since audit log entries with
                    // the same timestamp are not ordered in any way, trying to use this entry to draw conclusions about
                    // whether the demon we're looking at was moved leads to some very convoluted code .
                    if old_position == -1 {
                        continue;
                    }

                    // update the previous entry's "new_position" field
                    movement_log.last_mut().map(|entry| entry.new_position = Some(old_position));

                    // if the time part of the datetime object is just zeros, the log entry was generated from deltas,
                    // meaning we can't figure out reasons accurately
                    if time.time() == NaiveTime::from_hms_opt(12, 0, 0).unwrap() {
                        movement_log.push(MovementLogEntry {
                            reason: MovementReason::Unknown,
                            time,
                            new_position: None,
                        });

                        continue;
                    }

                    let moved = all_moves.get(&time);

                    match moved {
                        Some(id) if id.id == demon_id => movement_log.push(MovementLogEntry {
                            reason: MovementReason::Moved,
                            time,
                            new_position: None,
                        }),
                        Some(id) => movement_log.push(MovementLogEntry {
                            reason: MovementReason::OtherMoved { other: id.clone() },
                            new_position: Some(old_position),
                            time,
                        }),
                        None => {
                            let added_demon = additions.get(&time);

                            match added_demon {
                                Some(added_demon) => movement_log.push(MovementLogEntry {
                                    reason: MovementReason::OtherAddedAbove {
                                        other: added_demon.clone(),
                                    },
                                    new_position: None,
                                    time,
                                }),
                                None => movement_log.push(MovementLogEntry {
                                    reason: MovementReason::Unknown,
                                    new_position: None,
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
                }
            },
            AuditLogEntryType::Deletion => unreachable!(),
        }
    }

    // update the last entry with the current position
    MinimalDemon::by_id(demon_id, &mut *connection).await.map(|minimal_demon| {
        movement_log
            .last_mut()
            .map(|entry| entry.new_position = Some(minimal_demon.position));
    })?;

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
                    Some(id) => Some(NamedId {
                        name: row.verifier_name,
                        id,
                    }),
                    None => None,
                },
                publisher: match row.publisher {
                    Some(id) => Some(NamedId {
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
