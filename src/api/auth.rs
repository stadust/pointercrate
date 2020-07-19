//! Handlers for all endpoints under the `/api/v1/auth` prefix

use crate::{
    extractor::{
        auth::{BasicAuth, TokenAuth},
        if_match::IfMatch,
        ip::Ip,
    },
    model::user::{AuthenticatedUser, Authorization, PatchMe, Registration},
    ratelimit::RatelimitScope,
    state::PointercrateState,
    util::HttpResponseBuilderExt,
    ApiResult,
};
use actix_web::{web::Json, HttpResponse};
use actix_web_codegen::{delete, get, patch, post};
use serde_json::json;

#[post("/register/")]
pub async fn register(Ip(ip): Ip, body: Json<Registration>, state: PointercrateState) -> ApiResult<HttpResponse> {
    let mut connection = state.transaction().await?;
    let user = AuthenticatedUser::register(body.into_inner(), &mut connection, Some(state.ratelimits.prepare(ip))).await?;

    Ok(HttpResponse::Created()
        .header("Location", "/api/v1/auth/me/")
        .json_with_etag(user.inner()))
}

#[post("/")]
pub async fn login(Ip(ip): Ip, user: BasicAuth, state: PointercrateState) -> ApiResult<HttpResponse> {
    state.ratelimits.check(RatelimitScope::Login, ip)?;

    Ok(HttpResponse::Ok().etag(user.0.inner()).json(json! {{
        "data": user.0.inner(),
        "token": user.0.generate_token(&state.secret)
    }}))
}

#[post("/invalidate/")]
pub async fn invalidate(authorization: Authorization, state: PointercrateState) -> ApiResult<HttpResponse> {
    AuthenticatedUser::invalidate_all_tokens(authorization, &mut *state.connection().await?).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("/me/")]
pub async fn get_me(user: TokenAuth) -> ApiResult<HttpResponse> {
    Ok(HttpResponse::Ok().json_with_etag(user.0.inner()))
}

// FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
#[patch("/me/")]
pub async fn patch_me(
    if_match: IfMatch, BasicAuth(user): BasicAuth, state: PointercrateState, patch: Json<PatchMe>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user).await?;

    if_match.require_etag_match(user.inner())?;

    let changes_password = patch.changes_password();

    let updated_user = user.apply_patch(patch.into_inner(), &mut connection).await?;

    connection.commit().await?;

    if changes_password {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::Ok().json_with_etag(updated_user.inner()))
    }
}

// FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
#[delete("/me/")]
pub async fn delete_me(if_match: IfMatch, BasicAuth(user): BasicAuth, state: PointercrateState) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user).await?;

    if_match.require_etag_match(user.inner())?;

    user.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
