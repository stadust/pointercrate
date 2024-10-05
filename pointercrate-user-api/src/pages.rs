//! Endpoints that are only intended to be used from the browser
//!
//! These are not part of the pointercrate API, either because they simply serve
//! HTML content, or because they are related to browser based authentication flows
//! (they set the access_token cookie, or are oauth related).

use crate::{
    auth::{BasicAuth, TokenAuth},
    ratelimits::UserRatelimits,
};
use pointercrate_core::permission::PermissionsManager;
#[cfg(any(feature = "legacy_accounts", feature = "oauth2"))]
use pointercrate_core::pool::PointercratePool;
#[cfg(feature = "oauth2")]
use pointercrate_core_api::error::ErrorResponder;
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
    pointercrate_user::{
        auth::legacy::{LegacyAuthenticatedUser, Registration},
        auth::AuthenticatedUser,
        User,
    },
    rocket::serde::json::Json,
};

#[cfg(feature = "oauth2")]
#[derive(serde::Serialize, serde::Deserialize)]
struct OAuthClaims {
    sub: String,
    nonce: u64,
    exp: u64,
}

#[cfg(feature = "oauth2")]
#[rocket::get("/authorize")]
pub fn authorize(
    ip: IpAddr, ratelimits: &State<UserRatelimits>, cookies: &CookieJar<'_>, auth: Option<TokenAuth>,
) -> Result<Redirect, ErrorResponder> {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use pointercrate_core::error::CoreError;
    use pointercrate_user::config;
    use rocket::time::OffsetDateTime;

    ratelimits.login_attempts(ip)?;

    let mut redirect_uri = "https://accounts.google.com/o/oauth2/v2/auth".to_string()
        + format!("?client_id={}", config::google_client_id()).as_str()
        + "&response_type=code"
        + "&prompt=consent"
        + "&scope=profile"
        + "&redirect_uri=http%3A%2F%2Flocalhost%3A1971%2Fcallback";

    if let Some(auth) = auth {
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).map_err(|err| CoreError::internal_server_error(err.to_string()))?;
        let nonce = u64::from_le_bytes(nonce);

        redirect_uri = redirect_uri
            + "&state="
            + &auth.user.generate_jwt(&OAuthClaims {
                nonce,
                sub: auth.user.user().id.to_string(),
                exp: (SystemTime::now() + Duration::from_secs(5 * 60))
                    .duration_since(UNIX_EPOCH)
                    .expect("time went backwards")
                    .as_secs(),
            });

        cookies.add(
            Cookie::build(("oauth_nonce", nonce.to_string()))
                .http_only(true)
                .secure(!cfg!(debug_assertions))
                .expires(OffsetDateTime::now_utc() + Duration::from_secs(5 * 60))
                .same_site(SameSite::Strict)
                .path("/api/v1/auth/callback"),
        )
    }

    Ok(Redirect::to(redirect_uri))
}

#[cfg(feature = "oauth2")]
#[rocket::get("/callback?<code>&<state>")]
pub async fn callback(
    auth: Option<TokenAuth>, pool: &State<PointercratePool>, ip: IpAddr, ratelimits: &State<UserRatelimits>, code: &str,
    state: Option<&str>, cookies: &CookieJar<'_>,
) -> Result<rocket::response::content::RawHtml<&'static str>, ErrorResponder> {
    use pointercrate_core::error::CoreError;
    use pointercrate_user::auth::AuthenticatedUser;
    use rocket::response::content::RawHtml;

    ratelimits.login_attempts(ip)?;

    let user = match (state, auth) {
        (Some(jwt), Some(mut auth)) => {
            let claims = auth.user.validate_jwt::<OAuthClaims>(jwt, Default::default())?;

            let nonce = cookies.get("oauth_nonce").ok_or(UserError::Core(CoreError::Unauthorized))?;

            if nonce.value() != claims.nonce.to_string() {
                return Err(CoreError::Unauthorized.into());
            }

            cookies.remove(nonce.clone());

            let user = auth.user.upgrade_legacy_account(code, &mut auth.connection).await?;

            auth.connection.commit().await.map_err(UserError::from)?;

            user
        },

        (None, None) => {
            let mut connection = pool.transaction().await.map_err(UserError::from)?;
            let user = AuthenticatedUser::by_oauth_code(code, &mut connection).await?;
            connection.commit().await.map_err(UserError::from)?;
            user
        },

        // If we do not have the state parameter, it means that we were not logged in during the request to /authorize, e.g.
        // we wanted to create a new account. However, if now we are logged in, we could be in the scenario where some attacker
        // started the oauth flow, and then tricked someone else into clicking the callback link they got from their "registration attempt".
        // In other words: We cannot verify that the person logged in now is the same person that originally called /authorize.
        // Thus return 401 UNAUTHORIZED.
        _ => return Err(CoreError::Unauthorized.into()),
    };

    let cookie = Cookie::build(("access_token", user.generate_access_token()))
        .http_only(true)
        .secure(!cfg!(debug_assertions))
        .same_site(SameSite::Strict)
        .path("/");

    cookies.add(cookie);

    // We cannot use a HTTP redirect here, because HTTP redirect preserve "Referer" informatoin. Since we arrive
    // at /callback after a redirect from google, this data will point to some google domain, and thus if we redirect
    // here, we will open /account in the context of this referal from google. However, our access_token cookie is set
    // with "Same-Site: strict", meaning it is not sent along for requests that are the result of a cross-domain referal,
    // so even if we successfully login, we would just be dropped off at the login screen, until the user manually
    // navigates somewhere else.
    //
    // However, "redirects" initiated by javascript loose the referer context, and thus if we instead do the below,
    // the browser will send the access_token cookie along with the next request, and we end up on the profile
    // page, as wanted.
    Ok(RawHtml(
        r#"
        <html><head><title>Redirecting...</title><body>Redirecting...</body><script>window.location="/account"</script></html>
        "#,
    ))
}

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
