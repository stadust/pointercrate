use crate::error::Result;

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
