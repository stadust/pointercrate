use crate::{auth::Auth, ratelimits::UserRatelimits};
use pointercrate_core::etag::Taggable;
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, Tagged},
    response::Response2,
};
use pointercrate_user::{
    auth::{ApiToken, PasswordOrBrowser, PatchMe},
    error::UserError,
    User,
};
use rocket::{
    http::Status,
    serde::json::{serde_json, Json},
    State,
};
use std::net::IpAddr;

#[cfg(feature = "legacy_accounts")]
use {
    pointercrate_core::pool::PointercratePool,
    pointercrate_user::auth::legacy::{LegacyAuthenticatedUser, Registration},
};

#[cfg(feature = "legacy_accounts")]
#[rocket::post("/register", data = "<body>")]
pub async fn register(
    ip: IpAddr, body: Json<Registration>, ratelimits: &State<UserRatelimits>, pool: &State<PointercratePool>,
) -> Result<Response2<Tagged<User>>> {
    use pointercrate_user::auth::AuthenticatedUser;

    let mut connection = pool.transaction().await.map_err(UserError::from)?;

    ratelimits.soft_registrations(ip)?;

    LegacyAuthenticatedUser::validate_password(&body.password)?;
    User::validate_name(&body.name)?;

    let user = AuthenticatedUser::register(body.0, &mut connection).await?;

    ratelimits.registrations(ip)?;

    connection.commit().await.map_err(UserError::from)?;

    Ok(Response2::tagged(user.into_user())
        .with_header("Location", "api/v1/auth/me")
        .status(Status::Created))
}

#[rocket::post("/")]
pub async fn login(
    auth: std::result::Result<Auth<PasswordOrBrowser>, UserError>, ip: IpAddr, ratelimits: &State<UserRatelimits>,
) -> Result<Response2<Json<serde_json::Value>>> {
    ratelimits.login_attempts(ip)?;
    let auth = auth?;

    Ok(Response2::json(serde_json::json! {
        {
            "data": auth.user.user(),
            "token": auth.user.generate_programmatic_access_token()
        }
    })
    .with_header("etag", auth.user.user().etag_string()))
}

#[rocket::post("/invalidate")]
pub async fn invalidate(mut auth: Auth<PasswordOrBrowser>) -> Result<Status> {
    auth.user.invalidate_all_tokens(&mut auth.connection).await?;
    auth.connection.commit().await.map_err(UserError::from)?;

    Ok(Status::NoContent)
}

#[rocket::get("/me")]
pub fn get_me(auth: Auth<ApiToken>) -> Tagged<User> {
    Tagged(auth.user.into_user())
}

#[rocket::patch("/me", data = "<patch>")]
pub async fn patch_me(
    mut auth: Auth<PasswordOrBrowser>, patch: Json<PatchMe>, pred: Precondition,
) -> Result<std::result::Result<Tagged<User>, Status>> {
    pred.require_etag_match(auth.user.user())?;

    let changes_password = patch.changes_password();

    let updated_user = auth.user.apply_patch(patch.0, &mut auth.connection).await?;

    auth.connection.commit().await.map_err(UserError::from)?;

    if changes_password {
        Ok(Err(Status::NoContent))
    } else {
        Ok(Ok(Tagged(updated_user)))
    }
}

#[rocket::delete("/me")]
pub async fn delete_me(mut auth: Auth<PasswordOrBrowser>, pred: Precondition) -> Result<Status> {
    pred.require_etag_match(auth.user.user())?;

    auth.user.delete(&mut auth.connection).await?;
    auth.connection.commit().await.map_err(UserError::from)?;

    Ok(Status::NoContent)
}
