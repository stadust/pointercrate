//! Module containing all code relating to records on the demonlist
//!
//! Each record can have one of four statuses, 'approved', 'rejected', 'under consideration' or
//! 'submitted'. We will call a record of some player on some demon a (player, demon)-record.
//! We call a (player, demon)-record R _unique_ iff all other records by that player on the demon
//! have a different status than R. We call it _globally unique_ if R is the only record, regardless
//! of state, of player on demon.
//!
//! * 'approved' means that the record shows up on the demonlist and that further submissions for
//!   this (player, demon) pair are only allowed with a different video and higher progress. An
//!   approved record is unique. Whenever a record becomes 'accepted', all 'submitted' or 'under
//!   consideration' records with lower progress are removed.
//! * 'rejected' means that the record doesn't show up on the demonlist and that further submissions
//!   with that (player, demon) pair or that video will not be permitted. A rejected record is
//!   globally unique
//! * 'submitted' means that the record has been submitted. No further restrictions apply, meaning
//!   further submissions for this (demon, player) tuple are allowed. However as soon as one record
//!   for some (player, demon) tuple transitions from 'submitted' to ' approved' or 'rejected'. A
//!   submitted record is NOT unique
//! * 'under consideration' means essentially the same as 'submitted', only that all further
//!   submissions for this (demon, player) tuple are disallowed. Note that this does not mean that
//!   the 'under consideration' status makes. A record under consideration IS NOT UNIQUE!

pub use self::{
    get::{approved_records_by, approved_records_on, submission_count},
    paginate::RecordPagination,
    patch::PatchRecord,
    post::Submission,
};
use crate::{demon::MinimalDemon, error::Result, nationality::Nationality, player::DatabasePlayer, submitter::Submitter};
use derive_more::Display;
use pointercrate_core::etag::Taggable;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::PgConnection;
use std::{
    collections::hash_map::DefaultHasher,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

pub mod audit;
mod delete;
mod get;
pub mod note;
mod paginate;
mod patch;
mod post;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum RecordStatus {
    Submitted,
    Approved,
    Rejected,
    UnderConsideration,
}

impl RecordStatus {
    pub fn to_sql(self) -> String {
        match self {
            RecordStatus::Submitted => "SUBMITTED",
            RecordStatus::Approved => "APPROVED",
            RecordStatus::Rejected => "REJECTED",
            RecordStatus::UnderConsideration => "UNDER_CONSIDERATION",
        }
        .to_owned()
    }

    fn from_sql(sql: &str) -> Self {
        match sql {
            "SUBMITTED" => RecordStatus::Submitted,
            "APPROVED" => RecordStatus::Approved,
            "REJECTED" => RecordStatus::Rejected,
            "UNDER_CONSIDERATION" => RecordStatus::UnderConsideration,
            _ => panic!("invalid record state: {}", sql),
        }
    }
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
            RecordStatus::UnderConsideration => write!(f, "under consideration"),
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
            "under consideration" => Ok(RecordStatus::UnderConsideration),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&string),
                &"'approved', 'submitted', 'under consideration' or 'rejected'",
            )),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Display, Hash)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct FullRecord {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: DatabasePlayer,
    pub demon: MinimalDemon,
    pub submitter: Option<Submitter>,
    pub raw_footage: Option<String>,
}

impl Taggable for FullRecord {
    fn patch_part(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.progress.hash(&mut hasher);
        self.video.hash(&mut hasher);
        self.status.hash(&mut hasher);
        self.player.id.hash(&mut hasher);
        self.demon.id.hash(&mut hasher);
        // notes have sub-endpoint -> no hash
        // submitter cannot be patched -> no hash
        // raw footage cannot be patched -> no hash
        hasher.finish()
    }
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

#[derive(Debug, Hash, Serialize, Deserialize, Display, PartialEq, Eq)]
#[display(fmt = "{}% on {} (ID: {})", progress, demon, id)]
pub struct MinimalRecordD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: MinimalDemon,
}

#[derive(Debug, Hash, Serialize, Deserialize, Display, PartialEq, Eq)]
#[display(fmt = "{} - {}% (ID: {})", player, progress, id)]
pub struct MinimalRecordP {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: DatabasePlayer,
    pub nationality: Option<Nationality>,
}

impl FullRecord {
    pub async fn was_modified(&self, connection: &mut PgConnection) -> Result<bool> {
        Ok(sqlx::query!(
            r#"SELECT EXISTS (SELECT 1 FROM record_modifications WHERE id = $1 AND status_ IS NOT NULL) AS "was_modified!: bool""#,
            self.id
        )
        .fetch_one(&mut *connection)
        .await?
        .was_modified)
    }
}
