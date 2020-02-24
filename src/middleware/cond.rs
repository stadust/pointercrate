//! Module containing middleware for dealing with HTTP preconditions

use crate::error::PointercrateError;
use actix_web::{
    body::Body,
    dev::{HttpResponseBuilder, Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
    Error, HttpMessage, HttpResponse,
};
use bitflags::_core::num::ParseIntError;
use derive_more::Display;
use futures::future::{err, ok, Either, Ready};
use log::{debug, info, warn};
use serde::Serialize;
use std::{
    collections::hash_map::DefaultHasher,
    future::Future,
    hash::{Hash, Hasher},
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug, Copy, Clone)]
pub struct Precondition;
pub struct PreconditionMiddleware<S>(S);

#[derive(Debug, Display)]
#[display(fmt = "'object hash equal to any of {:?}'", _0)]
pub struct IfMatch(Vec<u64>);

impl<S> Transform<S> for Precondition
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<Body>, Error = Error>,
    S::Future: 'static,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Request = ServiceRequest;
    type Response = ServiceResponse<Body>;
    type Transform = PreconditionMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PreconditionMiddleware(service))
    }
}

impl<S> Service for PreconditionMiddleware<S>
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
        let (if_match, if_none_match) = match process_if_match(&req) {
            Ok(if_match) => if_match,
            Err(pc_error) => return Either::Left(err(pc_error.into())),
        };

        if let Some(if_match) = if_match {
            req.extensions_mut().insert(if_match);
        }

        let inner = self.0.call(req);

        Either::Right(Box::pin(async move {
            let response = inner.await?;

            if let Some(etag) = header!(response, "ETag") {
                let etag = etag.parse().unwrap(); // we always generate valid integers
                let if_match = response.request().extensions_mut().remove::<IfMatch>();
                let request_method = response.request().method();

                if let Some(if_match) = if_match {
                    if request_method == Method::PATCH && if_match.met(etag) {
                        return Ok(response.into_response(HttpResponse::NotModified().finish()))
                    }
                }
                if let Some(if_none_match) = if_none_match {
                    if request_method == Method::GET && if_none_match.contains(&etag) {
                        return Ok(response.into_response(HttpResponse::NotModified().finish()))
                    }
                }
            }

            Ok(response)
        }))
    }
}

/// Returns parsed `If-Match` header values and unprocessed `If-None-Match` header values
fn process_if_match(request: &ServiceRequest) -> Result<(Option<IfMatch>, Option<Vec<u64>>), PointercrateError> {
    // We only need the values for this header _after_ the request completes since they are only
    // relevant for comparison with ETag values, but we also want to abort early if they are malformed,
    // which is why we simply retrieve them once here.
    let if_none_match = header!(request, "If-None-Match").map(parse_header_value).transpose()?;
    let if_match = header!(request, "If-Match").map(parse_header_value).transpose()?.map(IfMatch);

    // PATCH requires `If-Match`, always. Actually checking if they match is up to the
    // actual endpoint though!
    let method = request.method();

    if method == Method::PATCH || method == Method::DELETE || method == Method::GET {
        if if_match.is_none() {
            warn!("PATCH or DELETE request without If-Match header");
        } else {
            debug!("If-Match values are {:?}", if_match);
        }
        if if_none_match.is_none() {
            info!("GET without If-None-Match header")
        }
    }

    Ok((if_match, if_none_match))
}

fn parse_header_value(value: &str) -> Result<Vec<u64>, PointercrateError> {
    value
        .split(',')
        .map(|value| value.parse())
        .collect::<Result<Vec<u64>, _>>()
        .map_err(|error| {
            warn!("Malformed 'If-Match' header value: {}", error);

            PointercrateError::InvalidHeaderValue { header: "If-Match" }
        })
}
impl IfMatch {
    pub fn met(&self, etag: u64) -> bool {
        self.0.contains(&etag)
    }
}

pub trait HttpResponseBuilderExt {
    fn etag<H: Hash>(&mut self, obj: &H) -> &mut Self;
    fn json_with_etag<H: Serialize + Hash>(&mut self, obj: H) -> HttpResponse;
}

impl HttpResponseBuilderExt for HttpResponseBuilder {
    fn etag<H: Hash>(&mut self, obj: &H) -> &mut Self {
        let mut hasher = DefaultHasher::new();
        obj.hash(&mut hasher);
        self.header("ETag", hasher.finish().to_string())
    }

    fn json_with_etag<H: Serialize + Hash>(&mut self, obj: H) -> HttpResponse {
        self.etag(&obj).json(serde_json::json!({ "data": obj }))
    }
}
