//! Module providing a "maintenance mode" fairing (middleware)

use crate::error::Result;
use pointercrate_core::error::CoreError;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Method,
    routes, uri, Build, Data, Request, Rocket,
};

/// Rocket fairing that causes all mutating requests (aka non-GET requests) to return 503 SERVICE UNAVAILABLE if `.0` is `true`.
///
/// Works in a very hacky way, as rocket does not allow fairing to terminate requests. Thus we instead rewrite the
/// request on the fly to be a GET /maintenance, which is an endpoint that unconditionally returns a 503 response.
/// This endpoint only exists when maintainence mode is active (it is dynamically mounted in `on_ignite`).
///
/// Idea taken from https://stackoverflow.com/questions/70011965/global-authentication-authorization-in-rocket-based-on-a-header
#[derive(Default)]
pub struct MaintenanceFairing(bool);

impl MaintenanceFairing {
    pub fn new(read_only: bool) -> Self {
        MaintenanceFairing(read_only)
    }
}

#[rocket::async_trait]
impl Fairing for MaintenanceFairing {
    fn info(&self) -> Info {
        Info {
            name: "Maintenance",
            kind: Kind::Ignite | Kind::Request,
        }
    }

    async fn on_ignite(&self, mut rocket: Rocket<Build>) -> rocket::fairing::Result {
        if self.0 {
            log::warn!("Maintenance mode activated! All non-GET requests will receive a 503 response!");
            rocket = rocket.mount("/", routes![maintenance]);
        }
        Ok(rocket)
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        if self.0 && request.method() != Method::Get {
            request.set_uri(uri!("/maintenance"));
            request.set_method(Method::Get);
        }
    }
}

#[rocket::get("/maintenance")]
async fn maintenance() -> Result<()> {
    Err(CoreError::ReadOnlyMaintenance.into())
}
