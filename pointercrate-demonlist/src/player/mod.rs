pub use self::{
    paginate::{PlayerPagination, RankedPlayer, RankingPagination},
    patch::PatchPlayer,
};
use crate::{demon::MinimalDemon, nationality::Nationality, record::MinimalRecordD};
use derive_more::Display;
use pointercrate_core::{error::CoreError, etag::Taggable};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
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

#[derive(Debug, Serialize, Deserialize, Display, PartialEq, Hash)]
#[display(fmt = "{}", player)]
pub struct FullPlayer {
    #[serde(flatten)]
    pub player: Player,
    pub records: Vec<MinimalRecordD>,
    pub created: Vec<MinimalDemon>,
    pub verified: Vec<MinimalDemon>,
    pub published: Vec<MinimalDemon>,
}

#[derive(Debug, PartialEq, Serialize, Display, Deserialize)]
#[display(fmt = "{}", base)]
pub struct Player {
    #[serde(flatten)]
    pub base: DatabasePlayer,

    /// This [`Player`]'s score on the stats viewer
    ///
    /// This value is cached in the `score` column of the `players` table, and not computed on-demand!
    /// Thus it needs to be updated on any event that can affect a player's score. These are
    /// - Record updates
    ///   * Record status updated (to approved, or from approved)
    ///   * Record progress updated
    ///   * Record holder updated
    ///   * Record Added
    /// - Demon updates
    ///   * Demon movement/addition (recompute all scores)
    ///   * Demon requirement updated (recompute all scores)
    ///   * Demon verifier updated
    /// - Player updates
    ///   * Player banned
    ///   * Player objects merged
    pub score: f64,
    pub nationality: Option<Nationality>,
}

// `f64` does not implement hash. Most things in the pointercrate frontend only display score with an accuracy of two digits after the dot,
// so hashing only this part should be fine for ETag purposes.
impl Hash for Player {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        ((self.score * 100f64) as u64).hash(state);
        self.nationality.hash(state);
    }
}

impl Taggable for FullPlayer {
    fn patch_part(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.player.hash(&mut hasher);
        hasher.finish()
    }
}

impl DatabasePlayer {
    /// Recomputes this player's score and updates it in the database.
    pub async fn update_score(&self, connection: &mut PgConnection) -> Result<f64, CoreError> {
        // No need to specially handle banned players - they have no approved records, so `score_of_player` will return 0
        let new_score = sqlx::query!(
            "UPDATE players SET score = coalesce(score_of_player($1), 0) WHERE id = $1 RETURNING score",
            self.id
        )
        .fetch_one(&mut *connection)
        .await?;

        sqlx::query!("UPDATE nationalities SET score = coalesce(score_of_nation(nationalities.iso_country_code), 0) FROM players WHERE players.id = $1 AND players.nationality = nationalities.iso_country_code", self.id).execute(&mut *connection).await?;
        sqlx::query!("UPDATE subdivisions SET score = coalesce(score_of_subdivision(subdivisions.nation, subdivisions.iso_code), 0) FROM players WHERE players.id = $1 AND players.nationality = subdivisions.nation AND players.subdivision = subdivisions.iso_code", self.id).execute(&mut *connection).await?;

        Ok(new_score.score)
    }
}

pub async fn recompute_scores(connection: &mut PgConnection) -> Result<(), CoreError> {
    sqlx::query!("SELECT recompute_player_scores();").execute(&mut *connection).await?;
    sqlx::query!("SELECT recompute_nation_scores();").execute(&mut *connection).await?;
    sqlx::query!("SELECT recompute_subdivision_scores();").execute(connection).await?;
    Ok(())
}
