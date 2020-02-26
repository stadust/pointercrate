use crate::{
    extractor::{auth::TokenAuth, if_match::IfMatch},
    model::demonlist::{
        player::{FullPlayer, PatchPlayer, Player, PlayerPagination, RankedPlayer, RankingPagination},
        record::FullRecord,
    },
    permissions::Permissions,
    state::PointercrateState,
    util::HttpResponseBuilderExt,
    ApiResult,
};
use actix_web::{
    web::{Json, Path},
    HttpResponse,
};
use actix_web_codegen::{get, patch};

#[get("/")]
pub async fn paginate(
    TokenAuth(user): TokenAuth, state: PointercrateState, mut pagination: Json<PlayerPagination>,
) -> ApiResult<HttpResponse> {
    user.inner().require_permissions(Permissions::ExtendedAccess)?;
    let mut connection = state.connection().await?;

    let demons = pagination.page(&mut connection).await?;
    let (min_id, max_id) = Player::extremal_player_ids(&mut connection).await?;

    pagination_response!(demons, pagination, min_id, max_id, before_id, after_id, base.id)
}

#[get("/ranking/")]
pub async fn ranking(state: PointercrateState, mut pagination: Json<RankingPagination>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    let demons = pagination.page(&mut connection).await?;
    let max_index = RankedPlayer::max_index(&mut connection).await?;

    pagination_response!(demons, pagination, 1, max_index, before_index, after_index, index)
}

#[get("/{player_id}/")]
pub async fn get(state: PointercrateState, path: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    let player = Player::by_id(path.into_inner(), &mut connection)
        .await?
        .upgrade(&mut connection)
        .await?;

    Ok(HttpResponse::Ok().json_with_etag(player))
}

#[patch("/{player_id}/")]
pub async fn patch(
    TokenAuth(user): TokenAuth, if_match: IfMatch, state: PointercrateState, data: Json<PatchPlayer>, path: Path<i32>,
) -> ApiResult<HttpResponse> {
    user.inner().require_permissions(Permissions::ListModerator)?;

    let mut connection = state.audited_transaction(&user).await?;

    let player = Player::by_id(path.into_inner(), &mut connection)
        .await?
        .upgrade(&mut connection)
        .await?;

    if_match.require_etag_match(&player)?;

    let player = player.apply_patch(data.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(player))
}
