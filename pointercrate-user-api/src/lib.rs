use crate::ratelimits::UserRatelimits;

use rocket::{Build, Rocket};

pub mod auth;
mod endpoints;
#[cfg(feature = "oauth2")]
mod oauth;
mod pages;
mod ratelimits;

#[allow(unused_mut)]
pub fn setup(mut rocket: Rocket<Build>) -> Rocket<Build> {
    let ratelimits = UserRatelimits::new();

    let mut auth_routes = rocket::routes![
        endpoints::auth::login,
        endpoints::auth::invalidate,
        endpoints::auth::get_me,
        endpoints::auth::patch_me,
        endpoints::auth::delete_me,
    ];
    let mut page_routes = rocket::routes![pages::login_page, pages::account_page, pages::login, pages::logout];
    #[cfg(feature = "legacy_accounts")]
    auth_routes.extend(rocket::routes![endpoints::auth::register]);
    #[cfg(feature = "legacy_accounts")]
    page_routes.extend(rocket::routes![pages::register]);
    #[cfg(feature = "oauth2")]
    auth_routes.extend(rocket::routes![pages::google_oauth_login]);

    #[cfg(feature = "oauth2")]
    {
        rocket = rocket.manage(oauth::GoogleCertificateStore::default());
    }

    rocket
        .manage(ratelimits)
        .mount("/api/v1/auth/", auth_routes)
        .mount(
            "/api/v1/users/",
            rocket::routes![
                endpoints::user::paginate,
                endpoints::user::get_user,
                endpoints::user::patch_user,
                endpoints::user::delete_user
            ],
        )
        .mount("/", page_routes)
}
