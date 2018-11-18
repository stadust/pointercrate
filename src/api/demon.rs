use actix_web::{
    AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path, Responder,
};
use crate::{
    actor::database::{PaginateMessage, TokenAuth},
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::demon::{Demon, DemonPagination, PartialDemon},
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
        .and_then(move |pagination: DemonPagination| state.paginate::<PartialDemon, _>(pagination))
        .map(|(demons, links)| HttpResponse::Ok().header("Links", links).json(demons))
        .responder()
}
