use crate::schema::players;
use diesel::{expression::bound::Bound, *};
use serde_derive::Serialize;

#[derive(Queryable, Debug, Identifiable, Hash, Eq, PartialEq, Serialize)]
#[table_name = "players"]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub banned: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "players"]
struct NewPlayer<'a> {
    name: &'a str,
}

type AllColumns = (players::id, players::name, players::banned);

const ALL_COLUMNS: AllColumns = (players::id, players::name, players::banned);

type All = diesel::dsl::Select<players::table, AllColumns>;

type WithName<'a> = diesel::dsl::Eq<players::name, Bound<sql_types::Text, &'a str>>;
type ByName<'a> = diesel::dsl::Filter<All, WithName<'a>>;

type WithId = diesel::dsl::Eq<players::id, Bound<sql_types::Int4, i32>>;
type ById = diesel::dsl::Filter<All, WithId>;

impl Player {
    pub fn all() -> All {
        players::table.select(ALL_COLUMNS)
    }

    pub fn by_name(name: &str) -> ByName {
        Player::all().filter(players::name.eq(name))
    }

    pub fn by_id(id: i32) -> ById {
        Player::all().filter(players::id.eq(id))
    }

    pub fn insert(conn: &PgConnection, name: &str) -> QueryResult<Player> {
        insert_into(players::table)
            .values(&NewPlayer { name })
            .get_result(conn)
    }
}
