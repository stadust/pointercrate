use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    model::Model,
    patch::{deserialize_patch, Patch, PatchField, Patchable, UpdateDatabase},
    schema::demons,
};
use diesel::{expression::bound::Bound, *};
use pointercrate_derive::Paginatable;
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Queryable, Insertable, Debug, Identifiable)]
#[table_name = "demons"]
#[primary_key("name")]
pub struct Demon {
    pub name: String,
    pub position: i16,
    pub requirement: i16,
    // TODO: remove this fields
    description: Option<String>,
    // TODO: remove this field
    notes: Option<String>,
    pub verifier: i32,
    pub publisher: i32,
}

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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("position", &self.position)?;
        map.serialize_entry("state", &list_state(self.position).to_string())?;
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

fn list_state(position: i16) -> ListState {
    if position <= *LIST_SIZE {
        ListState::Main
    } else if position <= *EXTENDED_LIST_SIZE {
        ListState::Extended
    } else {
        ListState::Legacy
    }
}

#[derive(Debug)]
pub enum ListState {
    Main,
    Extended,
    Legacy,
}

impl Display for ListState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
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
    pub fn by_name(name: &str) -> ByName {
        Demon::all().filter(demons::name.eq(name))
    }

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
