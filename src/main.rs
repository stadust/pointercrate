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
    unused_imports,
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
    api::wrap,
    error::PointercrateError,
    middleware::{auth::Authorizer, cond::Precondition, ip::IpResolve, mime::MimeProcess},
    state::{Http, PointercrateState},
    view::{documentation::Documentation, home::Homepage, Page},
};
use actix::System;
use actix_web::{
    dev::Handler,
    fs,
    http::{Method, NormalizePath, StatusCode},
    server, App, FromRequest, Path,
};
use std::sync::Arc;

#[macro_use]
pub mod operation;
#[macro_use]
pub mod permissions;
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

macro_rules! allowed {
    ($($allowed: ident),*) => {
        |req| {
            let error = PointercrateError::MethodNotAllowed {
                allowed_methods: vec![$(Method::$allowed,)*]
            };

            crate::api::error(req, error)
        }
    };
}

fn main() {
    env_logger::init();
    dotenv::dotenv().expect("Failed to initialize .env file!");

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
            .middleware(MimeProcess)
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
                r.get().f(|req| Documentation::new(req.state(), "index".into()).map(|d|d.render(req)));
                r.route().f(allowed!(GET))
            })
            .resource("/documentation/{page}/", |r| {
                r.get().f(|req|{
                    Path::<String>::extract(req)
                        .map(|page| page.into_inner())
                        .and_then(|page| Documentation::new(req.state(), page).map_err(Into::into))
                        .map(|d|d.render(req))
                });
                r.route().f(allowed!(GET))
            })
            .scope("/api/v1", |api_scope| {
                api_scope
                    .nested("/users", |user_scope| {
                        user_scope
                            .resource("/", |r| r.get().f(wrap(api::user::paginate)))
                            .resource("/{user_id}/", |r| {
                                r.get().f(wrap(api::user::get));
                                r.method(Method::PATCH).f(wrap(api::user::patch));
                                r.delete().f(wrap(api::user::delete));
                                r.route().f(allowed!(GET, PATCH, DELETE))
                            })
                    })
                    .nested("/demons", |demon_scope| {
                        demon_scope
                            .resource("/", |r| {
                                r.get().f(wrap(api::demon::paginate));
                                r.post().f(wrap(api::demon::post));
                                r.route().f(allowed!(GET, POST))
                            })
                            .resource("/{position}/", |r| {
                                r.get().f(wrap(api::demon::get));
                                r.method(Method::PATCH).f(wrap(api::demon::patch));
                                r.route().f(allowed!(GET, PATCH))
                            })
                            .resource("/{position}/creators/", |r| {
                                r.post().f(wrap(api::demon::post_creator));
                                r.route().f(allowed!(POST))
                            })
                            .resource("/{position}/creators/{player_id}/", |r| {
                                r.delete().f(wrap(api::demon::delete_creator));
                                r.route().f(allowed!(DELETE))
                            })
                    })
                    .nested("/players", |player_scope|{
                        player_scope
                            .resource("/", |r| {
                                r.get().f(wrap(api::player::paginate));
                                r.route().f(allowed!(GET))
                            })
                            .resource("/{player_id}/", |r|{
                                r.get().f(wrap(api::player::get));
                                r.method(Method::PATCH).f(wrap(api::player::patch));
                                r.route().f(allowed!(GET, PATCH))
                            })
                    })
                    .nested("/records", |record_scope| {
                        record_scope
                            .resource("/", |r| {
                                r.get().f(wrap(api::record::paginate));
                                r.post().f(wrap(api::record::submit));
                                r.route().f(allowed!(POST))
                            })
                            .resource("/{record_id}/", |r| {
                                r.get().f(wrap(api::record::get));
                                r.delete().f(wrap(api::record::delete));
                                r.route().f(allowed!(GET, DELETE))
                            })
                    })
                    .nested("/auth", |auth_scope| {
                        auth_scope
                            .resource("/", |r| {
                                r.post().f(wrap(api::auth::login));
                                r.route().f(allowed!(POST))
                            })
                            .resource("/register/", |r| {
                                r.post().f(wrap(api::auth::register));
                                r.route().f(allowed!(POST))
                            })
                            .resource("/me/", |r| {
                                r.get().f(wrap(api::auth::me));
                                r.delete().f(wrap(api::auth::delete_me));
                                r.method(Method::PATCH).f(wrap(api::auth::patch_me));
                                r.route().f(allowed!(GET, PATCH, DELETE))
                            })
                            .resource("/invalidate/", |r| {
                                r.post().f(wrap(api::auth::invalidate));
                                r.route().f(allowed!(POST))
                            })
                    })
            })
            .default_resource(|r| {
                let normalizer = NormalizePath::default();

                r.get().f(move |request| {
                    let normalized = normalizer.handle(request);

                    if normalized.status() == StatusCode::NOT_FOUND {
                        return crate::api::error(request, PointercrateError::NotFound)
                    }

                    Ok(normalized)
                });
                r.route().f(|req| crate::api::error(req, PointercrateError::NotFound))
            })
    };

    server::new(app_factory)
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
