use crate::{
    auth::{BasicAuth, TokenAuth},
    ratelimits::UserRatelimits,
};
use pointercrate_core::{etag::Taggable, pool::PointercratePool};
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, Tagged},
    response::Response2,
};
use pointercrate_user::{error::UserError, AuthenticatedUser, PatchMe, Registration, User};
use rocket::{
    http::Status,
    serde::json::{serde_json, Json},
    State,
};
use std::net::IpAddr;

#[rocket::post("/register", data = "<body>")]
pub async fn register(
    ip: IpAddr, body: Json<Registration>, ratelimits: &State<UserRatelimits>, pool: &State<PointercratePool>,
) -> Result<Response2<Tagged<User>>> {
    let mut connection = pool.transaction().await.map_err(UserError::from)?;

    ratelimits.soft_registrations(ip)?;

    AuthenticatedUser::validate_password(&body.password)?;
    User::validate_name(&body.name)?;

    ratelimits.registrations(ip)?;

    let user = AuthenticatedUser::register(body.0, &mut connection).await?;

    connection.commit().await.map_err(UserError::from)?;

    Ok(Response2::tagged(user.into_inner())
        .with_header("Location", "api/v1/auth/me")
        .status(Status::Created))
}

#[rocket::post("/")]
pub async fn login(
    auth: std::result::Result<BasicAuth, UserError>, ip: IpAddr, ratelimits: &State<UserRatelimits>,
) -> Result<Response2<Json<serde_json::Value>>> {
    ratelimits.login_attempts(ip)?;
    let auth = auth?;

    Ok(Response2::json(serde_json::json! {
        {
            "data": auth.user.inner(),
            "token": auth.user.generate_token(&pointercrate_core::config::secret())
        }
    })
    .with_header("etag", auth.user.inner().etag_string()))
}

#[rocket::post("/invalidate")]
pub async fn invalidate(mut auth: BasicAuth) -> Result<Status> {
    auth.user.invalidate_all_tokens(&auth.secret, &mut auth.connection).await?;
    auth.connection.commit().await.map_err(UserError::from)?;

    Ok(Status::NoContent)
}

#[rocket::get("/me")]
pub fn get_me(auth: TokenAuth) -> Tagged<User> {
    Tagged(auth.user.into_inner())
}

#[rocket::patch("/me", data = "<patch>")]
pub async fn patch_me(mut auth: BasicAuth, patch: Json<PatchMe>, pred: Precondition) -> Result<std::result::Result<Tagged<User>, Status>> {
    pred.require_etag_match(auth.user.inner())?;

    let changes_password = patch.changes_password();

    let updated_user = auth.user.apply_patch(patch.0, &mut auth.connection).await?;

    auth.connection.commit().await.map_err(UserError::from)?;

    if changes_password {
        Ok(Err(Status::NotModified))
    } else {
        Ok(Ok(Tagged(updated_user.into_inner())))
    }
}

#[rocket::delete("/me")]
pub async fn delete_me(mut auth: BasicAuth, pred: Precondition) -> Result<Status> {
    pred.require_etag_match(auth.user.inner())?;

    auth.user.delete(&mut auth.connection).await?;
    auth.connection.commit().await.map_err(UserError::from)?;

    Ok(Status::NoContent)
}
