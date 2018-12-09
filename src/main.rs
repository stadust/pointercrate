#![feature(proc_macro_hygiene)]
#![allow(proc_macro_derive_resolution_fallback)]
#![deny(
    bare_trait_objects,
    missing_debug_implementations,
    unused_extern_crates,
    patterns_in_fns_without_body,
    stable_features,
    unknown_lints,
    unused_features,
    //unused_imports,
    unused_parens
)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy::all))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
#![recursion_limit = "512"]

// TODO: manual deserialization of json and urlencoded (for pagaination) request data so we can
// provided better error reporting

// idk why we still need this extern crate, but removing it break the diesel derives
#[macro_use]
extern crate diesel;

type Result<T> = std::result::Result<T, PointercrateError>;

use actix::System;
use actix_web::{error::ResponseError, fs, http::Method, server, App};
use crate::{
    actor::{database::DatabaseActor, gdcf::GdcfActor},
    error::PointercrateError,
    middleware::{auth::Authorizer, cond::Precondition, ip::IpResolve},
    state::{Http, PointercrateState},
    view::{home::Homepage, Page},
};

#[macro_use]
pub mod operation;
#[macro_use]
pub mod model;
pub mod actor;
pub mod api;
pub mod bitstring;
pub mod config;
pub mod error;
pub mod middleware;
pub mod schema;
pub mod state;
pub mod video;
pub mod view;

macro_rules! mna {
    ($($allowed: expr),*) => {
        |_| {
            PointercrateError::MethodNotAllowed {
                allowed_methods: vec![$($allowed,)*]
            }.error_response()
        }
    }
}

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
            .middleware(Precondition)
            .handler(
                "/static",
                fs::StaticFiles::new("static").unwrap().show_files_listing(),
            )
            .resource("/", |r| r.get().f(|_| Homepage.render()))
            .scope("/api/v1", |api_scope| {
                api_scope
                    .nested("/users", |user_scope| {
                        user_scope
                            .resource("/", |r| r.get().f(api::user::paginate))
                            .resource("/{user_id}/", |r| {
                                r.get().f(api::user::user);
                                r.method(Method::PATCH).f(api::user::patch);
                                r.delete().f(api::user::delete);
                                r.route()
                                    .f(mna!(Method::GET, Method::PATCH, Method::DELETE))
                            })
                    })
                    .nested("/demons", |demon_scope| {
                        demon_scope.resource("/", |r| {
                            r.get().f(api::demon::paginate);
                            r.post().f(api::demon::post)
                        })
                    })
                    .nested("/records", |record_scope| {
                        record_scope
                            .resource("/", |r| {
                                r.post().f(api::record::submit);
                                r.route().f(mna!(Method::POST))
                            })
                            .resource("/{record_id}/", |r| {
                                r.get().f(api::record::get);
                                r.route().f(mna!(Method::GET))
                            })
                    })
                    .nested("/auth", |auth_scope| {
                        auth_scope
                            .resource("/", |r| {
                                r.post().f(api::auth::login);
                                r.route().f(mna!(Method::POST))
                            })
                            .resource("/register/", |r| {
                                r.post().f(api::auth::register);
                                r.route().f(mna!(Method::POST))
                            })
                            .resource("/me/", |r| {
                                r.get().f(api::auth::me);
                                r.delete().f(api::auth::delete_me);
                                r.method(Method::PATCH).f(api::auth::patch_me);
                                r.route()
                                    .f(mna!(Method::GET, Method::PATCH, Method::DELETE))
                            })
                            .resource("/invalidate/", |r| {
                                r.post().f(api::auth::invalidate);
                                r.route().f(mna!(Method::POST))
                            })
                    })
            })
    };

    server::new(app_factory)
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
