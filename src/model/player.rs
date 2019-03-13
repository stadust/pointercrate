pub use self::{paginate::PlayerPagination, patch::PatchPlayer};
use super::Model;
use crate::{
    citext::{CiStr, CiString, CiText},
    model::{
        demon::EmbeddedDemon,
        nationality::Nationality,
        record::{EmbeddedRecordD, RecordStatus},
        By,
    },
    operation::Delete,
    schema::{nationalities, players, records},
    Result,
};
use diesel::{
    expression::Expression,
    insert_into,
    pg::Pg,
    query_source::joins::{Join, JoinOn, LeftOuter},
    sql_types::{BigInt, Double, Integer},
    ExpressionMethods, NullableExpressionMethods, PgConnection, QueryResult, Queryable,
    RunQueryDsl,
};
use log::{info, trace};
use serde_derive::Serialize;
use std::fmt::{Display, Formatter};

mod delete;
mod get;
mod paginate;
mod patch;

// TODO: use that crate to derive Display lul

#[derive(Queryable, Debug, Identifiable, Hash, Eq, PartialEq, Serialize)]
#[table_name = "players"]
pub struct Player {
    pub id: i32,
    pub name: CiString,
    pub banned: bool,
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.name, self.id)
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
pub struct PlayerWithNationality {
    #[serde(flatten)]
    pub inner: Player,

    pub nationality: Option<Nationality>,
}

impl Display for PlayerWithNationality {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Serialize, Hash)]
pub struct PlayerWithDemonsAndRecords {
    #[serde(flatten)]
    pub player: PlayerWithNationality,
    pub records: Vec<EmbeddedRecordD>,
    pub created: Vec<EmbeddedDemon>,
    pub verified: Vec<EmbeddedDemon>,
    pub published: Vec<EmbeddedDemon>,
}

impl Display for PlayerWithDemonsAndRecords {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.player)
    }
}

#[derive(Debug, QueryableByName)]
pub struct RankedPlayer {
    #[sql_type = "Integer"]
    pub id: i32,

    #[sql_type = "CiText"]
    pub name: CiString,

    #[sql_type = "BigInt"]
    pub rank: i64,

    #[sql_type = "Double"]
    pub score: f64,
}

#[derive(Insertable, Debug)]
#[table_name = "players"]
struct NewPlayer<'a> {
    name: &'a CiStr,
}

impl By<players::id, i32> for Player {}
impl By<players::name, &CiStr> for Player {}

impl By<players::id, i32> for PlayerWithNationality {}
impl By<players::name, &CiStr> for PlayerWithNationality {}

impl Player {
    pub fn insert(name: &CiStr, conn: &PgConnection) -> QueryResult<Player> {
        info!("Creating new player with name {}", name);

        insert_into(players::table)
            .values(&NewPlayer { name })
            .returning(Player::selection())
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

impl Model for PlayerWithNationality {
    type From = JoinOn<
        Join<players::table, nationalities::table, LeftOuter>,
        diesel::dsl::Eq<
            players::nationality,
            diesel::expression::nullable::Nullable<nationalities::nation>,
        >,
    >;
    type Selection = (
        players::id,
        players::name,
        players::banned,
        diesel::expression::nullable::Nullable<nationalities::nation>,
        diesel::expression::nullable::Nullable<nationalities::iso_country_code>,
    );

    fn from() -> Self::From {
        Join::new(players::table, nationalities::table, LeftOuter)
            .on(players::nationality.eq(nationalities::nation.nullable()))
    }

    fn selection() -> Self::Selection {
        (
            players::id,
            players::name,
            players::banned,
            nationalities::nation.nullable(),
            nationalities::iso_country_code.nullable(),
        )
    }
}

impl Queryable<<<PlayerWithNationality as Model>::Selection as Expression>::SqlType, Pg>
    for PlayerWithNationality
{
    type Row = (i32, CiString, bool, Option<String>, Option<String>);

    fn build(row: Self::Row) -> Self {
        let nationality = match (row.3, row.4) {
            (Some(name), Some(country_code)) => Some(Nationality { name, country_code }),
            _ => None,
        };

        PlayerWithNationality {
            inner: Player {
                id: row.0,
                name: row.1,
                banned: row.2,
            },
            nationality,
        }
    }
}
