//! Module containing middleware for dealing with HTTP preconditions

use crate::{
    error::PointercrateError,
    model::user::Authorization,
    util::{header, parse_list_of_header_values},
};
use actix_web::{
    body::Body,
    dev::{HttpResponseBuilder, Service, ServiceRequest, ServiceResponse, Transform},
    http::{Method, StatusCode},
    Error, HttpMessage, HttpRequest, HttpResponse, ResponseError,
};
use bitflags::_core::num::ParseIntError;
use derive_more::Display;
use futures::future::{err, ok, Either, Ready};
use log::{debug, info, warn};
use mime::Mime;
use serde::Serialize;
use std::{
    collections::hash_map::DefaultHasher,
    future::Future,
    hash::{Hash, Hasher},
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

#[derive(Debug, Copy, Clone)]
pub struct Etag;
pub struct EtagMiddleware<S>(S);

#[derive(Debug, Display)]
#[display(fmt = "'object hash equal to any of {:?}'", _0)]
pub struct IfMatch(Vec<u64>);

#[derive(Debug)]
pub struct Accept(pub Vec<Mime>);

#[derive(Debug)]
pub struct ContentType(pub Option<Mime>);

impl<S> Transform<S> for Etag
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<Body>, Error = Error>,
    S::Future: 'static,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Request = ServiceRequest;
    type Response = ServiceResponse<Body>;
    type Transform = EtagMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(EtagMiddleware(service))
    }
}

impl<S> Service for EtagMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<Body>, Error = Error>,
    S::Future: 'static,
{
    type Error = Error;
    type Future = Either<Ready<Result<Self::Response, Self::Error>>, Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>>;
    type Request = ServiceRequest;
    type Response = ServiceResponse<Body>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let if_none_match = match process_headers(&req) {
            Ok(if_none_match) => if_none_match,
            Err(pc_err) => {
                let response = pc_err.dynamic(req.headers()).error_response();

                return Either::Left(ok(req.into_response(response)))
            },
        };

        let inner = self.0.call(req);

        Either::Right(Box::pin(async move {
            let response = inner.await?;

            if response.status() == StatusCode::OK {
                // we'll just assume that we always set the value correctly
                if let Ok(Some(etag)) = header(response.headers(), "ETag") {
                    // While we ourselves always generate valid integers as etags, actix's Files service does not!
                    if let Ok(etag) = etag.parse() {
                        let if_match = response.request().extensions_mut().remove::<IfMatch>();
                        let request_method = response.request().method();

                        if let Some(if_match) = if_match {
                            if request_method == Method::PATCH && if_match.met(etag) {
                                return Ok(response.into_response(HttpResponse::NotModified().finish()))
                            }
                        }
                        if !if_none_match.is_empty() {
                            if request_method == Method::GET && if_none_match.contains(&etag) {
                                return Ok(response.into_response(HttpResponse::NotModified().finish()))
                            }
                        }
                    }
                }
            }

            Ok(response)
        }))
    }
}

/// Returns parsed `If-Match` header values and unprocessed `If-None-Match` header values
///
/// Returns the `If-None-Match` values.
fn process_headers(request: &ServiceRequest) -> Result<Vec<u64>, PointercrateError> {
    // We only need the values for this header _after_ the request completes since they are only
    // relevant for comparison with ETag values, but we also want to abort early if they are malformed,
    // which is why we simply retrieve them once here.
    let if_none_match = parse_list_of_header_values(request.headers(), "If-None-Match")?;
    let if_match = parse_list_of_header_values(request.headers(), "If-Match")?;

    // PATCH requires `If-Match`, always. Actually checking if they match is up to the
    // actual endpoint though!
    let method = request.method();

    if method == Method::PATCH || method == Method::DELETE {
        if if_match.is_empty() {
            warn!("PATCH or DELETE request without If-Match header");
        } else {
            debug!("If-Match values are {:?}", if_match);
        }
    }
    if method == Method::GET {
        if if_none_match.is_empty() {
            info!("GET without If-None-Match header")
        }
    }

    let mut extensions = request.extensions_mut();

    if !if_match.is_empty() {
        extensions.insert(IfMatch(if_match));
    }

    Ok(if_none_match)
}

impl IfMatch {
    pub fn met(&self, etag: u64) -> bool {
        self.0.contains(&etag)
    }
}
