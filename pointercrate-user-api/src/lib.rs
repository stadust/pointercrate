use crate::ratelimits::UserRatelimits;
use pointercrate_core::{
    error::CoreError,
    permission::{Permission},
};
use rocket::{Build, Rocket};

pub mod auth;
mod endpoints;
mod ratelimits;

pub const ADMINISTRATOR: Permission = Permission::new("Administrator", 0x4000);
pub const MODERATOR: Permission = Permission::new("Moderator", 0x2000);

#[rocket::get("/")]
fn error() -> pointercrate_core_api::error::Result<()> {
    Err(CoreError::NotFound)?
}

pub fn setup(rocket: Rocket<Build>) -> Rocket<Build> {
    let ratelimits = UserRatelimits::new();

    rocket
        .manage(ratelimits)
        .mount("/api/v1/auth/", rocket::routes![
            endpoints::auth::register,
            endpoints::auth::login,
            endpoints::auth::invalidate,
            endpoints::auth::get_me,
            endpoints::auth::patch_me,
            endpoints::auth::delete_me
        ])
        .mount("/api/v1/users/", rocket::routes![
            endpoints::user::paginate,
            endpoints::user::get_user,
            endpoints::user::patch_user,
            endpoints::user::delete_user
        ])
        .mount("/error", rocket::routes![error])
}
