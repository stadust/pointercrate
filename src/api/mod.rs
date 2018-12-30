//! Module containg the actualy actix request handlers
pub mod auth;
pub mod demon;
pub mod record;
pub mod user;

use crate::{
    error::{PointercrateError, PointercrateErrorResponse},
    middleware::mime::Accept,
    state::PointercrateState,
    view::{error::ErrorPage, Page},
};
use actix_web::{HttpRequest, HttpResponse};
use mime;
use tokio::prelude::future::{Future, IntoFuture};

pub type PCResponder = Box<dyn Future<Item = HttpResponse, Error = PointercrateError>>;

// FIXME: This is kinda horrible

pub fn wrap<F>(
    handler: F,
) -> impl Fn(
    &HttpRequest<PointercrateState>,
) -> Box<dyn Future<Item = HttpResponse, Error = PointercrateErrorResponse>>
where
    F: Fn(&HttpRequest<PointercrateState>) -> PCResponder + 'static,
{
    move |req: &HttpRequest<PointercrateState>| -> Box<dyn Future<Item = HttpResponse, Error = PointercrateErrorResponse>> {
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
            .max_by_key(|(q, _)| (q * 100.0) as i32)
            .unwrap_or((1.0, mime::TEXT_HTML));

        if preference < 0.0 || preference > 1.0 {
            Box::new(Err(PointercrateErrorResponse::Default(PointercrateError::InvalidHeaderValue { header: "Accept" }))
                .into_future())
        } else {
            let req_clone = req.clone();

            Box::new(handler(req).map_err(move |pcerr| {
                match (mime_type.type_(), mime_type.subtype()) {
                    (mime::TEXT, mime::HTML) => PointercrateErrorResponse::Html(ErrorPage::new(&pcerr).render(&req_clone)),
                    _ => PointercrateErrorResponse::Default(pcerr),
                }
            }))
        }
    }
}
