// TODO: set up lint denys

use crate::{
    error::PointercrateError,
    middleware::{headers::Headers, ip::IpResolve},
    model::user::UserPagination,
    state::PointercrateState,
};
use actix_web::{web::scope, App, HttpServer};
use std::net::SocketAddr;

#[macro_use]
mod util;
mod api;
mod cistring;
mod config;
mod documentation;
mod error;
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
            .wrap(IpResolve)
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
                    ),
            )
    })
    .bind(SocketAddr::from(([127, 0, 0, 1], config::port())))?
    .run()
    .await?;

    Ok(())
}
