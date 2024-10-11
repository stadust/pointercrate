use crate::{auth::Auth, ratelimits::UserRatelimits};
use pointercrate_core::permission::PermissionsManager;
use pointercrate_core_api::response::Page;
use pointercrate_user::{
    auth::{NonMutating, PasswordOrBrowser},
    error::UserError,
};
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
pub async fn login_page(auth: Option<Auth<NonMutating>>) -> Result<Redirect, Page> {
    auth.map(|_| Redirect::to(rocket::uri!(account_page)))
        .ok_or_else(|| Page::new(pointercrate_user_pages::login::login_page()))
}

// Doing the post with cookies already set will just refresh them. No point in doing that, but also not harmful.
#[rocket::post("/login")]
pub async fn login(
    auth: Result<Auth<PasswordOrBrowser>, UserError>, ip: IpAddr, ratelimits: &State<UserRatelimits>, cookies: &CookieJar<'_>,
) -> pointercrate_core_api::error::Result<Status> {
    ratelimits.login_attempts(ip)?;

    let auth = auth?;

    let (access_token, csrf_token) = auth.user.generate_token_pair()?;

    let cookie = Cookie::build(("access_token", access_token))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(!cfg!(debug_assertions))
        .path("/");

    cookies.add(cookie);

    let cookie = Cookie::build(("csrf_token", csrf_token))
        .http_only(false)
        .same_site(SameSite::Strict)
        .secure(!cfg!(debug_assertions))
        .path("/");

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

    let (access_token, csrf_token) = user.generate_token_pair()?;

    let cookie = Cookie::build(("access_token", access_token))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(!cfg!(debug_assertions))
        .path("/");

    cookies.add(cookie);

    let cookie = Cookie::build(("csrf_token", csrf_token))
        .http_only(false)
        .same_site(SameSite::Strict)
        .secure(!cfg!(debug_assertions))
        .path("/");

    cookies.add(cookie);

    Ok(Status::Created)
}

#[rocket::get("/account")]
pub async fn account_page(
    auth: Option<Auth<NonMutating>>, permissions: &State<PermissionsManager>, tabs: &State<AccountPageConfig>,
) -> Result<Page, Redirect> {
    match auth {
        Some(mut auth) => Ok(Page::new(tabs.account_page(auth.user, permissions, &mut auth.connection).await)),
        None => Err(Redirect::to(rocket::uri!(login_page))),
    }
}

#[rocket::get("/logout")]
pub async fn logout(_auth: Auth<NonMutating>, cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove("access_token");
    cookies.remove("csrf_token");

    Redirect::to(rocket::uri!(login_page))
}
