pub use self::{
    get::{current_list, list_at, published_by, verified_by},
    paginate::{DemonIdPagination, DemonPositionPagination},
    patch::PatchDemon,
    post::PostDemon,
};
use crate::{
    error::{DemonlistError, Result},
    list::List,
    player::DatabasePlayer,
    record::MinimalRecordP,
};
use derive_more::Display;
use log::info;
use pointercrate_core::etag::Taggable;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[macro_use]
mod get;
pub mod audit;
mod paginate;
mod patch;
mod post;

pub async fn recompute_rated_positions(connection: &mut PgConnection) -> Result<()> {
    sqlx::query("SELECT recompute_rated_positions()").execute(connection).await?;

    Ok(())
}

pub struct TimeShiftedDemon {
    pub current_demon: Demon,
    pub position_now: Option<i16>,
}

/// Struct modelling a demon. These objects are returned from the paginating `/demons/` endpoint
#[derive(Debug, Deserialize, Serialize, Hash, Display, Eq, PartialEq)]
#[display("{}", base)]
pub struct Demon {
    #[serde(flatten)]
    pub base: MinimalDemon,

    /// The minimal progress a [`Player`] must achieve on this [`Demon`] to have their record
    /// accepted
    pub requirement: i16,

    pub video: Option<String>,

    pub thumbnail: String,

    /// This [`Demon`]'s publisher
    pub publisher: DatabasePlayer,

    /// This [`Demon`]'s verifier
    pub verifier: DatabasePlayer,

    /// This ['Demons']'s Geometry Dash level ID
    ///
    /// This is automatically queried based on the level name, but can be manually overridden by a
    /// list mod.
    pub level_id: Option<u64>,

    /// Whether this [`Demon`] has a star rating in Geometry Dash
    pub rated: bool,
}

/// Absolutely minimal representation of a demon to be sent when a demon is part of another object
#[derive(Debug, Hash, Serialize, Deserialize, Display, PartialEq, Eq, Clone)]
#[display("{} (at {})", name, position)]
pub struct MinimalDemon {
    /// The [`Demon`]'s unique internal pointercrate ID
    pub id: i32,

    /// The [`Demon`]'s position on the demonlist
    ///
    /// Positions for consecutive demons are always consecutive positive integers
    pub position: i16,

    /// The [`Demon`]'s position on the Rated+ list
    pub rated_position: Option<i16>,

    /// The [`Demon`]'s Geometry Dash level name
    ///
    /// Note that the name doesn't need to be unique!
    pub name: String,
}

/// Struct modelling the "full" version of a demon.
///
/// In addition to containing publisher/verifier information it also contains a list of the demon's
/// creators and a list of accepted records
#[derive(Debug, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
#[display("{}", demon)]
pub struct FullDemon {
    #[serde(flatten)]
    pub demon: Demon,
    pub creators: Vec<DatabasePlayer>,
    pub records: Vec<MinimalRecordP>,
}

impl Taggable for FullDemon {
    fn patch_part(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.demon.hash(&mut hasher);
        hasher.finish()
    }
}

impl MinimalDemon {
    /// Queries the record requirement for this demon from the database without collecting any of
    /// the other data
    pub async fn requirement(&self, connection: &mut PgConnection) -> Result<i16> {
        Ok(sqlx::query!("SELECT requirement FROM demons WHERE id = $1", self.id)
            .fetch_one(connection)
            .await?
            .requirement)
    }

    /// Utility function for getting the position of a demon on a particular list
    pub fn position(&self, list: &List) -> Option<i16> {
        match *list {
            List::Demonlist => self.rated_position,
            List::RatedPlus => Some(self.position),
        }
    }

    /// Returns whether this demon is on the main list for all the lists it's on
    pub fn is_any_main(&self) -> bool {
        self.position <= crate::config::list_size() || self.rated_position.is_some_and(|p| p <= crate::config::list_size())
    }

    /// Returns whether this demon is on the legacy list for all the lists it's on
    pub fn is_all_legacy(&self) -> bool {
        self.position > crate::config::extended_list_size() && self.rated_position.map_or(true, |p| p > crate::config::extended_list_size())
    }
}

impl FullDemon {
    pub fn position(&self) -> i16 {
        self.demon.base.position
    }

    pub fn name(&self) -> &str {
        self.demon.base.name.as_ref()
    }
}

impl Demon {
    pub fn validate_requirement(requirement: i16) -> Result<()> {
        if !(0..=100).contains(&requirement) {
            return Err(DemonlistError::InvalidRequirement);
        }

        Ok(())
    }

    pub fn validate_level_id(level_id: i64) -> Result<u64> {
        if level_id < 1 {
            return Err(DemonlistError::InvalidLevelId);
        }

        Ok(level_id as u64)
    }

    pub async fn validate_position(position: i16, connection: &mut PgConnection) -> Result<()> {
        // To prevent holes from being created in the list, the new position must lie between 1 and (current
        // last position + 1), inclusive
        let maximal_position = Demon::max_position(connection).await? + 1;

        if position > maximal_position || position < 1 {
            return Err(DemonlistError::InvalidPosition { maximal: maximal_position });
        }

        Ok(())
    }

    /// Increments the position of all demons with positions equal to or greater than the given one,
    /// by one.
    async fn shift_down(starting_at: i16, connection: &mut PgConnection) -> Result<()> {
        info!("Shifting down all demons, starting at {}", starting_at);

        sqlx::query!("UPDATE demons SET position = position + 1 WHERE position >= $1", starting_at)
            .execute(connection)
            .await?;

        Ok(())
    }

    /// Gets the current max position a demon has, or `0` if there are no demons
    /// in the database
    pub async fn max_position(connection: &mut PgConnection) -> Result<i16> {
        Ok(sqlx::query!("SELECT MAX(position) as max_position FROM demons")
            .fetch_one(connection)
            .await?
            .max_position
            .unwrap_or(0))
    }

    pub fn score(&self, list: &List, progress: i16) -> f64 {
        if progress < self.requirement {
            return 0.0;
        }

        let position = match *list {
            List::RatedPlus => self.base.position,
            List::Demonlist => self.base.rated_position.unwrap_or_else(|| return 0),
        };

        let beaten_score = match position {
            56..=150 => 1.039035131_f64 * ((185.7_f64 * (-0.02715_f64 * position as f64).exp()) + 14.84_f64),
            36..=55 => 1.0371139743_f64 * ((212.61_f64 * 1.036_f64.powf(1_f64 - position as f64)) + 25.071_f64),
            21..=35 => ((250_f64 - 83.389_f64) * (1.0099685_f64.powf(2_f64 - position as f64)) - 31.152_f64) * 1.0371139743_f64,
            4..=20 => ((326.1_f64 * (-0.0871_f64 * position as f64).exp()) + 51.09_f64) * 1.037117142_f64,
            1..=3 => (-18.2899079915_f64 * position as f64) + 368.2899079915_f64,
            _ => 0_f64,
        };

        if progress != 100 {
            (beaten_score * (5f64.powf((progress - self.requirement) as f64 / (100f64 - self.requirement as f64)))) / 10f64
        } else {
            beaten_score
        }
    }
}
