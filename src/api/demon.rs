//! Module containing all the actix request handlers for the `/api/v1/demons/` endpoints

use crate::{
    actor::database::{PaginateMessage, TokenAuth},
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::{
        demon::{Demon, DemonPagination, PartialDemon, PatchDemon, PostDemon},
        user::User,
    },
    state::PointercrateState,
};
use actix_web::{
    AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path, Responder,
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
        .map(|demon: Demon| HttpResponse::Created().json_with_etag(demon))
        .responder()
}

/// `GET /api/v1/demons/[id]/` handler
pub fn get(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("GET /api/v1/demons/{{position}}/");

    let state = req.state().clone();

    Path::<i16>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Demon position must be integer"))
        .into_future()
        .and_then(move |position| state.get(position.into_inner()))
        .map(|demon: Demon| HttpResponse::Ok().json_with_etag(demon))
        .responder()
}

/// `PATCH /api/v1/users/[id]/` handler
pub fn patch(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("PATCH /api/v1/demons/{{position}}/");

    let state = req.state().clone();
    let if_match = req.extensions_mut().remove().unwrap();
    let position = Path::<i16>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Demon position must be integer"));

    let body = req.json();

    state
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .and_then(move |user: User| {
            Ok((
                demand_perms!(user, ListModerator or ListAdministrator),
                position?,
            ))
        })
        .and_then(move |(user, position)| {
            body.from_err().and_then(move |patch: PatchDemon| {
                state.patch(user, position.into_inner(), patch, if_match)
            })
        })
        .map(|updated: Demon| HttpResponse::Ok().json_with_etag(updated))
        .responder()
}
