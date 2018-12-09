//! Module containing all the actix request handlers for the `/api/v1/demons/` endpoints

use actix_web::{
    AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path, Responder,
};
use crate::{
    actor::database::{PaginateMessage, TokenAuth},
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::demon::{Demon, DemonPagination, PartialDemon, PostDemon},
    state::PointercrateState,
};
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

/// `GET /api/v1/demons/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("GET /api/v1/demons/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();

    pagination
        .into_future()
        .and_then(move |pagination: DemonPagination| state.paginate::<PartialDemon, _>(pagination))
        .map(|(demons, links)| HttpResponse::Ok().header("Links", links).json(demons))
        .responder()
}

/// `POST /api/v1/demons/` handler
pub fn post(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("POST /api/v1/demons/");

    let state = req.state().clone();
    let json = req.json().from_err();

    state
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .and_then(|user| Ok(demand_perms!(user, ListModerator)))
        .and_then(|_| json)
        .and_then(move |demon: PostDemon| state.post(demon))
        .map(|demon: Demon| HttpResponse::Ok().json_with_etag(demon))
        .responder()
}
