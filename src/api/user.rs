//! Module containing all the actix request handlers for the `/api/v1/users/` endpoints

use super::PCResponder;
use crate::{
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::user::{PatchUser, User, UserPagination},
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

/// `GET /api/v1/users/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/users/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();

    state
        .authorize(req.extensions_mut().remove().unwrap(), perms!(Moderator))
        .and_then(move |_| pagination)
        .and_then(move |pagination: UserPagination| state.paginate::<User, _>(pagination))
        .map(|(users, links)| HttpResponse::Ok().header("Links", links).json(users))
        .responder()
}

/// `GET /api/v1/users/[id]/` handler
pub fn user(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/users/{{user_id}}/");

    let state = req.state().clone();
    let auth = req.extensions_mut().remove().unwrap();

    let user_id = Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("User ID must be integer"));

    user_id
        .into_future()
        .and_then(move |user_id| state.get_authorized(user_id.into_inner(), auth))
        .map(|user: User| HttpResponse::Ok().json_with_etag(user))
        .responder()
}

/// `PATCH /api/v1/users/[id]/` handler
pub fn patch(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("PATCH /api/v1/users/{{user_id}}/");

    let state = req.state().clone();
    let if_match = req.extensions_mut().remove().unwrap();
    let auth = req.extensions_mut().remove().unwrap();
    let user_id = Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("User ID must be integer"));

    req.json()
        .from_err()
        .and_then(move |patch: PatchUser| Ok((patch, user_id?.into_inner())))
        .and_then(move |(patch, user_id)| state.patch_authorized(auth, user_id, patch, if_match))
        .map(|updated: User| HttpResponse::Ok().json_with_etag(updated))
        .responder()
}

/// `DELETE /api/v1/users/[id]/` handler
pub fn delete(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("DELETE /api/v1/users/{{user_id}}/");

    let state = req.state().clone();
    let if_match = req.extensions_mut().remove().unwrap();
    let auth = req.extensions_mut().remove().unwrap();

    Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("User ID must be interger"))
        .into_future()
        .and_then(move |user_id| {
            state.delete_authorized::<i32, User>(user_id.into_inner(), Some(if_match), auth)
        })
        .map(|_| HttpResponse::NoContent().finish())
        .responder()
}
