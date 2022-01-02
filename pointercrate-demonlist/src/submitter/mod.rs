use crate::error::Result;
use derive_more::Display;
use serde::Serialize;
use sqlx::PgConnection;

pub use paginate::SubmitterPagination;
pub use patch::PatchSubmitter;
use pointercrate_core::etag::Taggable;

mod get;
mod paginate;
mod patch;
mod post;

#[derive(Debug, Serialize, Hash, Display, Copy, Clone)]
#[display(fmt = "{} (Banned: {})", id, banned)]
pub struct Submitter {
    pub id: i32,
    pub banned: bool,
}

impl Taggable for Submitter {}

impl Submitter {
    /// Gets the maximal and minimal submitter id currently in use
    ///
    /// The returned tuple is of the form (max, min)
    pub async fn extremal_submitter_ids(connection: &mut PgConnection) -> Result<(i32, i32)> {
        let row = sqlx::query!(r#"SELECT MAX(submitter_id) AS "max_id!: i32", MIN(submitter_id) AS "min_id!: i32" FROM submitters"#)
            .fetch_one(connection)
            .await?; // FIXME: crashes on empty table
        Ok((row.max_id, row.min_id))
    }
}
