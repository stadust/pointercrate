use std::sync::OnceLock;

use rocket::{
    fairing::{Fairing, Info, Kind},
    Data, Orbit, Request, Rocket, Route,
};

// heavily inspired by rocket's `rocket::fairing::AdHoc::uri_normalizer()` implementation
// only difference is that this applies a trailing slash internally as opposed to omitting it
// https://api.rocket.rs/master/src/rocket/fairing/ad_hoc.rs#315
pub fn uri_normalizer() -> impl Fairing {
    #[derive(Default)]
    struct Normalizer {
        routes: OnceLock<Vec<Route>>,
    }

    impl Normalizer {
        fn routes(&self, rocket: &Rocket<Orbit>) -> &[Route] {
            // gather all defined routes which have a trailing slash
            self.routes.get_or_init(|| {
                rocket
                    .routes()
                    .filter(|r| r.uri.has_trailing_slash() || r.uri.path() == "/")
                    .cloned()
                    .collect()
            })
        }
    }

    #[rocket::async_trait]
    impl Fairing for Normalizer {
        fn info(&self) -> Info {
            Info {
                name: "URI Normalizer",
                kind: Kind::Liftoff | Kind::Request,
            }
        }

        async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
            let _ = self.routes(rocket);
        }

        async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
            if request.uri().has_trailing_slash() {
                return;
            }

            if let Some(normalized) = request.uri().map_path(|p| format!("{}/", p)) {
                // check if the normalized uri (the request uri with a trailing slash) matches one of our defined routes
                let mut normalized_req = request.clone();
                normalized_req.set_uri(normalized.clone());

                if self.routes(request.rocket()).iter().any(|r| {
                    // we need to leverage rocket's route matching otherwise this will suck
                    r.matches(&normalized_req)
                }) {
                    // the request doesn't have a trailing slash AND it's trying to reach one of our defined routes
                    // so just point it to our defined route
                    request.set_uri(normalized);
                }
            }
        }
    }

    Normalizer::default()
}
