//! Handlers for all endpoints under the `/api/v1/auth` prefix

use crate::{
    extractor::{auth::TokenAuth, if_match::IfMatch},
    model::user::{PatchUser, User, UserPagination},
    permissions::Permissions,
    state::PointercrateState,
    util::HttpResponseBuilderExt,
    ApiResult,
};
use actix_web::{
    web::{Json, Path, Query},
    HttpResponse,
};
use actix_web_codegen::{delete, get, patch};

#[get("/")]
pub async fn paginate(user: TokenAuth, state: PointercrateState, mut pagination: Query<UserPagination>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    user.0.inner().require_permissions(Permissions::Administrator)?;

    let mut users = pagination.page(&mut connection).await?;

    let (max_id, min_id) = User::extremal_member_ids(&mut connection).await?;

    pagination_response!("/api/v1/users/", users, pagination, min_id, max_id, before_id, after_id, id)
}

#[get("/{user_id}/")]
pub async fn get(user: TokenAuth, state: PointercrateState, user_id: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    user.0.inner().require_permissions(Permissions::Moderator)?;

    let gotten_user = User::by_id(user_id.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(&gotten_user))
}

#[patch("/{user_id}/")]
pub async fn patch(
    if_match: IfMatch, user: TokenAuth, state: PointercrateState, user_id: Path<i32>, data: Json<PatchUser>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user.0).await?;

    if data.permissions.is_some() {
        user.0.inner().require_permissions(Permissions::Administrator)?;
    } else {
        user.0.inner().require_permissions(Permissions::Moderator)?;
    }

    // FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
    let gotten_user = User::by_id(user_id.into_inner(), &mut connection).await?;

    if_match.require_etag_match(&gotten_user)?;

    let gotten_user = gotten_user.apply_patch(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(&gotten_user))
}

#[delete("/{user_id}/")]
pub async fn delete(if_match: IfMatch, user: TokenAuth, state: PointercrateState, user_id: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user.0).await?;

    user.0.inner().require_permissions(Permissions::Administrator)?;

    // FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
    let to_delete = User::by_id(user_id.into_inner(), &mut connection).await?;

    if_match.require_etag_match(&to_delete)?;

    to_delete.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
