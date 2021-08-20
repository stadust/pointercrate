use pointercrate_core::error::CoreError;
use rocket::{Build, Rocket};

pub mod auth;
mod endpoints;

#[rocket::get("/")]
fn error() -> pointercrate_core_api::error::Result<()> {
    Err(CoreError::NotFound)?
}

pub fn setup(mut rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .mount("/api/v1/auth/me/", rocket::routes![endpoints::get_me])
        .mount("/error", rocket::routes![error])
}
