use actix_web::{AsyncResponder, HttpMessage, HttpRequest, HttpResponse, Responder};
use crate::{
    actor::database::Register, error::PointercrateError, model::user::Registration,
    state::PointercrateState,
};
use log::info;
use tokio::prelude::future::Future;

pub fn register(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("POST /api/v1/auth/register/");

    let database = req.state().database.clone();

    req.json()
        .from_err()
        .and_then(move |registration: Registration| {
            database
                .send(Register(registration))
                .map_err(PointercrateError::internal)
        }).map(|user| HttpResponse::Ok().json(user)) // TODO: Set 'Location' header
        .responder()
}
