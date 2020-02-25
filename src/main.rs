// TODO: set up lint denys

use crate::{error::PointercrateError, middleware::headers::Headers, model::user::UserPagination, state::PointercrateState};
use actix_web::{web::scope, App, HttpServer};
use std::net::SocketAddr;

#[macro_use]
mod util;
mod api;
mod cistring;
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

pub type Result<T> = std::result::Result<T, PointercrateError>;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().expect("Failed to initialize .env file!");

    let application_state = PointercrateState::initialize().await;

    // TODO: error handler
    // TODO: json config
    // TODO: 404 and 405 handling
    // TODO: logging

    HttpServer::new(move || {
        App::new()
            .wrap(Headers)
            .app_data(application_state.clone())
            .service(
                scope("/api/v1")
                    .service(
                        scope("/auth")
                            .service(api::auth::register)
                            .service(api::auth::delete_me)
                            .service(api::auth::get_me)
                            .service(api::auth::invalidate)
                            .service(api::auth::login)
                            .service(api::auth::patch_me),
                    )
                    .service(
                        scope("/users")
                            .service(api::user::paginate)
                            .service(api::user::get)
                            .service(api::user::delete)
                            .service(api::user::patch),
                    )
                    .service(
                        scope("/submitters")
                            .service(api::demonlist::submitter::get)
                            .service(api::demonlist::submitter::paginate)
                            .service(api::demonlist::submitter::patch),
                    )
                    .service(
                        scope("/demons")
                            .service(api::demonlist::demon::v1::get)
                            .service(api::demonlist::demon::v1::paginate)
                            .service(api::demonlist::demon::v1::patch)
                            .service(api::demonlist::demon::v1::delete_creator)
                            .service(api::demonlist::demon::v1::post_creator)
                            .service(api::demonlist::demon::post),
                    )
                    .service(
                        scope("/records")
                            .service(api::demonlist::record::delete)
                            .service(api::demonlist::record::get)
                            .service(api::demonlist::record::paginate)
                            .service(api::demonlist::record::patch)
                            .service(api::demonlist::record::submit),
                    ),
            )
            .service(
                scope("/api/v2").service(
                    scope("/demons")
                        .service(api::demonlist::demon::v2::get)
                        .service(api::demonlist::demon::v2::paginate)
                        .service(api::demonlist::demon::v2::patch)
                        .service(api::demonlist::demon::v2::delete_creator)
                        .service(api::demonlist::demon::v2::post_creator)
                        .service(api::demonlist::demon::post),
                ),
            )
    })
    .bind(SocketAddr::from(([127, 0, 0, 1], config::port())))?
    .run()
    .await?;

    Ok(())
}
