use crate::{
    cistring::CiString,
    model::{
        demonlist::{demon::MinimalDemon, record::MinimalRecordD},
        nationality::Nationality,
    },
};
use derive_more::Display;
use serde::Serialize;
use std::hash::{Hash, Hasher};

mod get;
mod paginate;
mod patch;

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Display, Clone)]
#[display(fmt = "{} (ID: {})", name, id)]
pub struct DatabasePlayer {
    pub id: i32,
    pub name: CiString,
    pub banned: bool,
}

#[derive(Debug, Serialize, Display)]
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
    pub name: CiString,
    pub rank: i64,
    pub score: f64,
    pub nationality: Option<Nationality>,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Display)]
#[display(fmt = "{}", base)]
pub struct Player {
    #[serde(flatten)]
    pub base: DatabasePlayer,

    pub nationality: Option<Nationality>,
}

impl Hash for FullPlayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.player.hash(state)
    }
}
