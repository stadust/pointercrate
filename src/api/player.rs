use super::PCResponder;
use crate::{
    error::PointercrateError,
    middleware::{auth::Token, cond::HttpResponseBuilderExt},
    model::player::{PatchPlayer, Player, PlayerPagination, PlayerWithDemonsAndRecords},
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

/// `GET /api/v1/players/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/players/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();

    state
        /*.authorize(
            req.extensions_mut().remove().unwrap(),
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
        )*/
        .auth::<Token>(req.extensions_mut().remove().unwrap()) // TODO: pagination permissions thingy
        .and_then(move |_| pagination)
        .and_then(move |pagination: PlayerPagination| {
            state.paginate::<Player, _>(pagination, "/api/v1/players/".to_string())
        })
        .map(|(players, links)| HttpResponse::Ok().header("Links", links).json(players))
        .responder()
}

get_handler!(
    "/api/v1/players/[id]/",
    i32,
    "Player ID",
    PlayerWithDemonsAndRecords
);

patch_handler_with_authorization!(
    "/api/v1/players/[id]/",
    i32,
    "Player ID",
    PatchPlayer,
    PlayerWithDemonsAndRecords
);
