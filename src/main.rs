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
    actor::{database::DatabaseActor, http::HttpActor},
    api::{wrap, wrap_direct},
    error::PointercrateError,
    middleware::{auth::Authorizer, cond::Precondition, ip::IpResolve, mime::MimeProcess},
    state::PointercrateState,
    view::{documentation::Documentation, Page},
};
use actix::System;
use actix_web::{
    dev::Handler,
    fs,
    http::{Method, NormalizePath, StatusCode},
    server, App, FromRequest, Path, Responder,
};
use std::{net::SocketAddr, sync::Arc};

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

    let database = DatabaseActor::from_env();
    let gdcf = HttpActor::from_env(database.clone().recipient());

    let app_factory = move || {
        let doc_files_location =
            std::env::var("DOCUMENTATION").unwrap_or(env!("OUT_DIR").to_string());
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

            documentation_toc: Arc::new(toc),
            documentation_topics: Arc::new(map),
        };

        App::with_state(state)
            .middleware(IpResolve)
            .middleware(Authorizer)
            .middleware(Precondition)
            .middleware(MimeProcess)
            .handler("/static2", fs::StaticFiles::new("static2").unwrap())
            .resource("/", |r| {
                r.name("home");
                r.get().f(wrap(view::home::handler));
                r.route().f(allowed!(GET))
            })
            .resource("/login/", |r| {
                r.get().f(view::login::handler);
                r.post().f(wrap(view::login::login));
                r.route().f(allowed!(GET, POST))
            })
            .resource("/account/", |r| {
                r.get().f(wrap(view::account::handler));
                r.route().f(allowed!(GET))
            })
            .resource("/demonlist/", |r| {
                r.name("demonlist-overview");
                r.get().f(wrap(view::demonlist::overview_handler));
                r.route().f(allowed!(GET))
            })
            .resource("/demonlist/{position}/", |r| {
                r.name("demonlist");
                r.get().f(wrap(view::demonlist::handler));
                r.route().f(allowed!(GET))
            })
            .resource("/documentation/", |r| {
                r.name("documentation");
                r.get().f(wrap_direct(|req| {
                    Documentation::new(req.state(), "index".into())
                        .map(|d| d.render(req).respond_to(req).unwrap())
                }));
                r.route().f(allowed!(GET))
            })
            .resource("/documentation/{page}/", |r| {
                r.get().f(wrap_direct(|req| {
                    Path::<String>::extract(req)
                        .map_err(|_| PointercrateError::GenericBadRequest)  // no idea how this could happen
                        .map(|page| page.into_inner())
                        .and_then(|page| Documentation::new(req.state(), page))
                        .map(|d| d.render(req).respond_to(req).unwrap())
                }));
                r.route().f(allowed!(GET))
            })
            .scope("/api/v1", |api_scope| {
                api_scope
                    .resource("/list_information", |r| {
                        r.get().f(api::misc::list_information);
                        r.route().f(allowed!(GET))
                    })
                    .nested("/users", |user_scope| {
                        user_scope
                            .resource("/", |r| {
                                r.get().f(wrap(api::user::paginate));
                                r.route().f(allowed!(GET))
                            })
                            .resource("/{user_id}/", |r| {
                                r.get().f(wrap(api::user::get));
                                r.method(Method::PATCH).f(wrap(api::user::patch));
                                r.delete().f(wrap(api::user::delete));
                                r.route().f(allowed!(GET, PATCH, DELETE))
                            })
                    })
                    /*.resource("/gd/levels/{level_id}/", |r| { // FIXME: requires to fix the serde support in GDCF
                        r.get().f(|req| {
                            use crate::actor::gdcf::LevelById;
                            use actix_web::{AsyncResponder, HttpResponse};
                            use tokio::prelude::{Future, IntoFuture};

                            let state = req.state().clone();

                            Path::<u64>::extract(req)
                                .into_future()
                                .map_err(|_| PointercrateError::bad_request("very bad"))
                                .and_then(move |position| {
                                    state
                                        .gdcf
                                        .send(LevelById(position.into_inner()))
                                        .map_err(PointercrateError::internal)
                                })
                                .map(|resource| HttpResponse::Ok().json(resource))
                                .responder()
                        })
                    })*/
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
                    .nested("/players", |player_scope| {
                        player_scope
                            .resource("/", |r| {
                                r.get().f(wrap(api::player::paginate));
                                r.route().f(allowed!(GET))
                            })
                            .resource("/{player_id}/", |r| {
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
                                r.route().f(allowed!(GET, POST))
                            })
                            .resource("/{record_id}/", |r| {
                                r.get().f(wrap(api::record::get));
                                r.delete().f(wrap(api::record::delete));
                                r.method(Method::PATCH).f(wrap(api::record::patch));
                                r.route().f(allowed!(GET, DELETE, PATCH))
                            })
                    })
                    .nested("/submitters", |record_scope| {
                        record_scope
                            .resource("/", |r| {
                                r.get().f(wrap(api::submitter::paginate));
                                r.route().f(allowed!(GET))
                            })
                            .resource("/{submitter_id}/", |r| {
                                r.get().f(wrap(api::submitter::get));
                                r.method(Method::PATCH).f(wrap(api::submitter::patch));
                                r.route().f(allowed!(GET, PATCH))
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
                r.route()
                    .f(|req| crate::api::error(req, PointercrateError::NotFound))
            })
    };

    let port = match std::env::var("PORT") {
        Ok(port) => port.parse().unwrap_or(8088),
        _ => 8088,
    };

    server::new(app_factory)
        .bind(SocketAddr::from(([127, 0, 0, 1], port)))
        .unwrap()
        .run();
}
