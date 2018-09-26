use actix_web::{AsyncResponder, HttpMessage, HttpRequest, HttpResponse, Responder};
use crate::{
    actor::database::Register,
    model::user::{Registration, User},
    state::PointercrateState,
};
use log::info;
use tokio::prelude::future::Future;

pub fn register(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("POST /api/v1/auth/register/");

    let state = req.state().clone();

    req.json()
        .from_err()
        .and_then(move |registration: Registration| state.database(Register(registration)))
        .map(|user: User| {
            HttpResponse::Created()
                .header("Location", "/auth/me/")
                .json(user)
        }).responder()
}
