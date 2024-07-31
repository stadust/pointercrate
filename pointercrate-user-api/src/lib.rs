use crate::ratelimits::UserRatelimits;

use rocket::{Build, Rocket};

pub mod auth;
mod endpoints;
mod pages;
mod ratelimits;

pub fn setup(rocket: Rocket<Build>) -> Rocket<Build> {
    let ratelimits = UserRatelimits::new();

    rocket
        .manage(ratelimits)
        .mount(
            "/api/v1/auth/",
            rocket::routes![
                endpoints::auth::login,
                endpoints::auth::invalidate,
                endpoints::auth::get_me,
                endpoints::auth::patch_me,
                endpoints::auth::delete_me,
                endpoints::auth::authorize,
                endpoints::auth::callback,
            ],
        )
        .mount(
            "/api/v1/users/",
            rocket::routes![
                endpoints::user::paginate,
                endpoints::user::get_user,
                endpoints::user::patch_user,
                endpoints::user::delete_user
            ],
        )
        .mount(
            "/",
            rocket::routes![pages::login_page, pages::account_page, pages::login, pages::register],
        )
}
