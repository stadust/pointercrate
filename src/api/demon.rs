//! Module containing all the actix request handlers for the `/api/v1/demons/` endpoints

use super::PCResponder;
use crate::{
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::{
        creator::{Creator, PostCreator},
        demon::{Demon, DemonPagination, PartialDemon, PatchDemon, PostDemon},
    },
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

/// `GET /api/v1/demons/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
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
pub fn post(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/demons/");

    let auth = req.extensions_mut().remove().unwrap();
    let state = req.state().clone();

    req.json()
        .from_err()
        .and_then(move |demon: PostDemon| state.post_authorized(demon, auth))
        .map(|demon: Demon| HttpResponse::Created().json_with_etag(demon))
        .responder()
}

/// `GET /api/v1/demons/[position]/` handler
pub fn get(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/demons/{{position}}/");

    let state = req.state().clone();

    Path::<i16>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Demon position must be integer"))
        .into_future()
        .and_then(move |position| state.get(position.into_inner()))
        .map(|demon: Demon| HttpResponse::Ok().json_with_etag(demon))
        .responder()
}

/// `PATCH /api/v1/demons/[position]/` handler
pub fn patch(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("PATCH /api/v1/demons/{{position}}/");

    let state = req.state().clone();
    let if_match = req.extensions_mut().remove().unwrap();
    let auth = req.extensions_mut().remove().unwrap();
    let position = Path::<i16>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Demon position must be integer"));

    req.json()
        .from_err()
        .and_then(move |patch: PatchDemon| Ok((patch, position?.into_inner())))
        .and_then(move |(patch, position)| state.patch_authorized(auth, position, patch, if_match))
        .map(|updated: Demon| HttpResponse::Ok().json_with_etag(updated))
        .responder()
}

pub fn post_creator(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/demons/{{position}}/creators/");

    let state = req.state().clone();
    let auth = req.extensions_mut().remove().unwrap();
    let position = Path::<i16>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Demon position must be integer"));

    req.json()
        .from_err()
        .and_then(move |post: PostCreator| Ok((position?.into_inner(), post.creator)))
        .and_then(move |data| state.post_authorized(data, auth))
        .map(|_: Creator| HttpResponse::Created().finish())
        .responder()
}

pub fn delete_creator(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("DELETE /api/v1/demons/{{position}}/creators/{{player_id}}/");

    let auth = req.extensions_mut().remove().unwrap();
    let state = req.state().clone();

    Path::<(i16, i32)>::extract(req)
        .map_err(|_| {
            PointercrateError::bad_request("Demons position and player ID must be intergers")
        })
        .into_future()
        .and_then(move |data| state.delete_authorized::<_, Creator>(data.into_inner(), None, auth))
        .map(|_| HttpResponse::NoContent().finish())
        .responder()
}
