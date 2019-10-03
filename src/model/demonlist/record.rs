use crate::{
    citext::CiString,
    error::PointercrateError,
    model::{
        demonlist::{demon::MinimalDemon, player::DatabasePlayer},
        By, Model,
    },
    schema::records,
};
use derive_more::Display;
use diesel::{deserialize::Queryable, pg::Pg, Expression, Table};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};

mod delete;
mod get;
mod paginate;
mod patch;
mod post;

pub use self::{paginate::RecordPagination, patch::PatchRecord, post::Submission};
use crate::model::demonlist::Submitter;

#[derive(Debug, AsExpression, Eq, PartialEq, Clone, Copy, Hash, DbEnum)]
#[DieselType = "Record_status"]
pub enum RecordStatus {
    #[db_rename = "SUBMITTED"]
    Submitted,

    #[db_rename = "APPROVED"]
    Approved,

    #[db_rename = "REJECTED"]
    Rejected,
}

impl Default for RecordStatus {
    fn default() -> Self {
        RecordStatus::Submitted
    }
}

impl Display for RecordStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            RecordStatus::Submitted => write!(f, "submitted"),
            RecordStatus::Approved => write!(f, "approved"),
            RecordStatus::Rejected => write!(f, "rejected"),
        }
    }
}

impl Serialize for RecordStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RecordStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?.to_lowercase();

        match &string[..] {
            "approved" => Ok(RecordStatus::Approved),
            "submitted" => Ok(RecordStatus::Submitted),
            "rejected" => Ok(RecordStatus::Rejected),
            _ =>
                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(&string),
                    &"'approved', 'submitted' or 'rejected'",
                )),
        }
    }
}

#[derive(Debug, Queryable, Display, Hash)]
#[display(fmt = "{} {}% on {} (ID: {})", player_id, progress, demon_name, id)]
pub struct DatabaseRecord {
    id: i32,
    progress: i16,
    video: Option<String>,
    status: RecordStatus,
    player_id: i32,
    submitter_id: i32,
    demon_name: CiString,
}

impl Model for DatabaseRecord {
    type From = records::table;
    type Selection = <records::table as Table>::AllColumns;

    fn from() -> Self::From {
        records::table
    }

    fn selection() -> Self::Selection {
        records::all_columns
    }
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;
    use crate::model::demonlist::record::Record_status;

    records_pds (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        player_id -> Int4,
        player_name -> CiText,
        player_banned -> Bool,
        demon_name -> CiText,
        position -> Int2,
        submitter_id -> Int4,
        submitter_banned -> Bool,
    }
}

#[derive(Debug, Serialize, Hash, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct FullRecord {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: DatabasePlayer,
    pub demon: MinimalDemon,
    pub submitter: Option<Submitter>,
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;
    use crate::model::demonlist::record::Record_status;

    records_pd (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        submitter_id -> Int4,
        player_id -> Int4,
        player_name -> CiText,
        player_banned -> Bool,
        demon_name -> CiText,
        position -> Int2,
    }
}

#[derive(Debug, Serialize, Hash, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct Record {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: DatabasePlayer,
    pub demon: MinimalDemon,
    pub submitter: Option<i32>,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct EmbeddedRecordPD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: MinimalDemon,
    pub player: DatabasePlayer,
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;
    use crate::model::demonlist::record::Record_status;

    records_d (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        player -> Int4,
        demon_name -> CiText,
        position -> Int2,
    }
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{}% on {} (ID: {})", progress, demon, id)]
pub struct MinimalRecordD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: MinimalDemon,
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;
    use crate::model::demonlist::record::Record_status;

    records_p (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        demon -> CiText,
        player_id -> Int4,
        player_name -> CiText,
        player_banned -> Bool,
    }
}
#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} - {}% (ID: {})", player, progress, id)]
pub struct MinimalRecordP {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: DatabasePlayer,
}

impl Model for FullRecord {
    type From = records_pds::table;
    type Selection = <records_pds::table as Table>::AllColumns;

    fn from() -> Self::From {
        records_pds::table
    }

    fn selection() -> Self::Selection {
        records_pds::all_columns
    }
}

impl By<records_pds::id, i32> for FullRecord {}

impl Queryable<<<FullRecord as Model>::Selection as Expression>::SqlType, Pg> for FullRecord {
    type Row = (
        i32,
        i16,
        Option<String>,
        RecordStatus,
        i32,
        CiString,
        bool,
        CiString,
        i16,
        i32,
        bool,
    );

    fn build(row: Self::Row) -> Self {
        FullRecord {
            id: row.0,
            progress: row.1,
            video: row.2,
            status: row.3,
            player: DatabasePlayer {
                id: row.4,
                name: row.5,
                banned: row.6,
            },
            demon: MinimalDemon {
                position: row.8,
                name: row.7,
            },
            submitter: Some(Submitter {
                id: row.9,
                banned: row.10,
            }),
        }
    }
}

impl Model for Record {
    type From = records_pd::table;
    type Selection = <records_pd::table as Table>::AllColumns;

    fn from() -> Self::From {
        records_pd::table
    }

    fn selection() -> Self::Selection {
        records_pd::all_columns
    }
}

impl By<records_pd::id, i32> for Record {}

impl Queryable<<<Record as Model>::Selection as Expression>::SqlType, Pg> for Record {
    type Row = (
        // record
        i32,
        i16,
        Option<String>,
        RecordStatus,
        // submitter
        i32,
        // player
        i32,
        CiString,
        bool,
        // demon
        CiString,
        i16,
    );

    fn build(row: Self::Row) -> Self {
        Record {
            id: row.0,
            progress: row.1,
            video: row.2,
            status: row.3,
            submitter: Some(row.4),
            player: DatabasePlayer {
                id: row.5,
                name: row.6,
                banned: row.7,
            },
            demon: MinimalDemon {
                name: row.8,
                position: row.9,
            },
        }
    }
}

impl Record {
    pub fn validate_video(video: &mut String) -> Result<(), PointercrateError> {
        *video = crate::video::validate(video)?;

        Ok(())
    }
}

impl Model for EmbeddedRecordPD {
    type From = records_pd::table;
    type Selection = (
        records_pd::id,
        records_pd::progress,
        records_pd::status_,
        records_pd::video,
        records_pd::player_id,
        records_pd::player_name,
        records_pd::player_banned,
        records_pd::demon_name,
        records_pd::position,
    );

    fn from() -> Self::From {
        records_pd::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Queryable<<<EmbeddedRecordPD as Model>::Selection as Expression>::SqlType, Pg>
    for EmbeddedRecordPD
{
    type Row = (
        i32,
        i16,
        RecordStatus,
        Option<String>,
        i32,
        CiString,
        bool,
        CiString,
        i16,
    );

    fn build(row: Self::Row) -> Self {
        EmbeddedRecordPD {
            id: row.0,
            progress: row.1,
            status: row.2,
            video: row.3,
            player: DatabasePlayer {
                id: row.4,
                name: row.5,
                banned: row.6,
            },
            demon: MinimalDemon {
                name: row.7,
                position: row.8,
            },
        }
    }
}

impl Model for MinimalRecordD {
    type From = records_d::table;
    type Selection = <records_d::table as Table>::AllColumns;

    fn from() -> Self::From {
        records_d::table
    }

    fn selection() -> Self::Selection {
        records_d::all_columns
    }
}

impl Queryable<<<MinimalRecordD as Model>::Selection as Expression>::SqlType, Pg>
    for MinimalRecordD
{
    type Row = (i32, i16, Option<String>, RecordStatus, i32, CiString, i16);

    fn build(row: Self::Row) -> Self {
        MinimalRecordD {
            id: row.0,
            progress: row.1,
            video: row.2,
            status: row.3,
            // skip index 4, player id
            demon: MinimalDemon {
                name: row.5,
                position: row.6,
            },
        }
    }
}

impl Model for MinimalRecordP {
    type From = records_p::table;
    type Selection = <records_p::table as Table>::AllColumns;

    fn from() -> Self::From {
        records_p::table
    }

    fn selection() -> Self::Selection {
        records_p::all_columns
    }
}

impl Queryable<<<MinimalRecordP as Model>::Selection as Expression>::SqlType, Pg>
    for MinimalRecordP
{
    type Row = (
        i32,
        i16,
        Option<String>,
        RecordStatus,
        CiString,
        i32,
        CiString,
        bool,
    );

    fn build(row: Self::Row) -> Self {
        MinimalRecordP {
            id: row.0,
            progress: row.1,
            video: row.2,
            status: row.3,
            // skip index 4, demon name
            player: DatabasePlayer {
                id: row.5,
                name: row.6,
                banned: row.7,
            },
        }
    }
}
