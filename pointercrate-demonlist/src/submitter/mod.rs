use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;

pub use paginate::SubmitterPagination;
pub use patch::PatchSubmitter;
use pointercrate_core::etag::Taggable;

mod get;
mod paginate;
mod patch;
mod post;

#[derive(Debug, Deserialize, Serialize, Hash, Display, Copy, Clone, PartialEq, Eq)]
#[display(fmt = "{} (Banned: {})", id, banned)]
pub struct Submitter {
    pub id: i32,
    pub banned: bool,
}

impl Taggable for Submitter {}
