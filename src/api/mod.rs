//! Module containg the actual actix request handlers
pub mod auth;
pub mod demon;
pub mod record;
pub mod user;

use crate::{
    error::PointercrateError,
    middleware::mime::Accept,
    state::PointercrateState,
    view::{error::ErrorPage, Page},
    Result,
};
use actix_web::{error::ResponseError, HttpRequest, HttpResponse};
use mime;
use tokio::prelude::future::{err, Either, Future};

pub type PCResponder = Box<dyn Future<Item = HttpResponse, Error = PointercrateError>>;

pub fn wrap<F>(
    handler: F,
) -> impl Fn(
    &HttpRequest<PointercrateState>,
) -> Box<dyn Future<Item = HttpResponse, Error = PointercrateError>>
where
    F: Fn(&HttpRequest<PointercrateState>) -> PCResponder + 'static,
{
    move |req: &HttpRequest<PointercrateState>| -> Box<dyn Future<Item = HttpResponse, Error = PointercrateError>> {
        let future = match preferred_mime_type(req) {
            Err(error) => Either::A(err(error)),
            Ok(mime_type) => {
                let req_clone = req.clone();
                let f = handler(req).or_else(move |error| {
                    let response = match (mime_type.type_(), mime_type.subtype()) {
                        (mime::TEXT, mime::HTML)  => {
                            let html = ErrorPage::new(&error).render(&req_clone);

                            HttpResponse::Ok()
                                .content_type("text/html; charset=utf-8")
                                .body(html.0)
                        },
                        (mime::APPLICATION, mime::JSON) => error.error_response(),
                        _ => unreachable!()
                    };

                    Ok(response)
                });
                Either::B(f)
            }
        };

        Box::new(future)
    }
}

pub fn preferred_mime_type(req: &HttpRequest<PointercrateState>) -> Result<mime::Mime> {
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
