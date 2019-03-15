use crate::{
    citext::CiString,
    error::PointercrateError,
    model::{
        player::{players_with_score, RankedPlayer2, ShortPlayer},
        Model,
    },
    operation::{Paginate, Paginator},
    schema::players,
    Result,
};
use diesel::{pg::Pg, query_builder::BoxedSelectStatement, PgConnection, QueryDsl, RunQueryDsl};
use serde_derive::{Deserialize, Serialize};
use diesel::dsl::sql;
use crate::citext::CiText;

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

    filter_method!(players_with_score[iso_country_code = nation]);

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

    fn first(&self, _: &PgConnection) -> Result<Option<Self>> {
        Ok(Some(self.page(None, Some(1))))
    }

    fn next(&self, _: &PgConnection) -> Result<Option<Self>> {
        if self.before_id.is_some() {
            Ok(Some(self.page(None, self.before_id)))
        } else if let Some(after) = self.after_id {
            Ok(Some(self.page(None, Some(after + 51))))
        } else {
            Ok(Some(self.page(None, Some(51))))
        }
    }

    fn prev(&self, _: &PgConnection) -> Result<Option<Self>> {
        if self.after_id.is_some() {
            Ok(Some(self.page(self.after_id, None)))
        } else if let Some(before) = self.before_id {
            Ok(Some(self.page(Some(before - 51), None)))
        } else {
            Ok(None)
        }
    }

    // We can probably also do a more efficient implementation of .last() by doing some sort of weird COUNT , but ehhh, not needed yet
}

impl Paginate<RankingPagination> for RankedPlayer2 {
    fn load(pagination: &RankingPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(RankedPlayer2::boxed_all());

        filter!(query[
            players_with_score::index > pagination.after_id,
            players_with_score::index < pagination.before_id
        ]);

        if let Some(ref like_name) = pagination.name_contains {
            query = query.filter(sql("STRPOS(name, ").bind::<CiText,_>(like_name).sql(") > 0"));
        }

        if pagination.after_id.is_none() && pagination.before_id.is_some() {
            query = query.filter(players_with_score::index.ge(pagination.before_id.unwrap() - 50))
        }

        query
            .limit(50)
            .load(connection)
            .map_err(PointercrateError::database)
    }
}
