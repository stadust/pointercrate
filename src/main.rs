// TODO: set up lint denys

use crate::{error::PointercrateError, middleware::ip::IpResolve, state::PointercrateState};
use actix_web::{App, HttpServer};
use std::net::SocketAddr;

#[macro_use]
mod util;
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

    HttpServer::new(move || App::new().wrap(IpResolve).app_data(application_state.clone()))
        .bind(SocketAddr::from(([127, 0, 0, 1], config::port())))?
        .run()
        .await?;

    /*let mut connection = application_state.connection_pool.acquire().await.unwrap();

    struct Test {
        notes: Option<String>,
    }

    let notes = sqlx::query_as!(Test, "select notes from records where id = 5290")
        .fetch_one(&mut connection)
        .await
        .unwrap()
        .notes;

    println!("{:?}", notes);*/

    Ok(())
}
