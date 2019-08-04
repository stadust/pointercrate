//! Module containing all the actix request handlers for the `/api/v1/records/` endpoints

use crate::{
    actor::demonlist::PostProcessRecord,
    api::PCResponder,
    error::PointercrateError,
    middleware::{auth::Token, cond::HttpResponseBuilderExt},
    model::demonlist::record::{PatchRecord, Record, RecordPagination, Submission},
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};

use log::info;
use tokio::prelude::future::{Future, IntoFuture};

/// `GET /api/v1/records/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/records/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let req = req.clone();

    pagination
        .into_future()
        .and_then(move |pagination: RecordPagination| {
            req.state().paginate::<Token, Record, _>(
                &req,
                pagination,
                "/api/v1/records/".to_string(),
            )
        })
        .map(|(players, links)| HttpResponse::Ok().header("Links", links).json(players))
        .responder()
}

/// `POST /api/v1/records/` handler
pub fn submit(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/records/");

    let req = req.clone();

    req.json()
        .from_err()
        .and_then(move |submission: Submission| {
            req.state()
                .post::<Token, _, _>(&req, submission)
                .and_then(move |record: Option<Record>| req.state().http(PostProcessRecord(record)))
        })
        .map(|record: Option<Record>| {
            match record {
                Some(record) => HttpResponse::Created().json_with_etag(record),
                None => HttpResponse::NoContent().finish(),
            }
        })
        .responder()
}

get_handler!("/api/v1/records/[record_id]/", i32, "Record ID", Record);
patch_handler!(
    "/api/v1/records/[record id]/",
    i32,
    "Record ID",
    PatchRecord,
    Record
);
delete_handler!("/api/v1/records/[record id]/", i32, "Record ID", Record);
