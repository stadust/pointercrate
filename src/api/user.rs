//! Handlers for all endpoints under the `/api/v1/auth` prefix

use crate::{
    middleware::headers::{HttpRequestExt, HttpResponseBuilderExt},
    model::user::{AuthenticatedUser, PatchUser, User, UserPagination},
    permissions::Permissions,
    state::PointercrateState,
    Result,
};
use actix_web::{
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse,
};
use actix_web_codegen::{delete, get, patch};

#[get("/")]
pub async fn paginate(request: HttpRequest, state: Data<PointercrateState>, mut pagination: Query<UserPagination>) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;

    user.inner().require_permissions(Permissions::Administrator)?;

    let users = pagination.page(&mut connection).await?;

    let (max_id, min_id) = User::extremal_member_ids(&mut connection).await?;

    pagination_response!(users, pagination, min_id, max_id, before_id, after_id)
}

#[get("/{user_id}/")]
pub async fn get(request: HttpRequest, state: Data<PointercrateState>, user_id: Path<i32>) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;

    user.inner().require_permissions(Permissions::Moderator)?;

    let gotten_user = User::by_id(user_id.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(gotten_user))
}

#[patch("/{user_id}/")]
pub async fn patch(
    request: HttpRequest, state: Data<PointercrateState>, user_id: Path<i32>, data: Json<PatchUser>,
) -> Result<HttpResponse> {
    let mut connection = state.transaction().await?;
    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;

    if data.permissions.is_some() {
        user.inner().require_permissions(Permissions::Administrator)?;
    } else {
        user.inner().require_permissions(Permissions::Moderator)?;
    }

    // FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
    let gotten_user = User::by_id(user_id.into_inner(), &mut connection).await?;

    request.validate_etag(&gotten_user)?;

    let gotten_user = gotten_user.apply_patch(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(gotten_user))
}

#[delete("/{user_id}/")]
pub async fn delete(request: HttpRequest, state: Data<PointercrateState>, user_id: Path<i32>) -> Result<HttpResponse> {
    let mut connection = state.transaction().await?;
    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;

    user.inner().require_permissions(Permissions::Administrator)?;

    // FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
    let to_delete = User::by_id(user_id.into_inner(), &mut connection).await?;

    request.validate_etag(&to_delete)?;

    to_delete.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
