pub use self::post::PostCreator;
use crate::{
    model::demonlist::player::DatabasePlayer,
    schema::{creators, demons, players},
};
use derive_more::Display;
use diesel::{expression::bound::Bound, sql_types, ExpressionMethods, QueryDsl, Queryable};
use serde_derive::Serialize;

mod delete;
mod get;
mod post;

#[derive(Debug, Serialize)]
pub struct Creators(pub Vec<DatabasePlayer>);

#[derive(Debug, Queryable, Display, Hash)]
#[display(fmt = "creator with id {} on demon {}", creator, demon)]
pub struct Creator {
    demon: i32,
    creator: i32,
}

type ByDemon = diesel::dsl::Eq<creators::demon, Bound<sql_types::Int4, i32>>;
type WithDemon = diesel::dsl::Filter<
    diesel::dsl::Select<
        diesel::dsl::InnerJoin<creators::table, players::table>,
        (players::id, players::name, players::banned),
    >,
    ByDemon,
>;

type ByPlayer = diesel::dsl::Eq<creators::creator, Bound<sql_types::Int4, i32>>;
type WithPlayer = diesel::dsl::Filter<
    diesel::dsl::Select<
        diesel::dsl::InnerJoin<creators::table, demons::table>,
        (demons::id, demons::position, demons::name),
    >,
    ByPlayer,
>;

pub fn created_by(player: i32) -> WithPlayer {
    creators::table
        .inner_join(demons::table)
        .select((demons::id, demons::position, demons::name))
        .filter(creators::creator.eq(player))
}

pub fn creators_of(demon: i32) -> WithDemon {
    creators::table
        .inner_join(players::table)
        .select((players::id, players::name, players::banned))
        .filter(creators::demon.eq(demon))
}
