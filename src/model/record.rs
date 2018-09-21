use super::{Demon, Player, Submitter};
use crate::{
    model::demon::PartialDemon,
    schema::{demons, players, records},
};
use diesel::{
    delete,
    deserialize::Queryable,
    expression::{bound::Bound, AsExpression},
    insert_into,
    pg::{Pg, PgConnection},
    query_dsl::{QueryDsl, RunQueryDsl},
    result::QueryResult,
    row::Row,
    serialize::Output,
    sql_types::{self, Text},
    types::{FromSql, FromSqlRow, IsNull, ToSql},
    BoolExpressionMethods, ExpressionMethods,
};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    io::Write,
};

#[derive(Debug, AsExpression, Eq, PartialEq, Clone, Copy, Hash)]
pub enum RecordStatus {
    Submitted,
    Approved,
    Rejected,
}

impl Display for RecordStatus {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            RecordStatus::Submitted => write!(f, "submitted"),
            RecordStatus::Approved => write!(f, "approved"),
            RecordStatus::Rejected => write!(f, "rejected"),
        }
    }
}

// Diesel trait impls

impl FromSql<Text, Pg> for RecordStatus {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let as_string: String = FromSql::<Text, Pg>::from_sql(bytes)?;

        Ok(match as_string.as_ref() {
            "SUBMITTED" => RecordStatus::Submitted,
            "APPROVED" => RecordStatus::Approved,
            "REJECTED" => RecordStatus::Rejected,
            _ => unreachable!(),
        })
    }
}
impl FromSqlRow<Text, Pg> for RecordStatus {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        FromSql::<Text, Pg>::from_sql(row.take())
    }
}

impl ToSql<Text, Pg> for RecordStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<dyn Error + Send + Sync + 'static>> {
        <str as ToSql<Text, Pg>>::to_sql(
            match self {
                RecordStatus::Submitted => "SUBMITTED",
                RecordStatus::Approved => "APPROVED",
                RecordStatus::Rejected => "REJECTED",
            },
            out,
        )
    }
}

impl AsExpression<Text> for RecordStatus {
    type Expression = diesel::expression::bound::Bound<Text, RecordStatus>;

    fn as_expression(self) -> Self::Expression {
        diesel::expression::bound::Bound::new(self)
    }
}

impl<'a> AsExpression<Text> for &'a RecordStatus {
    type Expression = diesel::expression::bound::Bound<Text, &'a RecordStatus>;

    fn as_expression(self) -> Self::Expression {
        diesel::expression::bound::Bound::new(self)
    }
}

impl Queryable<Text, Pg> for RecordStatus {
    type Row = String;

    fn build(row: Self::Row) -> Self {
        match row.as_ref() {
            "SUBMITTED" => RecordStatus::Submitted,
            "APPROVED" => RecordStatus::Approved,
            "REJECTED" => RecordStatus::Rejected,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Identifiable, Associations)]
#[table_name = "records"]
#[belongs_to(Player, foreign_key = "player")]
#[belongs_to(Submitter, foreign_key = "submitter")]
#[belongs_to(Demon, foreign_key = "demon")]
#[belongs_to(PartialDemon, foreign_key = "demon")]
pub struct Record {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: Player,
    pub submitter: i32,
    pub demon: PartialDemon,
}

impl Hash for Record {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.progress.hash(state);
        self.video.hash(state);
        self.status.hash(state);
        self.player.hash(state);
        self.submitter.hash(state);
        self.demon.name.hash(state);
    }
}

#[derive(Insertable, Debug)]
#[table_name = "records"]
struct NewRecord<'a> {
    progress: i16,
    video: Option<&'a str>,
    #[column_name = "status_"]
    status: RecordStatus, // TODO: add a DEFAULT 'SUBMITTED' to the column so this field wont be needed anymore
    player: i32,
    submitter: i32,
    demon: &'a str,
}

type AllColumns = (
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

const ALL_COLUMNS: AllColumns = (
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

type SqlType = (
    // record
    sql_types::Integer,
    sql_types::SmallInt,
    sql_types::Nullable<sql_types::Text>,
    sql_types::Text,
    // player
    sql_types::Integer,
    sql_types::Text,
    sql_types::Bool,
    // record
    sql_types::Integer,
    // demon
    sql_types::Text,
    sql_types::SmallInt,
);

type All = diesel::dsl::Select<diesel::dsl::InnerJoin<diesel::dsl::InnerJoin<records::table, demons::table>, players::table>, AllColumns>;

type WithId = diesel::dsl::Eq<records::id, Bound<sql_types::Int4, i32>>;
type ById = diesel::dsl::Filter<All, WithId>;

type WithVideo<'a> = diesel::dsl::Eq<records::video, Bound<sql_types::Nullable<sql_types::Text>, Option<&'a str>>>;
type ByVideo<'a> = diesel::dsl::Filter<All, WithVideo<'a>>;

type WithPlayerAndDemon<'a> = diesel::dsl::And<
    diesel::dsl::Eq<records::player, Bound<sql_types::Int4, i32>>,
    diesel::dsl::Eq<records::demon, Bound<sql_types::Text, &'a str>>,
>;
type ByPlayerAndDemon<'a> = diesel::dsl::Filter<All, WithPlayerAndDemon<'a>>;

type WithExisting<'a> = diesel::dsl::Or<WithPlayerAndDemon<'a>, WithVideo<'a>>;
type ByExisting<'a> = diesel::dsl::Filter<All, WithExisting<'a>>;

impl Record {
    pub fn all() -> All {
        records::table
            .inner_join(demons::table)
            .inner_join(players::table)
            .select(ALL_COLUMNS)
    }

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
        Record::all().filter(Record::with_player_and_demon(player, demon).or(records::video.eq(Some(video))))
    }

    pub fn insert(conn: &PgConnection, progress: i16, video: Option<&str>, player: i32, submitter: i32, demon: &str) -> QueryResult<i32> {
        let new = NewRecord {
            progress,
            video,
            status: RecordStatus::Submitted,
            player,
            submitter,
            demon,
        };

        insert_into(records::table).values(&new).returning(records::id).get_result(conn)
    }

    pub fn progress(&self) -> i16 {
        self.progress
    }

    pub fn status(&self) -> RecordStatus {
        self.status
    }

    pub fn delete(&self, conn: &PgConnection) -> QueryResult<()> {
        delete(records::table).filter(records::id.eq(self.id)).execute(conn).map(|_| ())
    }
}

impl Queryable<SqlType, Pg> for Record {
    type Row = (i32, i16, Option<String>, RecordStatus, i32, String, bool, i32, String, i16);

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
            },
        }
    }
}
