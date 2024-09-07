use crate::{
    auth::{BasicAuth, TokenAuth},
    ratelimits::UserRatelimits,
};
use pointercrate_core::permission::PermissionsManager;
use pointercrate_core_api::response::Page;
use pointercrate_core_pages::head::HeadLike;
use pointercrate_user::error::UserError;
use pointercrate_user_pages::account::AccountPageConfig;

use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    response::Redirect,
    State,
};
use std::net::IpAddr;

#[cfg(feature = "legacy_accounts")]
use {
    pointercrate_core::pool::PointercratePool,
    pointercrate_user::{
        auth::legacy::{LegacyAuthenticatedUser, Registration},
        auth::AuthenticatedUser,
        User,
    },
    rocket::serde::json::Json,
};

#[rocket::get("/login")]
pub async fn login_page(auth: Option<TokenAuth>) -> Result<Redirect, Page> {
    auth.map(|_| Redirect::to(rocket::uri!(account_page)))
        .ok_or_else(|| Page::new(pointercrate_user_pages::login::login_page()))
}

#[rocket::post("/login")]
pub async fn login(
    auth: Result<BasicAuth, UserError>, ip: IpAddr, ratelimits: &State<UserRatelimits>, cookies: &CookieJar<'_>,
) -> pointercrate_core_api::error::Result<Status> {
    ratelimits.login_attempts(ip)?;

    let auth = auth?;

    let mut cookie = Cookie::build(("access_token", auth.user.generate_access_token()))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path("/");

    if !cfg!(debug_assertions) {
        cookie = cookie.secure(true)
    }

    cookies.add(cookie);

    Ok(Status::NoContent)
}

#[cfg(feature = "legacy_accounts")]
#[rocket::post("/register", data = "<registration>")]
pub async fn register(
    ip: IpAddr, ratelimits: &State<UserRatelimits>, cookies: &CookieJar<'_>, registration: Json<Registration>,
    pool: &State<PointercratePool>,
) -> pointercrate_core_api::error::Result<Status> {
    let mut connection = pool.transaction().await.map_err(UserError::from)?;

    ratelimits.soft_registrations(ip)?;

    LegacyAuthenticatedUser::validate_password(&registration.password)?;
    User::validate_name(&registration.name)?;

    ratelimits.registrations(ip)?;

    let user = AuthenticatedUser::register(registration.0, &mut *connection).await?;

    connection.commit().await.map_err(UserError::from)?;

    let mut cookie = Cookie::build(("access_token", user.generate_access_token()))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path("/");

    if !cfg!(debug_assertions) {
        cookie = cookie.secure(true)
    }

    cookies.add(cookie);

    Ok(Status::Created)
}

#[rocket::get("/account")]
pub async fn account_page(
    auth: Option<TokenAuth>, permissions: &State<PermissionsManager>, tabs: &State<AccountPageConfig>,
) -> Result<Page, Redirect> {
    match auth {
        Some(mut auth) => {
            let csrf_token = auth.user.generate_csrf_token();

            Ok(Page::new(tabs.account_page(auth.user, permissions, &mut auth.connection).await).meta("csrf_token", csrf_token))
        },
        None => Err(Redirect::to(rocket::uri!(login_page))),
    }
}
