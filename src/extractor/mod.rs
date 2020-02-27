//! Module containing custom [`actix_web::FromRequest`] impls

use crate::state::PointercrateState;
use actix_web::{
    dev::{Payload, PayloadStream},
    FromRequest, HttpRequest,
};
use futures::future::{ok, Ready};

pub mod auth;
pub mod if_match;
pub mod ip;

impl FromRequest for PointercrateState {
    type Config = ();
    type Error = ();
    type Future = Ready<Result<PointercrateState, ()>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload<PayloadStream>) -> Self::Future {
        ok(req.app_data::<PointercrateState>().unwrap().clone())
    }
}
