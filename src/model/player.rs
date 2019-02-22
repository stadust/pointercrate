use super::{All, Model};
use crate::{
    model::{
        demon::EmbeddedDemon,
        record::{EmbeddedRecord, RecordStatus},
    },
    operation::Delete,
    schema::{players, records},
    Result,
};
use diesel::{
    expression::bound::Bound,
    insert_into,
    sql_types::{self, BigInt, Double, Integer, Text},
    ExpressionMethods, PgConnection, QueryDsl, QueryResult, RunQueryDsl,
};
use log::{info, trace};
use serde_derive::Serialize;
use std::fmt::{Display, Formatter};

mod delete;
mod get;
mod paginate;
mod patch;

pub use self::{paginate::PlayerPagination, patch::PatchPlayer};

#[derive(Queryable, Debug, Identifiable, Hash, Eq, PartialEq, Serialize)]
#[table_name = "players"]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub banned: bool,
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.name, self.id)
    }
}

#[derive(Debug, Serialize, Hash)]
pub struct PlayerWithDemonsAndRecords {
    #[serde(flatten)]
    pub player: Player,
    pub records: Vec<EmbeddedRecord>,
    pub created: Vec<EmbeddedDemon>,
    pub verified: Vec<EmbeddedDemon>,
    pub published: Vec<EmbeddedDemon>,
}

#[derive(Debug, QueryableByName)]
pub struct RankedPlayer {
    #[sql_type = "Integer"]
    id: i32,

    #[sql_type = "Text"]
    name: String,

    #[sql_type = "BigInt"]
    rank: i64,

    #[sql_type = "Double"]
    score: f64,
}

#[derive(Insertable, Debug)]
#[table_name = "players"]
struct NewPlayer<'a> {
    name: &'a str,
}

type WithName<'a> = diesel::dsl::Eq<players::name, Bound<sql_types::Text, &'a str>>;
type ByName<'a> = diesel::dsl::Filter<All<Player>, WithName<'a>>;

type WithId = diesel::dsl::Eq<players::id, Bound<sql_types::Int4, i32>>;
type ById = diesel::dsl::Filter<All<Player>, WithId>;

impl Player {
    pub fn by_name(name: &str) -> ByName {
        Player::all().filter(players::name.eq(name))
    }

    pub fn by_id(id: i32) -> ById {
        Player::all().filter(players::id.eq(id))
    }

    pub fn insert(name: &str, conn: &PgConnection) -> QueryResult<Player> {
        info!("Creating new player with name {}", name);

        insert_into(players::table)
            .values(&NewPlayer { name })
            .get_result(conn)
    }

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
            include_str!("../../sql/prepare_player_ban.sql"),
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

    pub fn merge(&self, with: Player, conn: &PgConnection) -> Result<()> {
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
            include_str!("../../sql/prepare_player_merge.sql"),
            self.id, with.id
        ))
        .execute(conn)?;

        diesel::update(records::table)
            .filter(records::player.eq(&with.id))
            .set(records::player.eq(self.id))
            .execute(conn)?;

        with.delete(conn)?;

        Ok(())
    }
}

impl Model for Player {
    type From = players::table;
    type Selection = (players::id, players::name, players::banned);

    fn from() -> Self::From {
        players::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}
