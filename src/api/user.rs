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

    let min_id = User::min_member_id(&mut connection).await?;
    let max_id = User::max_member_id(&mut connection).await?;

    pagination.after_id = Some(min_id - 1);
    pagination.before_id = None;

    let mut rel = format!(
        "</api/v1/users/?{}>; rel=first",
        serde_urlencoded::to_string(&pagination.0).unwrap()
    );

    pagination.after_id = None;
    pagination.before_id = Some(max_id + 1);

    rel.push_str(&format!(
        ",</api/v1/users/?{}>; rel=last",
        serde_urlencoded::to_string(&pagination.0).unwrap()
    ));

    if !users.is_empty() {
        if users.first().unwrap().id != min_id {
            pagination.before_id = Some(min_id);
            pagination.after_id = None;

            rel.push_str(&format!(
                ",</api/v1/users/?{}>; rel=prev",
                serde_urlencoded::to_string(&pagination.0).unwrap()
            ));
        }
        if users.last().unwrap().id != max_id {
            pagination.after_id = Some(max_id);
            pagination.before_id = None;

            rel.push_str(&format!(
                ",</api/v1/users/?{}>; rel=next",
                serde_urlencoded::to_string(&pagination.0).unwrap()
            ));
        }
    }

    Ok(HttpResponse::Ok().header("Links", rel).json(users))
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
