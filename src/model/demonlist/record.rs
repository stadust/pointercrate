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
    deserialize::Queryable, expression::bound::Bound, pg::Pg, query_dsl::QueryDsl, sql_types,
    BoolExpressionMethods, Expression, ExpressionMethods, Table,
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

#[derive(Debug, Queryable, Display, Hash)]
#[display(fmt = "{} {}% on {} (ID: {})", player_id, progress, demon_name, id)]
pub struct DatabaseRecord {
    id: i32,
    progress: i16,
    video: Option<String>,
    pub(crate) status: RecordStatus,
    player_id: i32,
    submitter_id: i32,
    demon_name: CiString,
}

impl Model for DatabaseRecord {
    type From = records::table;
    type Selection = <records::table as Table>::AllColumns;

    fn from() -> Self::From {
        records::table
    }

    fn selection() -> Self::Selection {
        records::all_columns
    }
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;
    use crate::model::demonlist::record::Record_status;

    records_pds (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        player_id -> Int4,
        player_name -> CiText,
        player_banned -> Bool,
        demon_name -> CiText,
        position -> Int2,
        submitter_id -> Int4,
        submitter_banned -> Bool,
    }
}

#[derive(Debug, Serialize, Hash, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct FullRecord {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: EmbeddedPlayer,
    pub demon: EmbeddedDemon,
    pub submitter: Option<(i32, bool)>,
}

table! {
    use crate::citext::CiText;
    use diesel::sql_types::*;
    use crate::model::demonlist::record::Record_status;

    records_pd (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        submitter_id -> Int4,
        player_id -> Int4,
        player_name -> CiText,
        player_banned -> Bool,
        demon_name -> CiText,
        position -> Int2,
    }
}

#[derive(Debug, Serialize, Hash, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct Record {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: EmbeddedPlayer,
    pub demon: EmbeddedDemon,
    pub submitter: Option<i32>,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{}% on {} (ID: {})", progress, demon, id)]
pub struct MinimalRecordD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: EmbeddedDemon,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} - {}% (ID: {})", player, progress, id)]
pub struct MinimalRecordP {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: EmbeddedPlayer,
}

impl Model for FullRecord {
    type From = records_pds::table;
    type Selection = <records_pds::table as Table>::AllColumns;

    fn from() -> Self::From {
        records_pds::table
    }

    fn selection() -> Self::Selection {
        records_pds::all_columns
    }
}

impl Queryable<<<FullRecord as Model>::Selection as Expression>::SqlType, Pg> for FullRecord {
    type Row = (
        i32,
        i16,
        Option<String>,
        RecordStatus,
        i32,
        CiString,
        bool,
        CiString,
        i16,
        i32,
        bool,
    );

    fn build(row: Self::Row) -> Self {
        FullRecord {
            id: row.0,
            progress: row.1,
            video: row.2,
            status: row.3,
            player: EmbeddedPlayer {
                id: row.4,
                name: row.5,
                banned: row.6,
            },
            demon: EmbeddedDemon {
                position: row.8,
                name: row.7,
            },
            submitter: Some((row.9, row.10)),
        }
    }
}

impl Model for Record {
    type From = records_pd::table;
    type Selection = <records_pd::table as Table>::AllColumns;

    fn from() -> Self::From {
        records_pd::table
    }

    fn selection() -> Self::Selection {
        records_pd::all_columns
    }
}

impl By<records_pd::id, i32> for Record {}

impl Queryable<<<Record as Model>::Selection as Expression>::SqlType, Pg> for Record {
    type Row = (
        // record
        i32,
        i16,
        Option<String>,
        RecordStatus,
        // submitter
        i32,
        // player
        i32,
        CiString,
        bool,
        // demon
        CiString,
        i16,
    );

    fn build(row: Self::Row) -> Self {
        Record {
            id: row.0,
            progress: row.1,
            video: row.2,
            status: row.3,
            submitter: Some(row.4),
            player: EmbeddedPlayer {
                id: row.5,
                name: row.6,
                banned: row.7,
            },
            demon: EmbeddedDemon {
                name: row.8,
                position: row.9,
            },
        }
    }
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

type WithVideo<'a> =
    diesel::dsl::Eq<records::video, Bound<sql_types::Nullable<sql_types::Text>, Option<&'a str>>>;

type WithPlayerAndDemon<'a> = diesel::dsl::And<
    diesel::dsl::Eq<records::player, Bound<sql_types::Int4, i32>>,
    diesel::dsl::Eq<records::demon, Bound<CiText, &'a CiStr>>,
>;
type ByPlayerAndDemon<'a> = diesel::dsl::Filter<All<DatabaseRecord>, WithPlayerAndDemon<'a>>;

type WithExisting<'a> = diesel::dsl::Or<WithPlayerAndDemon<'a>, WithVideo<'a>>;
type ByExisting<'a> = diesel::dsl::Filter<All<DatabaseRecord>, WithExisting<'a>>;

type WithStatus = diesel::dsl::Eq<records::status_, Bound<Record_status, RecordStatus>>;

impl DatabaseRecord {
    pub fn with_player_and_demon(player: i32, demon: &CiStr) -> WithPlayerAndDemon {
        records::player.eq(player).and(records::demon.eq(demon))
    }

    pub fn by_player_and_demon(player: i32, demon: &CiStr) -> ByPlayerAndDemon {
        DatabaseRecord::all().filter(DatabaseRecord::with_player_and_demon(player, demon))
    }

    pub fn get_existing<'a>(player: i32, demon: &'a CiStr, video: &'a str) -> ByExisting<'a> {
        DatabaseRecord::all().filter(
            DatabaseRecord::with_player_and_demon(player, demon).or(records::video.eq(Some(video))),
        )
    }

    pub fn with_status(status: RecordStatus) -> WithStatus {
        records::status_.eq(status)
    }
}

impl Record {
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
type ByPlayer = diesel::dsl::Filter<All<MinimalRecordD>, WithPlayer>;

type ByPlayerAndStatus =
    diesel::dsl::Filter<All<MinimalRecordD>, diesel::dsl::And<WithPlayer, WithStatus>>;

impl MinimalRecordD {
    fn by_player(player_id: i32) -> ByPlayer {
        MinimalRecordD::all().filter(records::player.eq(player_id))
    }

    fn by_player_and_status(player_id: i32, status: RecordStatus) -> ByPlayerAndStatus {
        Self::by_player(player_id).filter(DatabaseRecord::with_status(status))
    }
}

type WithDemon<'a> = diesel::dsl::Eq<records::demon, Bound<CiText, &'a CiStr>>;
type ByDemon<'a> = diesel::dsl::Filter<All<MinimalRecordP>, WithDemon<'a>>;

type ByDemonAndStatus<'a> =
    diesel::dsl::Filter<All<MinimalRecordP>, diesel::dsl::And<WithDemon<'a>, WithStatus>>;

impl MinimalRecordP {
    fn by_demon(demon: &CiStr) -> ByDemon {
        MinimalRecordP::all().filter(records::demon.eq(demon))
    }

    fn by_demon_and_status(demon: &CiStr, status: RecordStatus) -> ByDemonAndStatus {
        Self::by_demon(demon).filter(DatabaseRecord::with_status(status))
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

impl Model for MinimalRecordD {
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

impl Queryable<<<MinimalRecordD as Model>::Selection as Expression>::SqlType, Pg>
    for MinimalRecordD
{
    type Row = (i32, i16, RecordStatus, Option<String>, CiString, i16);

    fn build(row: Self::Row) -> Self {
        MinimalRecordD {
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

impl Model for MinimalRecordP {
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

impl Queryable<<<MinimalRecordP as Model>::Selection as Expression>::SqlType, Pg>
    for MinimalRecordP
{
    type Row = (i32, i16, RecordStatus, Option<String>, i32, CiString, bool);

    fn build(row: Self::Row) -> Self {
        MinimalRecordP {
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
