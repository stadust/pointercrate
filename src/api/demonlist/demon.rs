//! Module containing all the actix request handlers for the `/api/v1/demons/` endpoints

use crate::{
    api::PCResponder,
    error::PointercrateError,
    middleware::{auth::Token, cond::HttpResponseBuilderExt},
    model::demonlist::{
        creator::{Creator, PostCreator},
        demon::{DemonIdPagination, DemonPagination, FullDemon, PatchDemon, PostDemon},
        Demon,
    },
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

pub mod v1 {
    use super::*;

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
                req.state().paginate::<Token, Demon, _>(
                    &req,
                    pagination,
                    "/api/v1/demons/".to_string(),
                )
            })
            .map(|(demons, links)| HttpResponse::Ok().header("Links", links).json(demons))
            .responder()
    }
    get_handler!(
        "/api/v1/demons/[position]/",
        i16,
        "Demon position",
        FullDemon
    );
    patch_handler!(
        "/api/v1/demons/[position]/",
        i16,
        "Demon position",
        PatchDemon,
        FullDemon
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
            .map_err(|_| {
                PointercrateError::bad_request("Demon position and player ID must be integer")
            })
            .into_future()
            .and_then(move |resource_id| {
                req.state()
                    .delete::<Token, (i16, i32), Creator>(&req, resource_id.into_inner())
            })
            .map(|_| HttpResponse::NoContent().finish())
            .responder()
    }
}

pub mod v2 {
    use super::*;

    /// `GET /api/v1/demons/` handler
    pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
        info!("GET /api/v2/demons/");

        let query_string = req.query_string();
        let pagination = serde_urlencoded::from_str(query_string)
            .map_err(|err| PointercrateError::bad_request(&err.to_string()));

        let req = req.clone();

        pagination
            .into_future()
            .and_then(move |pagination: DemonIdPagination| {
                req.state().paginate::<Token, Demon, _>(
                    &req,
                    pagination,
                    "/api/v1/demons/".to_string(),
                )
            })
            .map(|(demons, links)| HttpResponse::Ok().header("Links", links).json(demons))
            .responder()
    }
    get_handler!("/api/v2/demons/[id]/", i32, "Demon ID", FullDemon);
    patch_handler!(
        "/api/v1/demons/[id]/",
        i32,
        "Demon ID",
        PatchDemon,
        FullDemon
    );

    pub fn post_creator(req: &HttpRequest<PointercrateState>) -> PCResponder {
        info!("POST /api/v2/demons/{{id}}/creators/");

        let req = req.clone();
        let position = Path::<i32>::extract(&req)
            .map_err(|_| PointercrateError::bad_request("Demon ID must be integer"));

        req.json()
            .from_err()
            .and_then(move |post: PostCreator| Ok((position?.into_inner(), post.creator)))
            .and_then(move |data| req.state().post::<Token, _, _>(&req, data))
            .map(|_: Creator| HttpResponse::Created().finish())
            .responder()
    }

    pub fn delete_creator(req: &HttpRequest<PointercrateState>) -> PCResponder {
        info!("DELETE /api/v2/demons/[demon id]/creators/[player id]/");

        let req = req.clone();

        Path::<(i32, i32)>::extract(&req)
            .map_err(|_| PointercrateError::bad_request("Demon ID and player ID must be integer"))
            .into_future()
            .and_then(move |resource_id| {
                req.state()
                    .delete::<Token, (i32, i32), Creator>(&req, resource_id.into_inner())
            })
            .map(|_| HttpResponse::NoContent().finish())
            .responder()
    }
}

post_handler!("/api/v1/demons/", PostDemon, FullDemon);
