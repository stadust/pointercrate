//! Module containg the actual actix request handlers
pub mod auth;
pub mod demonlist;
pub mod user;

use crate::{
    error::PointercrateError,
    middleware::mime::Accept,
    state::PointercrateState,
    view::{error::ErrorPage, Page},
    Result,
};
use actix_web::{error::ResponseError, HttpRequest, HttpResponse};
use log::warn;
use mime;
use tokio::prelude::future::Future;

pub type PCResponder = Box<dyn Future<Item = HttpResponse, Error = PointercrateError>>;

pub fn wrap<F>(handler: F) -> impl Fn(&HttpRequest<PointercrateState>) -> PCResponder
where
    F: Fn(&HttpRequest<PointercrateState>) -> PCResponder + 'static,
{
    move |req: &HttpRequest<PointercrateState>| -> Box<dyn Future<Item = HttpResponse, Error = PointercrateError>> {
        let req_clone = req.clone();

        Box::new(handler(req).or_else(move |error| handle_error(&req_clone, error)))
    }
}

pub fn wrap_direct(
    handler: impl Fn(&HttpRequest<PointercrateState>) -> Result<HttpResponse>,
) -> impl Fn(&HttpRequest<PointercrateState>) -> Result<HttpResponse> {
    move |req: &HttpRequest<PointercrateState>| -> Result<HttpResponse> {
        handler(req).or_else(|error| handle_error(req, error))
    }
}

fn handle_error(
    req: &HttpRequest<PointercrateState>,
    error: PointercrateError,
) -> Result<HttpResponse> {
    warn!("HTTP Error returned during request handling: {}", error);

    let mime_type = preferred_mime_type(req)?;

    let response = match (mime_type.type_(), mime_type.subtype()) {
        (mime::TEXT, mime::HTML) => {
            let html = ErrorPage::new(&error).render(&req);

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .status(error.status_code())
                .body(html.0)
        },
        (mime::APPLICATION, mime::JSON) => error.error_response(),
        _ => unreachable!(),
    };

    Ok(response)
}

fn preferred_mime_type(req: &HttpRequest<PointercrateState>) -> Result<mime::Mime> {
    let Accept(accepted) = req.extensions_mut().remove().unwrap();

    let (preference, mime_type) = accepted
        .into_iter()
        .filter(|mime| {
            match (mime.type_(), mime.subtype()) {
                (mime::TEXT, mime::HTML) | (mime::APPLICATION, mime::JSON) => true,
                _ => false,
            }
        })
        .map(|mime| {
            (
                mime.get_param("q")
                    .map(|q| q.as_str().parse::<f32>().unwrap_or(-1.0))
                    .unwrap_or(1.0),
                mime,
            )
        })
        .max_by_key(|(q, _)| (q * 100.0) as i32)  // cannot compare floats dammit
        .unwrap_or((1.0, mime::TEXT_HTML));

    if preference < 0.0 || preference > 1.0 {
        Err(PointercrateError::InvalidHeaderValue { header: "Accept" })
    } else {
        Ok(mime_type)
    }
}

/// Specialized form of crate::api::wrap that doesnt have to deal with
/// calling another handler function and thus doesnt have to bother with
/// futures
pub fn error(
    req: &HttpRequest<PointercrateState>,
    error: PointercrateError,
) -> Result<HttpResponse> {
    handle_error(req, error)
}
