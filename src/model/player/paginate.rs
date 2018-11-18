use super::Player;
use crate::{
    error::PointercrateError,
    operation::{Paginate, Paginator},
    schema::players,
    Result,
};
use diesel::{pg::Pg, query_builder::BoxedSelectStatement, PgConnection, QueryDsl, RunQueryDsl};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<i64>,

    name: Option<String>,
    banned: Option<bool>,
}

impl PlayerPagination {
    filter_method!(players[
        name = name,
        banned = banned
    ]);
}

impl Paginator for PlayerPagination {
    navigation!(players, id, before_id, after_id);
}

impl Paginate<PlayerPagination> for Player {
    fn load(&self, pagination: PlayerPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(Player::all().into_boxed());

        filter!(query[
            players::id > pagination.after_id,
            players::id < pagination.before_id
        ]);

        query
            .limit(pagination.limit.unwrap_or(50))
            .load(connection)
            .map_err(PointercrateError::database)
    }
}
