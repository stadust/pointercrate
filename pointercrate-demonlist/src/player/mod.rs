pub use self::{
    paginate::{PlayerPagination, RankingPagination},
    patch::PatchPlayer,
};
use crate::{demon::MinimalDemon, nationality::Nationality, record::MinimalRecordD};
use derive_more::Display;
use pointercrate_core::etag::Taggable;
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub mod claim;
mod get;
mod paginate;
mod patch;

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Display, Clone, Deserialize)]
#[display(fmt = "{} (ID: {})", name, id)]
pub struct DatabasePlayer {
    pub id: i32,
    pub name: String,
    pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
#[display(fmt = "{}", player)]
pub struct FullPlayer {
    #[serde(flatten)]
    pub player: Player,
    pub records: Vec<MinimalRecordD>,
    pub created: Vec<MinimalDemon>,
    pub verified: Vec<MinimalDemon>,
    pub published: Vec<MinimalDemon>,
}

#[derive(Debug, PartialEq, Serialize, Display)]
#[display(fmt = "{} (ID: {}) at rank {} with score {}", name, id, rank, score)]
pub struct RankedPlayer {
    pub id: i32,
    pub name: String,
    pub rank: i64,
    pub score: f64,
    pub nationality: Option<Nationality>,
    #[serde(skip)]
    pub index: i64,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Display, Deserialize)]
#[display(fmt = "{}", base)]
pub struct Player {
    #[serde(flatten)]
    pub base: DatabasePlayer,

    pub nationality: Option<Nationality>,
}

impl Taggable for FullPlayer {
    fn patch_part(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.player.hash(&mut hasher);
        hasher.finish()
    }
}
