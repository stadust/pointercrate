use crate::model::demonlist::record::MinimalRecordPD;
use derive_more::Display;
use serde::Serialize;
use std::hash::{Hash, Hasher};

mod get;
// mod paginate;
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
