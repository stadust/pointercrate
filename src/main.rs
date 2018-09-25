#![allow(proc_macro_derive_resolution_fallback)]
#![deny(
    bare_trait_objects,
    missing_debug_implementations,
    unused_extern_crates,
    patterns_in_fns_without_body,
    stable_features,
    unknown_lints,
    unused_features,
    unused_imports,
    unused_parens
)]

// idk why we still need this extern crate, but removing it break the diesel derives
#[macro_use]
extern crate diesel;

use actix::System;
use actix_web::{error::ResponseError, http::Method, server, App};
use crate::{
    actor::{database::DatabaseActor, gdcf::GdcfActor},
    error::PointercrateError,
    middleware::{auth::Authorizer, ip::IpResolve},
    state::{Http, PointercrateState},
};

mod actor;
mod api;
//mod auth;
mod config;
mod error;
mod middleware;
mod model;
mod schema;
mod state;
mod video;

fn main() {
    dotenv::dotenv().expect("Failed to initialize .env file!");
    env_logger::init().expect("Failed to initialize logging environment!");

    let _system = System::new("pointercrate");

    let gdcf = GdcfActor::from_env();
    let database = DatabaseActor::from_env();
    let http = Http::from_env();

    let app_factory = move || {
        let state = PointercrateState {
            database: database.clone(),
            gdcf: gdcf.clone(),
            http: http.clone(),
        };

        App::with_state(state)
            .middleware(IpResolve)
            .middleware(Authorizer)
            .resource("/api/v1/records/", |r| {
                r.post().f(api::record::submit);
                r.route().f(|_| {
                    PointercrateError::MethodNotAllowed {
                        allowed_methods: vec![Method::POST],
                    }.error_response()
                })
            }).resource("/api/v1/records/{record_id}/", |r| {
                r.get().f(api::record::get);
                r.route().f(|_| {
                    PointercrateError::MethodNotAllowed {
                        allowed_methods: vec![Method::GET],
                    }.error_response()
                })
            })
    };

    server::new(app_factory)
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
