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
use derive_more::Display;
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

mod delete;
mod get;
mod paginate;
mod patch;

table! {
    use diesel::sql_types::*;
    use crate::citext::Citext;

    players_with_score (id) {
        id -> Int4,
        name -> Citext,
        banned -> Bool,
        score -> Float,
        nationality -> Nullable<Varchar>,
    }
}

joinable!(players_with_score -> nationalities (nationality));
allow_tables_to_appear_in_same_query!(players_with_score, nationalities);

#[derive(Queryable, Debug, Hash, Eq, PartialEq, Serialize, Display)]
#[display(fmt = "{} (ID: {})", name, id)]
pub struct Player {
    pub id: i32,
    pub name: CiString,
    pub banned: bool,
}

#[derive(Queryable, Debug, Hash, PartialEq, Serialize, Display)]
#[display(fmt = "{} (ID: {})", name, id)]
pub struct PlayerWithScore {
    pub id: i32,
    pub name: CiString,
    pub banned: bool,
    pub score: f32,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Display)]
#[display(fmt = "{}", inner)]
pub struct PlayerWithNationality {
    #[serde(flatten)]
    pub inner: PlayerWithScore,

    pub nationality: Option<Nationality>,
}

#[derive(Debug, Serialize, Hash, Display)]
#[display(fmt = "{}", player)]
pub struct PlayerWithDemonsAndRecords {
    #[serde(flatten)]
    pub player: PlayerWithNationality,
    pub records: Vec<EmbeddedRecordD>,
    pub created: Vec<EmbeddedDemon>,
    pub verified: Vec<EmbeddedDemon>,
    pub published: Vec<EmbeddedDemon>,
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

impl By<players_with_score::id, i32> for Player {}
impl By<players_with_score::name, &CiStr> for Player {}

impl By<players_with_score::id, i32> for PlayerWithNationality {}
impl By<players_with_score::name, &CiStr> for PlayerWithNationality {}

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

impl Model for PlayerWithScore {
    type From = players_with_score::table;
    type Selection = (
        players_with_score::id,
        players_with_score::name,
        players_with_score::banned,
        players_with_score::score,
    );

    fn from() -> Self::From {
        players_with_score::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Model for PlayerWithNationality {
    type From = JoinOn<
        Join<players_with_score::table, nationalities::table, LeftOuter>,
        diesel::dsl::Eq<
            players_with_score::nationality,
            diesel::expression::nullable::Nullable<nationalities::iso_country_code>,
        >,
    >;
    type Selection = (
        players_with_score::id,
        players_with_score::name,
        players_with_score::banned,
        players_with_score::score,
        diesel::expression::nullable::Nullable<nationalities::iso_country_code>,
        diesel::expression::nullable::Nullable<nationalities::nation>,
    );

    fn from() -> Self::From {
        Join::new(players_with_score::table, nationalities::table, LeftOuter)
            .on(players_with_score::nationality.eq(nationalities::iso_country_code.nullable()))
    }

    fn selection() -> Self::Selection {
        (
            players_with_score::id,
            players_with_score::name,
            players_with_score::banned,
            players_with_score::score,
            nationalities::iso_country_code.nullable(),
            nationalities::nation.nullable(),
        )
    }
}

impl Queryable<<<PlayerWithNationality as Model>::Selection as Expression>::SqlType, Pg>
    for PlayerWithNationality
{
    type Row = (i32, CiString, bool, f32, Option<String>, Option<CiString>);

    fn build(row: Self::Row) -> Self {
        let nationality = match (row.4, row.5) {
            (Some(country_code), Some(name)) => Some(Nationality::new(country_code, name)),
            _ => None,
        };

        PlayerWithNationality {
            inner: PlayerWithScore {
                id: row.0,
                name: row.1,
                banned: row.2,
                score: row.3,
            },
            nationality,
        }
    }
}
