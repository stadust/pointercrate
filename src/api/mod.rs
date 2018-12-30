//! Module containg the actualy actix request handlers
pub mod auth;
pub mod demon;
pub mod record;
pub mod user;

use crate::{error::PointercrateError, state::PointercrateState};
use actix_web::{HttpRequest, HttpResponse};
use tokio::prelude::future::Future;

pub type PCResponder = Box<dyn Future<Item = HttpResponse, Error = PointercrateError>>;

pub fn wrap<F>(handler: F)
where
    F: Fn(&HttpRequest<PointercrateState>) -> PCResponder,
{

}
