use crate::{
    citext::{CiString, CiText},
    error::PointercrateError,
    model::{
        player::{players_with_score, RankedPlayer2, ShortPlayer},
        Model,
    },
    operation::{Paginate, Paginator},
    schema::players,
    Result,
};
use diesel::{
    dsl::sql, pg::Pg, query_builder::BoxedSelectStatement, PgConnection, QueryDsl, RunQueryDsl,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<i64>,

    name: Option<CiString>,
    banned: Option<bool>,

    nation: Option<String>,
}

impl Paginator for PlayerPagination {
    type Model = ShortPlayer;
    type PaginationColumn = players::id;
    type PaginationColumnType = i32;

    filter_method!(players[
        name = name,
        banned = banned,
        nationality = nation
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

    fn limit(&self) -> i64 {
        self.limit.unwrap_or(50)
    }

    fn before(&self) -> Option<i32> {
        self.before_id
    }

    fn after(&self) -> Option<i32> {
        self.after_id
    }
}

impl Paginate<PlayerPagination> for ShortPlayer {
    fn load(pagination: &PlayerPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        if pagination.limit() > 100 || pagination.limit() < 1 {
            return Err(PointercrateError::InvalidPaginationLimit)
        }

        let mut query = pagination.filter(ShortPlayer::boxed_all());

        filter!(query[
            players::id > pagination.after_id,
            players::id < pagination.before_id
        ]);

        pagination_result!(query, pagination, players::id, connection)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RankingPagination {
    #[serde(rename = "before")]
    before_id: Option<i64>,

    #[serde(rename = "after")]
    after_id: Option<i64>,

    nation: Option<String>,
    name_contains: Option<CiString>,
}

impl Paginator for RankingPagination {
    type Model = RankedPlayer2;
    type PaginationColumn = players_with_score::index;
    type PaginationColumnType = i64;

    fn filter<'a, ST>(
        &'a self,
        mut query: BoxedSelectStatement<'a, ST, <RankedPlayer2 as crate::model::Model>::From, Pg>,
    ) -> BoxedSelectStatement<'a, ST, <RankedPlayer2 as crate::model::Model>::From, Pg>
    {
        filter!(query[players_with_score::iso_country_code = self.nation]);

        if let Some(ref like_name) = self.name_contains {
            query = query.filter(
                sql("STRPOS(name, ")
                    .bind::<CiText, _>(like_name)
                    .sql(") > 0"),
            );
        }

        query
    }

    fn page(
        &self, last_on_page: Option<Self::PaginationColumnType>,
        first_on_page: Option<Self::PaginationColumnType>,
    ) -> Self {
        Self {
            before_id: last_on_page.map(|i| i + 1),
            after_id: first_on_page.map(|i| i - 1),
            ..self.clone()
        }
    }

    fn limit(&self) -> i64 {
        50
    }

    fn before(&self) -> Option<i64> {
        self.before_id
    }

    fn after(&self) -> Option<i64> {
        self.after_id
    }
}

impl Paginate<RankingPagination> for RankedPlayer2 {
    fn load(pagination: &RankingPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(RankedPlayer2::boxed_all());

        filter!(query[
            players_with_score::index > pagination.after_id,
            players_with_score::index < pagination.before_id
        ]);

        pagination_result!(query, pagination, players_with_score::index, connection)
    }
}
