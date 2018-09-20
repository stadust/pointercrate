use crate::schema::demons;
use diesel::{expression::bound::Bound, *};
use ipnetwork::IpNetwork;

#[derive(Queryable, Insertable, Debug, Identifiable)]
#[table_name = "demons"]
#[primary_key("name")]
pub struct Demon {
    name: String,
    position: i16,
    requirement: i16,
    // TODO: remove this fields
    description: Option<String>,
    // TODO: remove this field
    notes: Option<String>,
    verifier: i32,
    publisher: i32,
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
    pub fn all() -> All {
        demons::table.select(ALL_COLUMNS)
    }

    pub fn by_name(name: &str) -> ByName {
        Demon::all().filter(demons::name.eq(name))
    }

    pub fn by_position(position: i16) -> ByPosition {
        Demon::all().filter(demons::position.eq(position))
    }

    pub fn position(&self) -> i16 {
        self.position
    }

    pub fn requirement(&self) -> i16 {
        self.requirement
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
