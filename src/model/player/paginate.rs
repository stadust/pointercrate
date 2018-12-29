use super::Player;
use crate::{
    error::PointercrateError,
    model::Model,
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

impl Paginator for PlayerPagination {
    type Model = Player;
    type PaginationColumn = players::id;
    type PaginationColumnType = i32;

    navigation!(players, id, before_id, after_id);

    filter_method!(players[
        name = name,
        banned = banned
    ]);

    fn page(
        &self, last_on_page: Option<Self::PaginationColumnType>,
        first_on_page: Option<Self::PaginationColumnType>,
    ) -> Self {
        PlayerPagination {
            before_id: last_on_page.map(|i| i + 1),
            after_id: first_on_page.map(|i| i - 1),
            ..self.clone()
        }
    }
}

impl Paginate<PlayerPagination> for Player {
    fn load(pagination: &PlayerPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(Player::boxed_all());

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
