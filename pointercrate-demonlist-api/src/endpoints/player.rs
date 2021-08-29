use pointercrate_core::pool::PointercratePool;
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, TaggableExt, Tagged},
    pagination_response,
    query::Query,
    response::Response2,
};
use pointercrate_demonlist::{
    player::{
        claim::{ListedClaim, PatchVerified, PlayerClaim, PlayerClaimPagination},
        DatabasePlayer, FullPlayer, PatchPlayer, Player, PlayerPagination, RankedPlayer, RankingPagination,
    },
    LIST_HELPER,
};
use pointercrate_user::MODERATOR;
use pointercrate_user_api::auth::TokenAuth;
use rocket::{http::Status, serde::json::Json, State};

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
#[rocket::get("/", rank = 1)]
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

#[rocket::put("/<player_id>/claims")]
pub async fn put_claim(player_id: i32, mut auth: TokenAuth) -> Result<Response2<Json<PlayerClaim>>> {
    let player = DatabasePlayer::by_id(player_id, &mut auth.connection).await?;
    let claim = player.initiate_claim(auth.user.inner().id, &mut auth.connection).await?;

    Ok(Response2::json(claim).status(Status::Created).with_header(
        "Location",
        format!("/api/v1/players/{}/claims/{}/", player.id, auth.user.inner().id),
    ))
}

#[rocket::patch("/<player_id>/claims/<user_id>", data = "<data>")]
pub async fn patch_claim(player_id: i32, user_id: i32, mut auth: TokenAuth, data: Json<PatchVerified>) -> Result<Json<PlayerClaim>> {
    auth.require_permission(MODERATOR)?;

    let claim = PlayerClaim::get(user_id, player_id, &mut auth.connection).await?;
    let claim = claim.set_verified(data.verified, &mut auth.connection).await?;

    Ok(Json(claim))
}

#[rocket::get("/claims")]
pub async fn paginate_claims(mut auth: TokenAuth, pagination: Query<PlayerClaimPagination>) -> Result<Response2<Json<Vec<ListedClaim>>>> {
    auth.require_permission(MODERATOR)?;

    let mut pagination = pagination.0;

    let mut claims = pagination.page(&mut auth.connection).await?;
    let (max_id, min_id) = ListedClaim::extremal_ids(&mut auth.connection).await?;

    pagination_response!(
        "/api/v1/players/claims/",
        claims,
        pagination,
        min_id,
        max_id,
        before_id,
        after_id,
        id
    )
}
