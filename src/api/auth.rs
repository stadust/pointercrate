//! Module containing all the actix request handlers for the `/api/v1/auth/` endpoints

use super::PCResponder;
use crate::{
    actor::database::{BasicAuth, Invalidate, TokenAuth},
    middleware::cond::HttpResponseBuilderExt,
    model::user::{PatchMe, Registration, User},
    state::PointercrateState,
};
use actix_web::{AsyncResponder, HttpMessage, HttpRequest, HttpResponse};
use log::info;
use serde_json::json;
use tokio::prelude::future::Future;

/// `POST /api/v1/auth/register/` handler
pub fn register(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/auth/register/");

    let state = req.state().clone();

    req.json()
        .from_err()
        .and_then(move |registration: Registration| state.post(registration))
        .map(|user: User| {
            HttpResponse::Created()
                .header("Location", "/api/v1/auth/me/")
                .json_with_etag(user)
        })
        .responder()
}

/// `POST /api/v1/auth/` handler
pub fn login(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/auth/");

    req.state()
        .database(BasicAuth(req.extensions_mut().remove().unwrap()))
        .map(|user: User| {
            HttpResponse::Ok().etag(&user).json(json!({
                "data": user,
                "token": user.generate_token()
            }))
        })
        .responder()
}

/// `POST /api/v1/auth/invalidate/` handler
pub fn invalidate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/auth/invalidate/");

    req.state()
        .database(Invalidate(req.extensions_mut().remove().unwrap()))
        .map(|_| HttpResponse::NoContent().finish())
        .responder()
}

/// `GET /api/v1/auth/me/` handler
pub fn me(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/auth/me/");

    req.state()
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .map(|user: User| HttpResponse::Ok().json_with_etag(user))
        .responder()
}

/// `PATCH /api/v1/auth/me/` handler
pub fn patch_me(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("PATCH /api/v1/auth/me/");

    let state = req.state().clone();
    let auth = req.extensions_mut().remove().unwrap();
    let if_match = req.extensions_mut().remove().unwrap();

    req.json()
        .from_err()
        .and_then(move |patch: PatchMe| {
            state.database(BasicAuth(auth)).and_then(move |user: User| {
                let user_id = user.id; // AAA silly moving rules are silly
                state.patch(user, user_id, patch, if_match)
            })
        })
        .map(|user: User| HttpResponse::Ok().json_with_etag(user))
        .responder()
}

/// `DELETE /api/v1/auth/me/` handler
pub fn delete_me(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("DELETE /api/v1/auth/me/");

    let state = req.state().clone();
    let if_match = req.extensions_mut().remove().unwrap();

    state
        .database(BasicAuth(req.extensions_mut().remove().unwrap()))
        .and_then(move |user: User| state.delete::<i32, User>(user.id, if_match))
        .map(|_| HttpResponse::NoContent().finish())
        .responder()
}
