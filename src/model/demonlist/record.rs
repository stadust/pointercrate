use crate::{
    error::PointercrateError,
    model::{
        demonlist::{demon::EmbeddedDemon, player::EmbeddedPlayer},
        All, Model,
    },
    schema::{demons, players, records},
};
use derive_more::Display;
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
use std::fmt::{Display, Formatter};

mod delete;
mod get;
mod paginate;
mod patch;
mod post;

pub use self::{paginate::RecordPagination, patch::PatchRecord, post::Submission};
use crate::{
    citext::{CiStr, CiString, CiText},
    model::By,
};

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

impl Default for RecordStatus {
    fn default() -> Self {
        RecordStatus::Submitted
    }
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
        let string = String::deserialize(deserializer)?.to_lowercase();

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

#[derive(Debug, Identifiable, Serialize, Hash, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
#[table_name = "records"]
pub struct Record {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: EmbeddedPlayer,
    pub submitter: Option<i32>,
    pub demon: EmbeddedDemon,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct EmbeddedRecordPD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: EmbeddedDemon,
    pub player: EmbeddedPlayer,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{}% on {} (ID: {})", progress, demon, id)]
pub struct EmbeddedRecordD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: EmbeddedDemon,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} - {}% (ID: {})", player, progress, id)]
pub struct EmbeddedRecordP {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: EmbeddedPlayer,
}

#[derive(Insertable, Debug)]
#[table_name = "records"]
struct NewRecord<'a> {
    progress: i16,
    video: Option<&'a str>,
    #[column_name = "status_"]
    status: RecordStatus,
    player: i32,
    submitter: i32,
    demon: &'a CiStr,
}

type WithVideo<'a> =
    diesel::dsl::Eq<records::video, Bound<sql_types::Nullable<sql_types::Text>, Option<&'a str>>>;

type WithPlayerAndDemon<'a> = diesel::dsl::And<
    diesel::dsl::Eq<records::player, Bound<sql_types::Int4, i32>>,
    diesel::dsl::Eq<records::demon, Bound<CiText, &'a CiStr>>,
>;
type ByPlayerAndDemon<'a> = diesel::dsl::Filter<All<Record>, WithPlayerAndDemon<'a>>;

type WithExisting<'a> = diesel::dsl::Or<WithPlayerAndDemon<'a>, WithVideo<'a>>;
type ByExisting<'a> = diesel::dsl::Filter<All<Record>, WithExisting<'a>>;

type WithStatus = diesel::dsl::Eq<records::status_, Bound<Record_status, RecordStatus>>;

impl By<records::id, i32> for Record {}

impl Record {
    pub fn with_player_and_demon(player: i32, demon: &CiStr) -> WithPlayerAndDemon {
        records::player.eq(player).and(records::demon.eq(demon))
    }

    pub fn by_player_and_demon(player: i32, demon: &CiStr) -> ByPlayerAndDemon {
        Record::all().filter(Record::with_player_and_demon(player, demon))
    }

    pub fn get_existing<'a>(player: i32, demon: &'a CiStr, video: &'a str) -> ByExisting<'a> {
        Record::all()
            .filter(Record::with_player_and_demon(player, demon).or(records::video.eq(Some(video))))
    }

    pub fn with_status(status: RecordStatus) -> WithStatus {
        records::status_.eq(status)
    }

    pub fn insert(
        progress: i16,
        video: Option<&str>,
        status: RecordStatus,
        player: i32,
        submitter: i32,
        demon: &CiStr,
        conn: &PgConnection,
    ) -> QueryResult<i32> {
        let new = NewRecord {
            progress,
            video,
            status,
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

    pub fn validate_video(video: &mut String) -> Result<(), PointercrateError> {
        *video = crate::video::validate(video)?;

        Ok(())
    }
}

type WithPlayer = diesel::dsl::Eq<records::player, Bound<sql_types::Int4, i32>>;
type ByPlayer = diesel::dsl::Filter<All<EmbeddedRecordD>, WithPlayer>;

type ByPlayerAndStatus =
    diesel::dsl::Filter<All<EmbeddedRecordD>, diesel::dsl::And<WithPlayer, WithStatus>>;

impl EmbeddedRecordD {
    fn by_player(player_id: i32) -> ByPlayer {
        EmbeddedRecordD::all().filter(records::player.eq(player_id))
    }

    fn by_player_and_status(player_id: i32, status: RecordStatus) -> ByPlayerAndStatus {
        Self::by_player(player_id).filter(Record::with_status(status))
    }
}

type WithDemon<'a> = diesel::dsl::Eq<records::demon, Bound<CiText, &'a CiStr>>;
type ByDemon<'a> = diesel::dsl::Filter<All<EmbeddedRecordP>, WithDemon<'a>>;

type ByDemonAndStatus<'a> =
    diesel::dsl::Filter<All<EmbeddedRecordP>, diesel::dsl::And<WithDemon<'a>, WithStatus>>;

impl EmbeddedRecordP {
    fn by_demon(demon: &CiStr) -> ByDemon {
        EmbeddedRecordP::all().filter(records::demon.eq(demon))
    }

    fn by_demon_and_status(demon: &CiStr, status: RecordStatus) -> ByDemonAndStatus {
        Self::by_demon(demon).filter(Record::with_status(status))
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

impl Queryable<<<Record as Model>::Selection as Expression>::SqlType, Pg> for Record {
    type Row = (
        i32,
        i16,
        Option<String>,
        RecordStatus,
        i32,
        CiString,
        bool,
        i32,
        CiString,
        i16,
    );

    fn build(row: Self::Row) -> Self {
        Record {
            id: row.0,
            progress: row.1,
            video: row.2,
            status: row.3,
            player: EmbeddedPlayer {
                id: row.4,
                name: row.5,
                banned: row.6,
            },
            submitter: Some(row.7),
            demon: EmbeddedDemon {
                name: row.8,
                position: row.9,
            },
        }
    }
}

impl Model for EmbeddedRecordPD {
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
        records::status_,
        records::video,
        players::id,
        players::name,
        players::banned,
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

impl Queryable<<<EmbeddedRecordPD as Model>::Selection as Expression>::SqlType, Pg>
    for EmbeddedRecordPD
{
    type Row = (
        i32,
        i16,
        RecordStatus,
        Option<String>,
        i32,
        CiString,
        bool,
        CiString,
        i16,
    );

    fn build(row: Self::Row) -> Self {
        EmbeddedRecordPD {
            id: row.0,
            progress: row.1,
            status: row.2,
            video: row.3,
            player: EmbeddedPlayer {
                id: row.4,
                name: row.5,
                banned: row.6,
            },
            demon: EmbeddedDemon {
                name: row.7,
                position: row.8,
            },
        }
    }
}

impl Model for EmbeddedRecordD {
    #[allow(clippy::type_complexity)]
    type From = diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            records::table,
            demons::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::expression::operators::Eq<records::demon, demons::name>,
    >;
    type Selection = (
        records::id,
        records::progress,
        records::status_,
        records::video,
        demons::name,
        demons::position,
    );

    fn from() -> Self::From {
        diesel::query_source::joins::Join::new(
            records::table,
            demons::table,
            diesel::query_source::joins::Inner,
        )
        .on(records::demon.eq(demons::name))
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Queryable<<<EmbeddedRecordD as Model>::Selection as Expression>::SqlType, Pg>
    for EmbeddedRecordD
{
    type Row = (i32, i16, RecordStatus, Option<String>, CiString, i16);

    fn build(row: Self::Row) -> Self {
        EmbeddedRecordD {
            id: row.0,
            progress: row.1,
            status: row.2,
            video: row.3,
            demon: EmbeddedDemon {
                name: row.4,
                position: row.5,
            },
        }
    }
}

impl Model for EmbeddedRecordP {
    #[allow(clippy::type_complexity)]
    type From = diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            records::table,
            players::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::expression::operators::Eq<records::player, players::id>,
    >;
    type Selection = (
        records::id,
        records::progress,
        records::status_,
        records::video,
        players::id,
        players::name,
        players::banned,
    );

    fn from() -> Self::From {
        diesel::query_source::joins::Join::new(
            records::table,
            players::table,
            diesel::query_source::joins::Inner,
        )
        .on(records::player.eq(players::id))
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

impl Queryable<<<EmbeddedRecordP as Model>::Selection as Expression>::SqlType, Pg>
    for EmbeddedRecordP
{
    type Row = (i32, i16, RecordStatus, Option<String>, i32, CiString, bool);

    fn build(row: Self::Row) -> Self {
        EmbeddedRecordP {
            id: row.0,
            progress: row.1,
            status: row.2,
            video: row.3,
            player: EmbeddedPlayer {
                id: row.4,
                name: row.5,
                banned: row.6,
            },
        }
    }
}
