pub use self::{
    paginate::{PlayerPagination, RankingPagination},
    patch::PatchPlayer,
};
use crate::{
    citext::{CiStr, CiString},
    error::PointercrateError,
    model::{
        demonlist::{
            demon::EmbeddedDemon,
            record::{EmbeddedRecordD, RecordStatus},
        },
        nationality::Nationality,
        By, Model,
    },
    schema::{nationalities, players, records},
    Result,
};
use derive_more::Display;
use diesel::{
    expression::Expression,
    insert_into,
    pg::Pg,
    query_source::joins::{Join, JoinOn, LeftOuter},
    ExpressionMethods, NullableExpressionMethods, PgConnection, QueryResult, Queryable,
    RunQueryDsl,
};
use log::{info, trace};
use serde_derive::Serialize;
use std::hash::{Hash, Hasher};

mod get;
mod paginate;
mod patch;

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

#[derive(Queryable, Debug, Hash, Eq, PartialEq, Serialize, Display)]
#[display(fmt = "{} (ID: {})", name, id)]
pub struct EmbeddedPlayer {
    pub id: i32,
    pub name: CiString,
    pub banned: bool,
}

#[derive(Debug, PartialEq, Serialize, Display)]
#[display(fmt = "{} (ID: {}) at rank {} with score {}", name, id, rank, score)]
pub struct RankedPlayer2 {
    pub id: i32,
    pub name: CiString,
    pub rank: i64,
    pub score: f64,

    /// This field exists solely for pagination purposes, code should never interact with it.
    /// It is not possible to paginate reliable over the `rank` value, since it can contain
    /// duplicates. All our other values aren't ordered. This field is simply something that
    /// counts the rows returned from the database deterministically
    #[serde(skip)]
    pub index: i64,

    pub nationality: Option<Nationality>,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Display)]
#[display(fmt = "{}", inner)]
pub struct ShortPlayer {
    #[serde(flatten)]
    pub inner: EmbeddedPlayer,

    pub nationality: Option<Nationality>,
}

#[derive(Debug, Serialize, Display)]
#[display(fmt = "{}", player)]
pub struct PlayerWithDemonsAndRecords {
    #[serde(flatten)]
    pub player: ShortPlayer,
    pub records: Vec<EmbeddedRecordD>,
    pub created: Vec<EmbeddedDemon>,
    pub verified: Vec<EmbeddedDemon>,
    pub published: Vec<EmbeddedDemon>,
}

impl Hash for PlayerWithDemonsAndRecords {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.player.hash(state)
    }
}

#[derive(Insertable, Debug)]
#[table_name = "players"]
struct NewPlayer<'a> {
    name: &'a CiStr,
}

impl By<players::id, i32> for EmbeddedPlayer {}
impl By<players::name, &CiStr> for EmbeddedPlayer {}

impl By<players::id, i32> for ShortPlayer {}
impl By<players::name, &CiStr> for ShortPlayer {}

impl EmbeddedPlayer {
    pub fn insert(name: &CiStr, conn: &PgConnection) -> QueryResult<EmbeddedPlayer> {
        info!("Creating new player with name {}", name);

        insert_into(players::table)
            .values(&NewPlayer { name })
            .returning(EmbeddedPlayer::selection())
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

    pub fn merge(&self, with: EmbeddedPlayer, conn: &PgConnection) -> Result<()> {
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

impl Model for EmbeddedPlayer {
    type From = players::table;
    type Selection = (players::id, players::name, players::banned);

    fn from() -> Self::From {
        players::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Model for ShortPlayer {
    type From = JoinOn<
        Join<players::table, nationalities::table, LeftOuter>,
        diesel::dsl::Eq<
            players::nationality,
            diesel::expression::nullable::Nullable<nationalities::iso_country_code>,
        >,
    >;
    type Selection = (
        players::id,
        players::name,
        players::banned,
        diesel::expression::nullable::Nullable<nationalities::iso_country_code>,
        diesel::expression::nullable::Nullable<nationalities::nation>,
    );

    fn from() -> Self::From {
        Join::new(players::table, nationalities::table, LeftOuter)
            .on(players::nationality.eq(nationalities::iso_country_code.nullable()))
    }

    fn selection() -> Self::Selection {
        (
            players::id,
            players::name,
            players::banned,
            nationalities::iso_country_code.nullable(),
            nationalities::nation.nullable(),
        )
    }
}

impl Model for RankedPlayer2 {
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

impl Queryable<<<ShortPlayer as Model>::Selection as Expression>::SqlType, Pg> for ShortPlayer {
    type Row = (i32, CiString, bool, Option<String>, Option<CiString>);

    fn build(row: Self::Row) -> Self {
        let nationality = match (row.3, row.4) {
            (Some(country_code), Some(name)) => Some(Nationality::new(country_code, name)),
            _ => None,
        };

        ShortPlayer {
            inner: EmbeddedPlayer {
                id: row.0,
                name: row.1,
                banned: row.2,
            },
            nationality,
        }
    }
}

impl Queryable<<<RankedPlayer2 as Model>::Selection as Expression>::SqlType, Pg> for RankedPlayer2 {
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
        RankedPlayer2 {
            id: row.0,
            name: row.1,
            rank: row.2,
            score: row.3,
            index: row.4,
            nationality,
        }
    }
}
