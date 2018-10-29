use actix_web::{
    AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path, Responder,
};
use crate::{
    actor::database::{Paginate, Patch, TokenAuth, UserById, DeleteUserById},
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::user::{PatchUser, User, UserPagination},
    state::PointercrateState,
};
use log::info;
use tokio::prelude::future::Future;

pub fn paginate(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("GET /api/v1/users/");

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

pub fn user(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("GET /api/v1/users/{{user_id}}/");

    let state = req.state().clone();

    let user_id = Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("User ID must be integer"));

    state
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .and_then(|user| Ok(demand_perms!(user, Moderator)))
        .and_then(move |_| user_id)
        .and_then(move |user_id| state.database(UserById(user_id.into_inner())))
        .map(|user: User| HttpResponse::Ok().json_with_etag(user))
        .responder()
}

pub fn patch(req: &HttpRequest<PointercrateState>) -> impl Responder {
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
                state
                    .database_if_match(UserById(user_id.into_inner()), if_match)
                    .and_then(move |target| state.database(Patch(user, target, patch)))
            })
        })
        .map(|updated: User| HttpResponse::Ok().json_with_etag(updated))
        .responder()
}

pub fn delete(req: &HttpRequest<PointercrateState>) -> impl Responder {
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
                .database_if_match(UserById(user_id.into_inner()), if_match)
                .and_then(move |user| state.database(DeleteUserById(user.id)))
                .map(|_| HttpResponse::NoContent().finish())
        })
        .responder()
}
