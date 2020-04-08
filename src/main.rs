#![feature(proc_macro_hygiene)]
#![allow(non_upper_case_globals)]
#![deny(unused_imports)]

use crate::{
    error::{HtmlError, JsonError, PointercrateError},
    middleware::etag::Etag,
    state::PointercrateState,
};
use actix_files::{Files, NamedFile};
use actix_web::{
    middleware::{Logger, NormalizePath},
    web,
    web::{route, scope, JsonConfig, PathConfig, QueryConfig},
    App, HttpRequest, HttpServer,
};
use api::{
    auth,
    demonlist::{demon, misc, player, record, submitter},
    user,
};
use std::net::SocketAddr;

#[macro_use]
mod util;
mod api;
mod cistring;
mod compat;
mod config;
mod documentation;
mod error;
mod extractor;
mod middleware;
mod model;
mod permissions;
mod ratelimit;
mod state;
mod video;
mod view;

#[cfg(test)]
mod test;

pub type Result<T> = std::result::Result<T, PointercrateError>;

pub type ApiResult<T> = std::result::Result<T, JsonError>;
pub type ViewResult<T> = std::result::Result<T, HtmlError>;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().expect("Failed to initialize .env file!");

    let application_state = PointercrateState::initialize().await;

    HttpServer::new(move || {
        let json_config =
            JsonConfig::default().error_handler(|error, request| PointercrateError::from(error).dynamic(request.headers()).into());
        let path_config =
            PathConfig::default().error_handler(|error, request| PointercrateError::from(error).dynamic(request.headers()).into());
        let query_config =
            QueryConfig::default().error_handler(|error, request| PointercrateError::from(error).dynamic(request.headers()).into());

        App::new()
            .app_data(json_config)
            .app_data(path_config)
            .app_data(query_config)
            .wrap(Etag)
            .wrap(Logger::default())
            .wrap(NormalizePath::default())
            .app_data(application_state.clone())
            .service(Files::new("/static2", "./static2").use_etag(true))
            .route(
                "/robots.txt",
                web::get().to(|req: HttpRequest| NamedFile::open("robots.txt").unwrap().into_response(&req).unwrap()),
            )
            .service(view::home::index)
            .service(view::login::index)
            .service(view::login::post)
            .service(view::login::register)
            .service(view::demonlist::page)
            .service(view::demonlist::index)
            .service(view::account::index)
            .service(view::documentation::index)
            .service(view::documentation::topic)
            .service(
                scope("/api/v1")
                    .service(misc::list_information)
                    .service(
                        scope("/auth")
                            .service(auth::register)
                            .service(auth::delete_me)
                            .service(auth::get_me)
                            .service(auth::invalidate)
                            .service(auth::login)
                            .service(auth::patch_me),
                    )
                    .service(
                        scope("/users")
                            .service(user::paginate)
                            .service(user::get)
                            .service(user::delete)
                            .service(user::patch),
                    )
                    .service(
                        scope("/submitters")
                            .service(submitter::get)
                            .service(submitter::paginate)
                            .service(submitter::patch),
                    )
                    .service(
                        scope("/demons")
                            .service(demon::v1::get)
                            .service(demon::v1::paginate)
                            .service(demon::v1::patch)
                            .service(demon::v1::delete_creator)
                            .service(demon::v1::post_creator)
                            .service(demon::post),
                    )
                    .service(
                        scope("/records")
                            .service(record::delete)
                            .service(record::get)
                            .service(record::paginate)
                            .service(record::patch)
                            .service(record::submit)
                            .service(record::add_note)
                            .service(record::patch_note)
                            .service(record::delete_note),
                    )
                    .service(
                        scope("/players")
                            .service(player::patch)
                            .service(player::paginate)
                            .service(player::ranking)
                            .service(player::get),
                    ),
            )
            .service(
                scope("/api/v2").service(
                    scope("/demons")
                        .service(demon::v2::paginate_listed)
                        .service(demon::v2::get)
                        .service(demon::v2::paginate)
                        .service(demon::v2::patch)
                        .service(demon::v2::delete_creator)
                        .service(demon::v2::post_creator)
                        .service(demon::post),
                ),
            )
            .default_service(route().to(api::handle_404_or_405))
    })
    .bind(SocketAddr::from(([127, 0, 0, 1], config::port())))?
    .run()
    .await?;

    Ok(())
}
