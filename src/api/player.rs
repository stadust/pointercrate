use super::PCResponder;
use crate::{
    error::PointercrateError,
    middleware::{auth::Token, cond::HttpResponseBuilderExt},
    model::player::{
        PatchPlayer, PlayerPagination, PlayerWithDemonsAndRecords, RankedPlayer2,
        RankingPagination, ShortPlayer,
    },
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
    let auth = req.extensions_mut().remove().unwrap();

    pagination
        .into_future()
        .and_then(move |pagination: PlayerPagination| {
            state.paginate::<Token, ShortPlayer, _>(
                pagination,
                "/api/v1/players/".to_string(),
                auth,
            )
        })
        .map(|(players, links)| HttpResponse::Ok().header("Links", links).json(players))
        .responder()
}

/// `GET /api/v1/players/ranking` handler
pub fn ranking(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/players/ranking/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();
    let auth = req.extensions_mut().remove().unwrap();

    pagination
        .into_future()
        .and_then(move |pagination: RankingPagination| {
            state.paginate::<Token, RankedPlayer2, _>(
                pagination,
                "/api/v1/players/ranking/".to_string(),
                auth,
            )
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

patch_handler!(
    "/api/v1/players/[id]/",
    i32,
    "Player ID",
    PatchPlayer,
    PlayerWithDemonsAndRecords
);
