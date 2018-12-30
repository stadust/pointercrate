//! Module containg the actualy actix request handlers
pub mod auth;
pub mod demon;
pub mod record;
pub mod user;

use crate::error::PointercrateError;
use actix_web::HttpResponse;
use tokio::prelude::future::Future;

pub type PCResponder = Box<dyn Future<Item = HttpResponse, Error = PointercrateError>>;
