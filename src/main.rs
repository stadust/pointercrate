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

use actix::{Actor, Addr, SyncArbiter, System};
use actix_web::{error::ResponseError, http::Method, server, App};
use crate::{
    actor::{demonlist::DatabaseActor, gdcf::GdcfActor},
    error::PointercrateError,
    middleware::ip::IpResolve,
};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use gdcf::chrono::Duration;
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig};
use gdrs::BoomlingsClient;
use log::*;
use std::env;

mod actor;
mod api;
mod error;
mod middleware;
mod model;
mod schema;
mod video;

#[allow(missing_debug_implementations)]
pub struct PointercrateState {
    database: Addr<DatabaseActor>,
    gdcf: Addr<GdcfActor>,
}

fn main() {
    dotenv::dotenv().expect("Failed to initialize .env file!");
    env_logger::init().expect("Failed to initialize logging environment!");

    let _system = System::new("pointercrate");

    let gdcf_url = env::var("GDCF_DATABASE_URL").expect("GDCF_DATABASE_URL is not set");

    let mut config = DatabaseCacheConfig::postgres_config(&gdcf_url);
    config.invalidate_after(Duration::minutes(30));

    let cache = DatabaseCache::new(config);
    let client = BoomlingsClient::new();

    let actor = GdcfActor::new(client, cache);
    let gdcf_addr = actor.start();

    info!("Initialized GDCF");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager).expect("Failed to create database connection pool");
    let db_addr = SyncArbiter::start(4, move || DatabaseActor(pool.clone()));

    info!("Initialized pointercrate database connection pool");

    let app_factory = move || {
        let state = PointercrateState {
            database: db_addr.clone(),
            gdcf: gdcf_addr.clone(),
        };

        App::with_state(state)
            .middleware(IpResolve)
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

    server::new(app_factory).bind("127.0.0.1:8088").unwrap().run();
}
