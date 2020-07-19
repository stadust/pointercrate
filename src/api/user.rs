//! Handlers for all endpoints under the `/api/v1/auth` prefix

use crate::{
    error::{JsonError, PointercrateError},
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
pub async fn paginate(
    TokenAuth(user): TokenAuth, state: PointercrateState, mut pagination: Query<UserPagination>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    // Rule of thumb: If you can assign permissions, you can see all users that currently have those
    // permissions
    if user.inner().permissions.assigns().is_empty() {
        return Err(JsonError(PointercrateError::Forbidden))
    }

    if !user.inner().has_permission(Permissions::Moderator) {
        // Pointercrate staff need to be able to see all users, not only those whose permissions they can
        // assign
        pagination.any_permissions = match pagination.any_permissions {
            Some(perms) => Some(perms | user.inner().permissions.assigns()),
            None => Some(user.inner().permissions.assigns()),
        };
    }

    let mut users = pagination.page(&mut connection).await?;

    let (max_id, min_id) = User::extremal_member_ids(&mut connection).await?;

    pagination_response!("/api/v1/users/", users, pagination, min_id, max_id, before_id, after_id, id)
}

#[get("/{user_id}/")]
pub async fn get(TokenAuth(user): TokenAuth, state: PointercrateState, user_id: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    let gotten_user = User::by_id(user_id.into_inner(), &mut connection).await?;

    // We are only allowed to retrieve users who already have permissions we can set.
    // We're also using that ListModerator implies ListHelper
    if !user.inner().has_permission(Permissions::Administrator)
        && !(user.inner().has_permission(Permissions::ListAdministrator) && gotten_user.has_permission(Permissions::ListHelper))
    {
        return Err(JsonError(PointercrateError::ModelNotFound {
            model: "User",
            identified_by: gotten_user.id.to_string(),
        })) // don't leak internal information about database
    }

    Ok(HttpResponse::Ok().json_with_etag(&gotten_user))
}

#[patch("/{user_id}/")]
pub async fn patch(
    if_match: IfMatch, user: TokenAuth, state: PointercrateState, user_id: Path<i32>, data: Json<PatchUser>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user.0).await?;

    if data.display_name.is_some() || data.youtube_channel.is_some() {
        user.0.inner().require_permissions(Permissions::Moderator)?;
    }

    // FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
    let gotten_user = User::by_id(user_id.into_inner(), &mut connection).await?;

    // We probably don't have to check if we are even allowed to retrieve this user, since we require a
    // correct ETag, which means we previously retrieved this user successfully and passed the
    // permissions check at GET. However, on might guess the ETag. Or use an ETag value they got from
    // before they were demoted.
    if !user.0.inner().has_permission(Permissions::Administrator)
        && !(user.0.inner().has_permission(Permissions::ListAdministrator) && gotten_user.has_permission(Permissions::ListHelper))
    {
        return Err(JsonError(PointercrateError::ModelNotFound {
            model: "User",
            identified_by: gotten_user.id.to_string(),
        }))
    }

    if let Some(assign) = data.permissions {
        if !user.0.inner().has_permission(Permissions::Administrator) {
            // XOR here gets us the set of permissions that _changed_ which is what we really care about!
            user.0
                .inner()
                .require_permissions((assign ^ gotten_user.permissions).required_for_assignment())?;
        }
    }

    if_match.require_etag_match(&gotten_user)?;

    let gotten_user = gotten_user.apply_patch(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(&gotten_user))
}

#[delete("/{user_id}/")]
pub async fn delete(if_match: IfMatch, user: TokenAuth, state: PointercrateState, user_id: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user.0).await?;

    let to_delete = user_id.into_inner();

    if user.0.inner().id == to_delete {
        return Err(PointercrateError::DeleteSelf.into())
    }

    user.0.inner().require_permissions(Permissions::Administrator)?;

    // FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
    let to_delete = User::by_id(to_delete, &mut connection).await?;

    if_match.require_etag_match(&to_delete)?;

    to_delete.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
