//! Module containing all the actix request handlers for the `/api/v1/submitters/` endpoints

use crate::{
    api::PCResponder,
    error::PointercrateError,
    middleware::{auth::Token, cond::HttpResponseBuilderExt},
    model::demonlist::submitter::{FullSubmitter, PatchSubmitter, Submitter, SubmitterPagination},
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

/// `GET /api/v1/users/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/submitters/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let req = req.clone();

    pagination
        .into_future()
        .and_then(move |pagination: SubmitterPagination| {
            req.state().paginate::<Token, Submitter, _>(
                &req,
                pagination,
                "/api/v1/submitters/".to_owned(),
            )
        })
        .map(|(users, links)| HttpResponse::Ok().header("Links", links).json(users))
        .responder()
}

get_handler!(
    "/api/v1/submitters/[id]",
    i32,
    "Submitter ID",
    FullSubmitter
);
patch_handler!(
    "/api/v1/submitters/[id]/",
    i32,
    "Submitter ID",
    PatchSubmitter,
    FullSubmitter
);
