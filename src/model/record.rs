use super::{All, Model, Player};
use crate::{
    model::demon::PartialDemon,
    schema::{demons, players, records},
};
use diesel::{
    deserialize::Queryable,
    expression::bound::Bound,
    insert_into,
    pg::{Pg, PgConnection},
    query_dsl::{QueryDsl, RunQueryDsl},
    result::QueryResult,
    sql_types, BoolExpressionMethods, Expression, ExpressionMethods,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_derive::Serialize;
use std::fmt::{Display, Formatter};

mod delete;
mod get;
mod paginate;
mod post;

pub use self::{paginate::RecordPagination, post::Submission};

#[derive(Debug, AsExpression, Eq, PartialEq, Clone, Copy, Hash, DbEnum)]
#[DieselType = "Record_status"]
pub enum RecordStatus {
    #[db_rename = "SUBMITTED"]
    Submitted,

    #[db_rename = "APPROVED"]
    Approved,

    #[db_rename = "REJECTED"]
    Rejected,
}

impl Display for RecordStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            RecordStatus::Submitted => write!(f, "submitted"),
            RecordStatus::Approved => write!(f, "approved"),
            RecordStatus::Rejected => write!(f, "rejected"),
        }
    }
}

impl Serialize for RecordStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RecordStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct _Visitor;

        impl<'de> serde::de::Visitor<'de> for _Visitor {
            type Value = String;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<String, E>
            where
                E: std::error::Error,
            {
                Ok(v.into())
            }
        }

        let string = deserializer.deserialize_str(_Visitor)?.to_lowercase();

        match &string[..] {
            "approved" => Ok(RecordStatus::Approved),
            "submitted" => Ok(RecordStatus::Submitted),
            "rejected" => Ok(RecordStatus::Rejected),
            _ =>
                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(&string),
                    &"'approved', 'submitted' or 'rejected'",
                )),
        }
    }
}

#[derive(Debug, Identifiable, Associations, Serialize, Hash)]
#[table_name = "records"]
pub struct Record {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: Player,
    pub submitter: i32,
    pub demon: PartialDemon,
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {}% on {} (ID: {})",
            self.player, self.progress, self.demon, self.id
        )
    }
}

#[derive(Debug, Identifiable, Serialize, Hash, Queryable)]
#[table_name = "records"]
pub struct PartialRecord {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: i32,
    pub submitter: i32,
    pub demon: String,
}

#[derive(Insertable, Debug)]
#[table_name = "records"]
struct NewRecord<'a> {
    progress: i16,
    video: Option<&'a str>,
    #[column_name = "status_"]
    status: RecordStatus, /* TODO: add a DEFAULT 'SUBMITTED' to the column so this field wont
                           * be needed anymore */
    player: i32,
    submitter: i32,
    demon: &'a str,
}

type WithId = diesel::dsl::Eq<records::id, Bound<sql_types::Int4, i32>>;
type ById = diesel::dsl::Filter<All<Record>, WithId>;

type WithVideo<'a> =
    diesel::dsl::Eq<records::video, Bound<sql_types::Nullable<sql_types::Text>, Option<&'a str>>>;

type WithPlayerAndDemon<'a> = diesel::dsl::And<
    diesel::dsl::Eq<records::player, Bound<sql_types::Int4, i32>>,
    diesel::dsl::Eq<records::demon, Bound<sql_types::Text, &'a str>>,
>;
type ByPlayerAndDemon<'a> = diesel::dsl::Filter<All<Record>, WithPlayerAndDemon<'a>>;

type WithExisting<'a> = diesel::dsl::Or<WithPlayerAndDemon<'a>, WithVideo<'a>>;
type ByExisting<'a> = diesel::dsl::Filter<All<Record>, WithExisting<'a>>;

impl Record {
    pub fn by_id(id: i32) -> ById {
        Record::all().filter(records::id.eq(id))
    }

    pub fn with_player_and_demon(player: i32, demon: &str) -> WithPlayerAndDemon {
        records::player.eq(player).and(records::demon.eq(demon))
    }

    pub fn by_player_and_demon(player: i32, demon: &str) -> ByPlayerAndDemon {
        Record::all().filter(Record::with_player_and_demon(player, demon))
    }

    pub fn get_existing<'a>(player: i32, demon: &'a str, video: &'a str) -> ByExisting<'a> {
        Record::all()
            .filter(Record::with_player_and_demon(player, demon).or(records::video.eq(Some(video))))
    }

    pub fn insert(
        progress: i16, video: Option<&str>, player: i32, submitter: i32, demon: &str,
        conn: &PgConnection,
    ) -> QueryResult<i32> {
        let new = NewRecord {
            progress,
            video,
            status: RecordStatus::Submitted,
            player,
            submitter,
            demon,
        };

        insert_into(records::table)
            .values(&new)
            .returning(records::id)
            .get_result(conn)
    }

    pub fn progress(&self) -> i16 {
        self.progress
    }

    pub fn status(&self) -> RecordStatus {
        self.status
    }
}

impl Queryable<<<Record as Model>::Selection as Expression>::SqlType, Pg> for Record {
    type Row = (
        i32,
        i16,
        Option<String>,
        RecordStatus,
        i32,
        String,
        bool,
        i32,
        String,
        i16,
    );

    fn build(row: Self::Row) -> Self {
        Record {
            id: row.0,
            progress: row.1,
            video: row.2,
            status: row.3,
            player: Player {
                id: row.4,
                name: row.5,
                banned: row.6,
            },
            submitter: row.7,
            demon: PartialDemon {
                name: row.8,
                position: row.9,
                publisher: String::new(), // TODO: either omit here or write another view
            },
        }
    }
}

impl Model for PartialRecord {
    type From = records::table;
    type Selection = (
        records::id,
        records::progress,
        records::video,
        records::status_,
        records::player,
        records::submitter,
        records::demon,
    );

    fn from() -> Self::From {
        records::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Model for Record {
    #[allow(clippy::type_complexity)]
    type From = diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            diesel::query_source::joins::JoinOn<
                diesel::query_source::joins::Join<
                    records::table,
                    demons::table,
                    diesel::query_source::joins::Inner,
                >,
                diesel::expression::operators::Eq<records::demon, demons::name>,
            >,
            players::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::expression::operators::Eq<records::player, players::id>,
    >;
    type Selection = (
        records::id,
        records::progress,
        records::video,
        records::status_,
        players::id,
        players::name,
        players::banned,
        records::submitter,
        demons::name,
        demons::position,
    );

    fn from() -> Self::From {
        diesel::query_source::joins::Join::new(
            diesel::query_source::joins::Join::new(
                records::table,
                demons::table,
                diesel::query_source::joins::Inner,
            )
            .on(records::demon.eq(demons::name)),
            players::table,
            diesel::query_source::joins::Inner,
        )
        .on(records::player.eq(players::id))
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}
