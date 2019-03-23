//! Module containing all the actix request handlers for the `/api/v1/auth/` endpoints

use super::PCResponder;
use crate::{
    actor::database::{DeleteMessage, Invalidate, PatchMessage},
    context::RequestData,
    middleware::{
        auth::{Basic, Me, Token},
        cond::{HttpResponseBuilderExt, IfMatch},
    },
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
    let req = req.clone();

    req.json()
        .from_err()
        .and_then(move |registration: Registration| state.post::<Token, _, _>(&req, registration))
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
        .auth::<Basic>(req.extensions_mut().remove().unwrap())
        .map(|user| {
            HttpResponse::Ok().etag(&user).json(json!({
                "data": user,
                "token": user.0.generate_token()
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
        .auth::<Token>(req.extensions_mut().remove().unwrap())
        .map(|user| HttpResponse::Ok().json_with_etag(user))
        .responder()
}

/// `PATCH /api/v1/auth/me/` handler
pub fn patch_me(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("PATCH /api/v1/auth/me/");

    let req = req.clone();
    let auth = req.extensions_mut().remove().unwrap();

    req.json()
        .from_err()
        .and_then(move |patch: PatchMe| {
            req.state().auth::<Basic>(auth).and_then(move |user| {
                req.state().database(PatchMessage::<Me, Me, _>::new(
                    user,
                    patch,
                    RequestData::from_request(&req),
                ))
            })
        })
        .map(|user| HttpResponse::Ok().json_with_etag(user))
        .responder()
}

/// `DELETE /api/v1/auth/me/` handler
pub fn delete_me(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("DELETE /api/v1/auth/me/");

    let state = req.state().clone();
    let if_match: IfMatch = req.extensions_mut().remove().unwrap();
    let ip = req.extensions_mut().remove().unwrap();

    state
        .auth::<Basic>(req.extensions_mut().remove().unwrap())
        .and_then(move |me| {
            state.database(DeleteMessage::<Me, Me>::new(
                me,
                RequestData::new(ip).with_if_match(Some(if_match)),
            ))
        })
        .map(|_| HttpResponse::NoContent().finish())
        .responder()
}
