use crate::{
    citext::{CiString, CiText},
    context::RequestContext,
    error::PointercrateError,
    model::{
        demonlist::player::{players_with_score, RankedPlayer, ShortPlayer},
        Model,
    },
    operation::{Paginate, Paginator, PaginatorQuery, TablePaginator},
    schema::players,
    Result,
};
use diesel::{dsl::sql, ExpressionMethods, QueryDsl, RunQueryDsl};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<u8>,

    name: Option<CiString>,
    banned: Option<bool>,

    nation: Option<String>,
}

impl TablePaginator for PlayerPagination {
    type ColumnType = i32;
    type PaginationColumn = players::id;
    type Table = players::table;

    fn query(&self, _: RequestContext) -> PaginatorQuery<players::table> {
        let mut query = ShortPlayer::boxed_all();

        filter!(query[
            players::name = self.name,
            players::banned = self.banned,
            players::nationality = self.nation
        ]);

        // FIXME: once players have their views
        //query
        unimplemented!()
    }
}

delegate_to_table_paginator!(PlayerPagination);

impl Paginate<PlayerPagination> for ShortPlayer {
    fn load(pagination: &PlayerPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        ctx.check_permissions(
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
        )?;

        let mut query = pagination.query(ctx);

        /*filter!(query[
            players::id > pagination.after_id,
            players::id < pagination.before_id
        ]);

        pagination_result!(query, pagination, players::id, ctx.connection())*/
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RankingPagination {
    #[serde(rename = "before")]
    before_id: Option<i64>,

    #[serde(rename = "after")]
    after_id: Option<i64>,

    // FIXME: ???
    limit: Option<u8>,

    nation: Option<String>,
    name_contains: Option<CiString>,
}

impl TablePaginator for RankingPagination {
    type ColumnType = i64;
    type PaginationColumn = players_with_score::index;
    type Table = players_with_score::table;

    fn query(&self, _: RequestContext) -> PaginatorQuery<players_with_score::table> {
        let mut query = RankedPlayer::boxed_all();

        if let Some(ref nation) = self.nation {
            query = query.filter(players_with_score::iso_country_code.eq(nation));
        }

        if let Some(ref like_name) = self.name_contains {
            query = query.filter(
                sql("STRPOS(name, ")
                    .bind::<CiText, _>(like_name)
                    .sql(") > 0"),
            );
        }

        query
    }
}

delegate_to_table_paginator!(RankingPagination);

impl Paginate<RankingPagination> for RankedPlayer {
    fn load(pagination: &RankingPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        let mut query = pagination.query(ctx);

        filter!(query[
            players_with_score::index > pagination.after_id,
            players_with_score::index < pagination.before_id
        ]);

        pagination_result!(
            query,
            pagination,
            players_with_score::index,
            ctx.connection()
        )
    }
}
