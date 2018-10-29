use actix_web::{
    AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path, Responder,
};
use crate::{
    actor::database::{DeleteUserById, Paginate, Patch, TokenAuth, UserById},
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::demon::{Demon, DemonPagination, PatchDemon},
    state::PointercrateState,
};
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

pub fn paginate(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("GET /api/v1/demons/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();

    pagination
        .into_future()
        .and_then(move |pagination: DemonPagination| state.database(Paginate(pagination)))
        .map(|demons| HttpResponse::Ok().json(demons))
        .responder()
}
