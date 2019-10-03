pub use self::{
    paginate::{PlayerPagination, RankingPagination},
    patch::PatchPlayer,
};
use crate::{
    citext::{CiStr, CiString},
    error::PointercrateError,
    model::{
        demonlist::{
            demon::MinimalDemon,
            record::{MinimalRecordD, RecordStatus},
        },
        nationality::Nationality,
        By, Model,
    },
    schema::{players, records},
    Result,
};
use derive_more::Display;
use diesel::{
    expression::Expression, pg::Pg, ExpressionMethods, PgConnection, QueryResult, Queryable,
    RunQueryDsl, Table,
};
use log::trace;
use serde_derive::Serialize;
use std::hash::{Hash, Hasher};

mod get;
mod paginate;
mod patch;

#[derive(Queryable, Debug, Hash, Eq, PartialEq, Serialize, Display)]
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

table! {
    use diesel::sql_types::*;
    use crate::citext::CiText;

    players_with_score (id) {
        id -> Int4,
        name -> CiText,
        rank -> Int8,
        score -> Double,
        index -> Int8,
        iso_country_code -> Nullable<Varchar>,
        nation -> Nullable<CiText>,
    }
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

table! {
    use diesel::sql_types::*;
    use crate::citext::CiText;

    players_n (id) {
        id -> Int4,
        name -> CiText,
        banned -> Bool,
        iso_country_code -> Nullable<Varchar>,
        nation -> Nullable<CiText>,
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Display)]
#[display(fmt = "{}", inner)]
pub struct Player {
    #[serde(flatten)]
    pub inner: DatabasePlayer,

    pub nationality: Option<Nationality>,
}

impl Hash for FullPlayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.player.hash(state)
    }
}

impl By<players::id, i32> for DatabasePlayer {}
impl By<players::name, &CiStr> for DatabasePlayer {}

impl By<players_n::id, i32> for Player {}
impl By<players_n::name, &CiStr> for Player {}

impl DatabasePlayer {
    pub fn ban(&self, conn: &PgConnection) -> QueryResult<()> {
        // delete all submissions
        diesel::delete(records::table)
            .filter(records::player.eq(self.id))
            .filter(records::status_.eq(RecordStatus::Submitted))
            .execute(conn)?;

        // Make sure the set of records the player has follows some stricter constraints
        // By default, there is a UNIQUE (player, demon, status_) constraint
        // The following query updates the table in a way that ensures that even a UNIQUE (player,
        // demon) would hold for the current player, which is required for the next step to
        // work
        diesel::sql_query(format!(
            include_str!("../../../sql/prepare_player_ban.sql"),
            self.id
        ))
        .execute(conn)?;

        // Reject all records
        diesel::update(records::table)
            .filter(records::player.eq(&self.id))
            .set(records::status_.eq(RecordStatus::Rejected))
            .execute(conn)?;

        Ok(())
    }

    pub fn merge(&self, with: DatabasePlayer, conn: &PgConnection) -> Result<()> {
        // FIXME: I had a serious headache while writing this code and didn't really think much
        // while doing so. Maybe look over it again at some point If both `self` and `with`
        // are registered as the creator of a level, delete `with` as creator
        trace!("Deleting duplicate creator entries");
        diesel::sql_query(format!(
            "DELETE FROM creators AS c1 WHERE c1.creator = {0} AND EXISTS (SELECT * FROM creators AS c2 WHERE c2.demon = c1.demon AND c2.creator = {1})",
            with.id, self.id
        ))
        .execute(conn)?;

        trace!("Transfering all creator entries from {} to {}", with, self);
        // Transfer all other creator entries to `self`
        diesel::sql_query(format!(
            "UPDATE creators SET creator = {1} WHERE creator = {0}",
            with.id, self.id
        ))
        .execute(conn)?;

        trace!(
            "Transfering all publisher/verifier entries from {} to {}",
            with,
            self
        );
        // Transfer all publisher/verifier entries to `self`
        diesel::sql_query(format!(
            "UPDATE demons SET publisher = {1} WHERE publisher = {0}",
            with.id, self.id
        ))
        .execute(conn)?;

        diesel::sql_query(format!(
            "UPDATE demons SET verifier = {1} WHERE verifier = {0}",
            with.id, self.id
        ))
        .execute(conn)?;

        diesel::sql_query(format!(
            include_str!("../../../sql/prepare_player_merge.sql"),
            self.id, with.id
        ))
        .execute(conn)?;

        diesel::update(records::table)
            .filter(records::player.eq(&with.id))
            .set(records::player.eq(self.id))
            .execute(conn)?;

        diesel::delete(players::table)
            .filter(players::id.eq(with.id))
            .execute(conn)
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}

impl Model for DatabasePlayer {
    type From = players::table;
    type Selection = (players::id, players::name, players::banned);

    fn from() -> Self::From {
        players::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Model for Player {
    type From = players_n::table;
    type Selection = <players_n::table as Table>::AllColumns;

    fn from() -> Self::From {
        players_n::table
    }

    fn selection() -> Self::Selection {
        players_n::all_columns
    }
}

impl Model for RankedPlayer {
    type From = players_with_score::table;
    type Selection = (
        players_with_score::id,
        players_with_score::name,
        players_with_score::rank,
        players_with_score::score,
        players_with_score::index,
        players_with_score::iso_country_code,
        players_with_score::nation,
    );

    fn from() -> Self::From {
        players_with_score::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Queryable<<<Player as Model>::Selection as Expression>::SqlType, Pg> for Player {
    type Row = (i32, CiString, bool, Option<String>, Option<CiString>);

    fn build(row: Self::Row) -> Self {
        let nationality = match (row.3, row.4) {
            (Some(country_code), Some(name)) => Some(Nationality::new(country_code, name)),
            _ => None,
        };

        Player {
            inner: DatabasePlayer {
                id: row.0,
                name: row.1,
                banned: row.2,
            },
            nationality,
        }
    }
}

impl Queryable<<<RankedPlayer as Model>::Selection as Expression>::SqlType, Pg> for RankedPlayer {
    type Row = (
        i32,
        CiString,
        i64,
        f64,
        i64,
        Option<String>,
        Option<CiString>,
    );

    fn build(row: Self::Row) -> Self {
        let nationality = match (row.5, row.6) {
            (Some(country_code), Some(name)) => Some(Nationality::new(country_code, name)),
            _ => None,
        };
        RankedPlayer {
            id: row.0,
            name: row.1,
            rank: row.2,
            score: row.3,
            nationality,
        }
    }
}
