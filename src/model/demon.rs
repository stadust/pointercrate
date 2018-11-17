use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    error::PointercrateError,
    model::{Get, Model},
    patch::{deserialize_patch, PatchField},
    schema::demons,
    Result,
};
use diesel::{expression::bound::Bound, result::Error, *};
use pointercrate_derive::Paginatable;
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;

/// Struct modelling a demon in the database
#[derive(Queryable, Insertable, Debug, Identifiable)]
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

#[derive(Serialize, Deserialize, Clone, Paginatable, Debug)]
#[database_table = "demons"]
#[column_type = "i16"]
#[result = "PartialDemon"]
#[allow(non_snake_case)]
pub struct DemonPagination {
    #[database_column = "position"]
    before: Option<i16>,

    #[database_column = "position"]
    after: Option<i16>,

    limit: Option<i32>,

    name: Option<String>,

    requirement: Option<i16>,
    requirement__lt: Option<i16>,
    requirement__gt: Option<i16>,
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

make_patch! {
    struct PatchDemon {
        name: String,
        position: i16,
        requirement: i16,
        verifier: i32,
        publisher: i32
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

impl Model for Demon {
    type Columns = AllColumns;
    type Table = demons::table;

    fn all() -> All {
        demons::table.select(ALL_COLUMNS)
    }
}

impl Model for PartialDemon {
    type Columns = (demons::name, demons::position);
    type Table = demons::table;

    fn all() -> diesel::dsl::Select<Self::Table, Self::Columns> {
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

impl Get<String> for Demon {
    fn get(name: String, connection: &PgConnection) -> Result<Self> {
        match Demon::by_name(&name).first(connection) {
            Ok(demon) => Ok(demon),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Demon",
                    identified_by: name,
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}
