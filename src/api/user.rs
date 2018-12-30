//! Module containing all the actix request handlers for the `/api/v1/users/` endpoints

use super::PCResponder;
use crate::{
    actor::database::TokenAuth,
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::user::{PatchUser, User, UserPagination},
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use log::info;
use tokio::prelude::future::Future;

/// `GET /api/v1/users/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/users/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();

    state
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .and_then(|user| Ok(demand_perms!(user, Moderator)))
        .and_then(move |_| pagination)
        .and_then(move |pagination: UserPagination| state.paginate::<User, _>(pagination))
        .map(|(users, links)| HttpResponse::Ok().header("Links", links).json(users))
        .responder()
}

/// `GET /api/v1/users/[id]/` handler
pub fn user(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/users/{{user_id}}/");

    let state = req.state().clone();

    let user_id = Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("User ID must be integer"));

    state
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .and_then(|user| Ok(demand_perms!(user, Moderator)))
        .and_then(move |_| user_id)
        .and_then(move |user_id| state.get(user_id.into_inner()))
        .map(|user: User| HttpResponse::Ok().json_with_etag(user))
        .responder()
}

/// `PATCH /api/v1/users/[id]/` handler
pub fn patch(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("PATCH /api/v1/users/{{user_id}}/");

    let state = req.state().clone();
    let if_match = req.extensions_mut().remove().unwrap();
    let user_id = Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("User ID must be integer"));

    let body = req.json();

    state
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .and_then(move |user: User| Ok((demand_perms!(user, Moderator or Administrator), user_id?)))
        .and_then(move |(user, user_id)| {
            body.from_err().and_then(move |patch: PatchUser| {
                state.patch(user, user_id.into_inner(), patch, if_match)
            })
        })
        .map(|updated: User| HttpResponse::Ok().json_with_etag(updated))
        .responder()
}

/// `DELETE /api/v1/users/[id]/` handler
pub fn delete(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("DELETE /api/v1/users/{{user_id}}/");

    let state = req.state().clone();
    let if_match = req.extensions_mut().remove().unwrap();
    let user_id = Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("User ID must be interger"));

    state
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .and_then(|user: User| Ok(demand_perms!(user, Administrator)))
        .and_then(move |_| user_id)
        .and_then(move |user_id| {
            state
                .delete::<i32, User>(user_id.into_inner(), if_match)
                .map(|_| HttpResponse::NoContent().finish())
        })
        .responder()
}
