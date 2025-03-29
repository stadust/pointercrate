use crate::{auth::Auth, ratelimits::UserRatelimits};
use pointercrate_core::permission::PermissionsManager;
use pointercrate_core_api::response::Page;
use pointercrate_user::auth::AuthenticatedUser;
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

#[cfg(any(feature = "legacy_accounts", feature = "oauth2"))]
use {pointercrate_core::pool::PointercratePool, rocket::serde::json::Json};

#[cfg(feature = "legacy_accounts")]
use pointercrate_user::{
    auth::legacy::{LegacyAuthenticatedUser, Registration},
    User,
};

#[cfg(feature = "oauth2")]
use {crate::oauth::GoogleCertificateStore, pointercrate_core::error::CoreError, pointercrate_user::auth::oauth::GoogleOauthPayload};

fn build_cookies(user: &AuthenticatedUser<PasswordOrBrowser>, cookies: &CookieJar<'_>) -> pointercrate_user::error::Result<()> {
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

    Ok(())
}

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

    build_cookies(&auth.user, cookies)?;

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

    let user = AuthenticatedUser::register(registration.0, &mut connection).await?;

    connection.commit().await.map_err(UserError::from)?;

    build_cookies(&user, cookies)?;

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

#[cfg(feature = "oauth2")]
#[rocket::post("/oauth/google", data = "<payload>")]
pub async fn google_oauth_login(
    payload: Json<GoogleOauthPayload>, auth: Option<Auth<PasswordOrBrowser>>, key_store: &State<GoogleCertificateStore>,
    pool: &State<PointercratePool>, cookies: &rocket::http::CookieJar<'_>,
) -> pointercrate_core_api::error::Result<Status> {
    if key_store.needs_refresh().await {
        key_store
            .refresh()
            .await
            .map_err(|err| CoreError::internal_server_error(format!("Failed to retrieve signing certificates from Google! {:?}", err)))?;
    }

    let validated_credentials = key_store
        .validate_credentials(&payload.into_inner().credential)
        .await
        .ok_or(CoreError::Unauthorized)?;

    let maybe_linked_user = AuthenticatedUser::by_validated_google_creds(&validated_credentials, &mut *pool.connection().await?).await;

    let authenticated_user = match auth {
        None => maybe_linked_user?,
        Some(mut signed_in_user) => {
            // Unauthorized = No linked account found. But in the flow that is supposed to establish the link,
            // that is exactly what we need.
            if !matches!(maybe_linked_user, Err(UserError::Core(CoreError::Unauthorized))) {
                return Err(CoreError::Unauthorized.into());
            }

            signed_in_user
                .user
                .link_google_account(&validated_credentials, &mut signed_in_user.connection)
                .await?;
            signed_in_user.connection.commit().await.map_err(UserError::from)?;
            signed_in_user.user
        },
    };

    build_cookies(&authenticated_user, cookies)?;

    Ok(Status::NoContent)
}
