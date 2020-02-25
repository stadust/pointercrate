pub use self::{paginate::SubmitterPagination, patch::PatchSubmitter};
use crate::{model::demonlist::record::MinimalRecordPD, Result};
use derive_more::Display;
use serde::Serialize;
use sqlx::PgConnection;
use std::hash::{Hash, Hasher};

mod get;
mod paginate;
mod patch;

#[derive(Debug, Serialize, Hash, Display, Copy, Clone)]
#[display(fmt = "{} (Banned: {})", id, banned)]
pub struct Submitter {
    pub id: i32,
    pub banned: bool,
}

#[derive(Debug, Serialize, Display)]
#[display(fmt = "{}", submitter)]
pub struct FullSubmitter {
    #[serde(flatten)]
    submitter: Submitter,
    records: Vec<MinimalRecordPD>,
}

impl Hash for FullSubmitter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.submitter.hash(state)
    }
}

impl Submitter {
    /// Gets the maximal and minimal submitter id currently in use
    ///
    /// The returned tuple is of the form (max, min)
    pub async fn extremal_submitter_ids(connection: &mut PgConnection) -> Result<(i32, i32)> {
        let row = sqlx::query!("SELECT MAX(submitter_id) AS max_id, MIN(submitter_id) AS min_id FROM submitters")
            .fetch_one(connection)
            .await?; // FIXME: crashes on empty table
        Ok((row.max_id, row.min_id))
    }
}
