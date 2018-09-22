use crate::schema::audit_log;
use diesel_derive_enum::DbEnum;
use gdcf::chrono::NaiveDateTime;

#[derive(Debug, AsExpression, Eq, PartialEq, Clone, Copy, Hash, DbEnum)]
#[DieselType = "Audit_operation"]
pub enum AuditOperation {
    #[db_rename = "ADD_DEMON"]
    AddDemon,

    #[db_rename = "PATCH_DEMON"]
    PatchDemon,

    #[db_rename = "ADD_RECORD"]
    AddRecord,

    #[db_rename = "REMOVE_RECORD"]
    RemoveRecord,

    #[db_rename = "PATCH_RECORD"]
    PatchRecord,

    #[db_rename = "ADD_PLAYER"]
    AddPlayer,

    #[db_rename = "REMOVE_PLAYER"]
    RemovePlayer,

    #[db_rename = "PATCH_PLAYER"]
    PatchPlayer,

    #[db_rename = "BAN_SUBMITTER"]
    BanSubmitter,
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
