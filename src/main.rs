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

use crate::{
    actor::{database::DatabaseActor, gdcf::GdcfActor},
    error::PointercrateError,
    middleware::{auth::Authorizer, cond::Precondition, ip::IpResolve},
    state::{Http, PointercrateState},
    view::{documentation::Documentation, home::Homepage, Page},
};
use actix::System;
use actix_web::{error::ResponseError, fs, http::Method, server, App};
use std::sync::Arc;

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

// TODO: for all database code: fucking fix the order of the PgConnection parameter (and the others)

// TODO: custom 404 handling, how does it work??????

fn main() {
    dotenv::dotenv().expect("Failed to initialize .env file!");
    env_logger::init().expect("Failed to initialize logging environment!");

    let _system = System::new("pointercrate");

    let gdcf = GdcfActor::from_env();
    let database = DatabaseActor::from_env();
    let http = Http::from_env();

    let app_factory = move || {
        let doc_files_location = env!("OUT_DIR");
        let doc_files_location = std::path::Path::new(&doc_files_location);

        let toc = std::fs::read_to_string(doc_files_location.join("../output")).unwrap();
        let mut map = std::collections::HashMap::new();
        for entry in std::fs::read_dir(doc_files_location).unwrap() {
            let entry = entry.unwrap();
            if let Some("html") = entry.path().extension().and_then(std::ffi::OsStr::to_str) {
                let cnt = std::fs::read_to_string(entry.path()).unwrap();

                map.insert(
                    entry
                        .path()
                        .file_stem()
                        .and_then(std::ffi::OsStr::to_str)
                        .unwrap()
                        .to_string(),
                    cnt,
                );
            }
        }

        let state = PointercrateState {
            database: database.clone(),
            gdcf: gdcf.clone(),
            http: http.clone(),

            documentation_toc: Arc::new(toc),
            documentation_topics: Arc::new(map),
        };

        App::with_state(state)
            .middleware(IpResolve)
            .middleware(Authorizer)
            .middleware(Precondition)
            .handler(
                "/static",
                fs::StaticFiles::new("static").unwrap(),
            )
            .resource("/", |r| {
                r.name("home");
                r.get().f(|req| Homepage.render(req))
            })
            .resource("/demonlist/{position}/", |r| r.name("demonlist"))
            .resource("/about/", |r| r.name("about"))  // TODO: this
            .resource("/documentation/", |r| {
                r.name("documentation");
                r.get().f(|req| Documentation::new(req.state(), "index").map(|d|d.render(req)))
            })
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
                        demon_scope
                            .resource("/", |r| {
                                r.get().f(api::demon::paginate);
                                r.post().f(api::demon::post)
                            })
                            .resource("/{position}/", |r| {
                                r.get().f(api::demon::get);
                                r.method(Method::PATCH).f(api::demon::patch)
                            })
                            .resource("/{position}/creators/", |r| r.post().f(api::demon::post_creator))
                            .resource("/{position}/creators/{player_id}/", |r| r.delete().f(api::demon::delete_creator))
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
