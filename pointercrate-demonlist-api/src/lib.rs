use crate::endpoints::misc;
use rocket::{Build, Rocket};

mod endpoints;

pub fn setup(mut rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/api/v1/list_information/", rocket::routes![misc::list_information])
}
