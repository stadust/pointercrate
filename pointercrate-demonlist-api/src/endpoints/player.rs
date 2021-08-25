use crate::LIST_HELPER;
use pointercrate_core::pool::PointercratePool;
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, TaggableExt, Tagged},
    pagination_response,
    query::Query,
    response::Response2,
};
use pointercrate_demonlist::player::{FullPlayer, PatchPlayer, Player, PlayerPagination, RankedPlayer, RankingPagination};
use pointercrate_user_api::auth::TokenAuth;
use rocket::{serde::json::Json, State};

#[rocket::get("/")]
pub async fn paginate(mut auth: TokenAuth, query: Query<PlayerPagination>) -> Result<Response2<Json<Vec<Player>>>> {
    let mut pagination = query.0;

    if !auth.has_permission(LIST_HELPER) {
        pagination.banned = Some(false);
    }

    let mut players = pagination.page(&mut auth.connection).await?;
    let (max_id, min_id) = Player::extremal_player_ids(&mut auth.connection).await?;

    pagination_response!(
        "/api/v1/players/",
        players,
        pagination,
        min_id,
        max_id,
        before_id,
        after_id,
        base.id
    )
}
#[rocket::get("/")]
pub async fn unauthed_paginate(pool: &State<PointercratePool>, query: Query<PlayerPagination>) -> Result<Response2<Json<Vec<Player>>>> {
    let mut pagination = query.0;
    let mut connection = pool.connection().await?;

    let mut players = pagination.page(&mut connection).await?;
    let (max_id, min_id) = Player::extremal_player_ids(&mut connection).await?;

    pagination_response!(
        "/api/v1/players/",
        players,
        pagination,
        min_id,
        max_id,
        before_id,
        after_id,
        base.id
    )
}

#[rocket::get("/ranking")]
pub async fn ranking(pool: &State<PointercratePool>, query: Query<RankingPagination>) -> Result<Response2<Json<Vec<RankedPlayer>>>> {
    let mut pagination = query.0;
    let mut connection = pool.connection().await?;

    let mut players = pagination.page(&mut connection).await?;
    let max_index = RankedPlayer::max_index(&mut connection).await?;

    pagination_response!(
        "/api/v1/players/ranking/",
        players,
        pagination,
        1,
        max_index,
        before_index,
        after_index,
        index
    )
}

#[rocket::get("/<player_id>")]
pub async fn get(player_id: i32, pool: &State<PointercratePool>) -> Result<Tagged<FullPlayer>> {
    let mut connection = pool.connection().await?;

    Ok(Tagged(
        Player::by_id(player_id, &mut connection).await?.upgrade(&mut connection).await?,
    ))
}

#[rocket::patch("/<player_id>", data = "<patch>")]
pub async fn patch(
    player_id: i32, mut auth: TokenAuth, precondition: Precondition, patch: Json<PatchPlayer>,
) -> Result<Tagged<FullPlayer>> {
    let player = Player::by_id(player_id, &mut auth.connection)
        .await?
        .upgrade(&mut auth.connection)
        .await?
        .require_match(precondition)?
        .apply_patch(patch.0, &mut auth.connection)
        .await?;

    auth.commit().await?;

    Ok(Tagged(player))
}
