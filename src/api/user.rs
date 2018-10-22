use actix_web::{AsyncResponder, HttpRequest, HttpResponse, Responder};
use crate::{
    actor::database::{Paginate, TokenAuth},
    model::user::UserPagination,
    state::PointercrateState,
};
use tokio::prelude::future::Future;

pub fn paginate(req: &HttpRequest<PointercrateState>) -> impl Responder {
    let query_string = req.query_string();
    let pagination: UserPagination =
        serde_urlencoded::from_str(query_string).expect("TODO: error handling");

    let state = req.state().clone();

    state
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .and_then(|user| Ok(demand_perms!(user, Moderator)))
        .and_then(move |_| state.database(Paginate(pagination)))
        .map(|users| HttpResponse::Ok().json(users))
        .responder()
}
