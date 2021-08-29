//! Module containing some basic structures for dealing with audit logs

use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Serialize)]
pub struct NamedId {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct AuditLogEntry<T> {
    pub time: NaiveDateTime,
    pub entry_id: i32,
    pub id: i32,
    pub user: NamedId,
    pub r#type: AuditLogEntryType<T>,
}

#[derive(Serialize)]
pub enum AuditLogEntryType<T> {
    Addition,
    Modification(T),
    Deletion,
}
