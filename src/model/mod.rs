use crate::schema::*;
use diesel::{
    backend::Backend,
    expression::Expression,
    serialize::Output,
    sql_types::Text,
    types::{FromSql, IsNull, ToSql},
};
use failure::Fail;
use gdcf::chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use std::{error::Error, io::Write};

pub mod demon;
pub mod player;
pub mod record;
pub mod submitter;

pub use self::{demon::Demon, player::Player, record::Record, submitter::Submitter};

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

impl<DB: Backend> FromSql<Text, DB> for AuditOperation
where
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> Result<Self, Box<Error + Send + Sync + 'static>> {
        Ok(match <String as FromSql<Text, DB>>::from_sql(bytes)?.as_ref() {
            "ADD_DEMON" => AuditOperation::AddDemon,
            "PATCH_DEMON" => AuditOperation::PatchDemon,
            "ADD_RECORD" => AuditOperation::AddRecord,
            "REMOVE_RECORD" => AuditOperation::RemoveRecord,
            "PATCH_RECORD" => AuditOperation::PatchRecord,
            "ADD_PLAYER" => AuditOperation::AddPlayer,
            "REMOVE_PLAYER" => AuditOperation::RemovePlayer,
            "PATCH_PLAYER" => AuditOperation::PatchPlayer,
            "BAN_SUBMITTER" => AuditOperation::BanSubmitter,
            _ => unreachable!(),
        })
    }
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
