use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    schema::demons,
};
use diesel::{expression::bound::Bound, sql_types, ExpressionMethods, QueryDsl};
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_derive::Serialize;
use std::fmt::Display;

mod get;
mod paginate;
mod patch;

pub use self::paginate::DemonPagination;

/// Struct modelling a demon in the database
#[derive(Queryable, Insertable, Debug, Identifiable, Serialize)]
#[table_name = "demons"]
#[primary_key("name")]
pub struct Demon {
    /// The [`Demon`]'s Geometry Dash level name
    pub name: String,

    /// The [`Demon`]'s position on the demonlist
    ///
    /// Positions for consecutive demons are always consecutive positive integers
    pub position: i16,

    /// The minimal progress a [`Player`] must achieve on this [`Demon`] to have their record
    /// accepted
    pub requirement: i16,

    // TODO: remove this fields
    description: Option<String>,
    // TODO: remove this field
    notes: Option<String>,

    /// The player-ID of this [`Demon`]'s verifer
    pub verifier: i32,

    /// The player-ID of this [`Demon`]'s publisher
    pub publisher: i32,
}

/// Struct modelling a minimal representation of a [`Demon`] in the database
///
/// These representations are used whenever a different object references a demon, or when a list of
/// demons is requested
#[derive(Debug, Queryable, Identifiable, Hash, Eq, PartialEq, Associations)]
#[table_name = "demons"]
#[primary_key("name")]
pub struct PartialDemon {
    pub name: String,
    pub position: i16,
}
impl Serialize for PartialDemon {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("position", &self.position)?;
        map.serialize_entry("state", &ListState::from(self.position).to_string())?;
        map.end()
    }
}

/// Enum encoding the 3 different parts of the demonlist
#[derive(Debug)]
pub enum ListState {
    /// The main part of the demonlist, ranging from position 1 onwards to [`LIST_SIZE`]
    /// (inclusive)
    Main,

    /// The extended part of the demonlist, ranging from [`LIST_SIZE`] (exclusive) onwards to
    /// [`EXTENDED_LIST_SIZE`] (inclusive)
    Extended,

    /// The legacy part of the demonlist, starting at [`EXTENDED_LIST_SIZE`] (exclusive) and being
    /// theoretically unbounded
    Legacy,
}

impl From<i16> for ListState {
    /// Calculates the [`ListState`] of [`Demon`] based on its [`Demon::position`]
    fn from(position: i16) -> ListState {
        if position <= *LIST_SIZE {
            ListState::Main
        } else if position <= *EXTENDED_LIST_SIZE {
            ListState::Extended
        } else {
            ListState::Legacy
        }
    }
}

impl Display for ListState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ListState::Main => write!(f, "MAIN"),
            ListState::Extended => write!(f, "EXTENDED"),
            ListState::Legacy => write!(f, "LEGACY"),
        }
    }
}

type AllColumns = (
    demons::name,
    demons::position,
    demons::requirement,
    demons::description,
    demons::notes,
    demons::verifier,
    demons::publisher,
);

const ALL_COLUMNS: AllColumns = (
    demons::name,
    demons::position,
    demons::requirement,
    demons::description,
    demons::notes,
    demons::verifier,
    demons::publisher,
);

type All = diesel::dsl::Select<demons::table, AllColumns>;

type WithName<'a> = diesel::dsl::Eq<demons::name, Bound<sql_types::Text, &'a str>>;
type ByName<'a> = diesel::dsl::Filter<All, WithName<'a>>;

type WithPosition = diesel::dsl::Eq<demons::position, Bound<sql_types::Int2, i16>>;
type ByPosition = diesel::dsl::Filter<All, WithPosition>;

impl Demon {
    fn all() -> All {
        demons::table.select(ALL_COLUMNS)
    }

    /// Constructs a diesel query returning all columns of demons whose name matches the given
    /// string
    pub fn by_name(name: &str) -> ByName {
        Demon::all().filter(demons::name.eq(name))
    }

    /// Constructs a diesel query returning all columns of position whose name matches the given i16
    pub fn by_position(position: i16) -> ByPosition {
        Demon::all().filter(demons::position.eq(position))
    }
}

impl PartialDemon {
    fn all() -> diesel::dsl::Select<demons::table, (demons::name, demons::position)> {
        demons::table.select((demons::name, demons::position))
    }
}

impl Into<PartialDemon> for Demon {
    fn into(self) -> PartialDemon {
        PartialDemon {
            name: self.name,
            position: self.position,
        }
    }
}
