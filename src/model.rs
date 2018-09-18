use diesel::{
    backend::Backend,
    expression::Expression,
    serialize::Output,
    sql_types::Text,
    types::{IsNull, ToSql},
};
use gdcf::chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use schema::*;
use std::{error::Error, io::Write};

#[derive(Debug, AsExpression)]
pub enum AuditOperation {
    AddDemon,
    PatchDemon,
    AddRecord,
    RemoveRecord,
    PatchRecord,
    AddPlayer,
    RemovePlayer,
    PatchPlayer,
    BanSubmitter,
}

impl Expression for AuditOperation {
    type SqlType = Text;
}

impl<DB: Backend> ToSql<Text, DB> for AuditOperation {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> Result<IsNull, Box<Error + Send + Sync + 'static>> {
        <str as ToSql<Text, DB>>::to_sql(
            match self {
                AuditOperation::AddDemon => "ADD_DEMON",
                AuditOperation::PatchDemon => "PATCH_DEMON",
                AuditOperation::AddRecord => "ADD_RECORD",
                AuditOperation::RemoveRecord => "REMOVE_RECORD",
                AuditOperation::PatchRecord => "PATCH_RECORD",
                AuditOperation::AddPlayer => "ADD_PLAYER",
                AuditOperation::RemovePlayer => "REMOVE_PLAYER",
                AuditOperation::PatchPlayer => "PATCH_PLAYER",
                AuditOperation::BanSubmitter => "BAN_SUBMITTER",
            },
            out,
        )
    }
}

#[derive(Debug, AsExpression)]
pub enum RecordStatus {
    Submitted,
    Approved,
    Rejected,
}

impl Expression for RecordStatus {
    type SqlType = Text;
}

impl<DB: Backend> ToSql<Text, DB> for RecordStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> Result<IsNull, Box<Error + Send + Sync + 'static>> {
        <str as ToSql<Text, DB>>::to_sql(
            match self {
                RecordStatus::Submitted => "SUBMITTED",
                RecordStatus::Approved => "APPROVED",
                RecordStatus::Rejected => "REJECTED",
            },
            out,
        )
    }
}

#[derive(Queryable, Insertable, Debug, Identifiable)]
#[table_name = "players"]
pub struct Player {
    id: i32,
    name: String,
    banned: bool,
}

#[derive(Queryable, Insertable, Debug, Identifiable)]
#[table_name = "submitters"]
#[primary_key("submitter_id")]
pub struct Submitter {
    #[column_name = "submitter_id"]
    id: i32,

    #[column_name = "ip_address"]
    ip: IpNetwork,
    banned: bool,
}

#[derive(Queryable, Insertable, Debug)]
#[table_name = "members"]
pub struct User {
    #[column_name = "member_id"]
    id: i32,

    name: String,
    display_name: Option<String>,
    youtube_channel: Option<String>,

    password_hash: Vec<u8>,
    password_salt: Vec<u8>,

    // TODO: deal with this
    permissions: Vec<u8>,
}

#[derive(Queryable, Insertable, Debug, Identifiable)]
#[table_name = "demons"]
#[primary_key("name")]
pub struct Demon {
    name: String,
    position: i16,
    requirement: i16,
    description: Option<String>,
    notes: Option<String>,
    verifier: i32,
    publisher: i32,
}

#[derive(Queryable, Insertable, Debug, Identifiable, Associations)]
#[table_name = "records"]
#[belongs_to(Player, foreign_key = "player")]
#[belongs_to(Submitter, foreign_key = "submitter")]
#[belongs_to(Demon, foreign_key = "demon")]
pub struct Record {
    id: i32,
    progress: i16,
    video: Option<String>,
    #[column_name = "status_"]
    status: RecordStatus,
    player: i32,
    submitter: i32,
    demon: String,
}

#[derive(Queryable, Insertable, Debug, Identifiable)]
#[table_name = "audit_log"]
pub struct AuditLogEntry {
    id: i32,
    operation: AuditOperation,
    target: Option<String>,
    old_value: Option<String>,
    new_value: Option<String>,
    #[column_name = "time_"]
    time: NaiveDateTime,
    #[column_name = "list_mod"]
    user: Option<String>, // TODO: I have no idea why the fuck this is referencing a string column, what the fuck. the fuck. why past-me???
    demon: Option<String>,
    record: Option<i32>,
    player: Option<i32>,
}
