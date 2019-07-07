//! Module containing all the actix request handlers for the `/api/v1/demons/` endpoints

use super::PCResponder;
use crate::{
    error::PointercrateError,
    middleware::{auth::Token, cond::HttpResponseBuilderExt},
    model::{
        creator::{Creator, PostCreator},
        demon::{
            DemonPagination, DemonWithCreatorsAndRecords, PartialDemon, PatchDemon, PostDemon,
        },
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

    let req = req.clone();

    pagination
        .into_future()
        .and_then(move |pagination: DemonPagination| {
            req.state().paginate::<Token, PartialDemon, _>(
                &req,
                pagination,
                "/api/v1/demons/".to_string(),
            )
        })
        .map(|(demons, links)| HttpResponse::Ok().header("Links", links).json(demons))
        .responder()
}

post_handler!("/api/v1/demons/", PostDemon, DemonWithCreatorsAndRecords);
get_handler!(
    "/api/v1/demons/[position]/",
    i16,
    "Demon position",
    DemonWithCreatorsAndRecords
);
patch_handler!(
    "/api/v1/demons/[position]/",
    i16,
    "Demon position",
    PatchDemon,
    DemonWithCreatorsAndRecords
);

pub fn post_creator(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/demons/{{position}}/creators/");

    let req = req.clone();
    let position = Path::<i16>::extract(&req)
        .map_err(|_| PointercrateError::bad_request("Demon position must be integer"));

    req.json()
        .from_err()
        .and_then(move |post: PostCreator| Ok((position?.into_inner(), post.creator)))
        .and_then(move |data| req.state().post::<Token, _, _>(&req, data))
        .map(|_: Creator| HttpResponse::Created().finish())
        .responder()
}

pub fn delete_creator(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("DELETE /api/v1/demons/[position]/creators/[player id]/");

    let req = req.clone();

    Path::<(i16, i32)>::extract(&req)
        .map_err(|_| PointercrateError::bad_request("Demon position and player ID must be integer"))
        .into_future()
        .and_then(move |resource_id| {
            req.state()
                .delete::<Token, (i16, i32), Creator>(&req, resource_id.into_inner())
        })
        .map(|_| HttpResponse::NoContent().finish())
        .responder()
}
