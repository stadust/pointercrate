pub use self::{
    get::{approved_records_by, approved_records_on, submitted_by},
    paginate::RecordPagination,
    patch::PatchRecord,
    post::Submission,
};
use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{demon::MinimalDemon, player::DatabasePlayer, submitter::Submitter},
    ratelimit::RatelimitScope::RecordSubmission,
    Result,
};
use derive_more::Display;
use log::Record;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::PgConnection;
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

mod delete;
mod get;
mod paginate;
mod patch;
mod post;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum RecordStatus {
    Submitted,
    Approved,
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

impl FromStr for RecordStatus {
    type Err = PointercrateError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "submitted" => Ok(RecordStatus::Submitted),
            "approved" => Ok(RecordStatus::Approved),
            "rejected" => Ok(RecordStatus::Rejected),
            _ =>
                Err(PointercrateError::InvalidInternalStateError {
                    cause: "Encountered a record state other than 'approved', 'submitted' or 'rejected'",
                }),
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
    pub notes: Option<String>,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct MinimalRecordPD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: MinimalDemon,
    pub player: DatabasePlayer,
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

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} - {}% (ID: {})", player, progress, id)]
pub struct MinimalRecordP {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: DatabasePlayer,
}

impl FullRecord {
    /// Gets the maximal and minimal submitter id currently in use
    ///
    /// The returned tuple is of the form (max, min)
    pub async fn extremal_record_ids(connection: &mut PgConnection) -> Result<(i32, i32)> {
        let row = sqlx::query!("SELECT MAX(id) AS max_id, MIN(id) AS min_id FROM records")
            .fetch_one(connection)
            .await?; // FIXME: crashes on empty table
        Ok((row.max_id, row.min_id))
    }
}
