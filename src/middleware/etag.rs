//! Module containing middleware for dealing with HTTP preconditions

use crate::util::header;
use actix_web::{
    body::Body,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{Method, StatusCode},
    Error, HttpResponse,
};
use derive_more::Display;
use futures::future::{ok, Ready};
use log::{debug, info, warn};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug, Copy, Clone)]
pub struct Etag;
pub struct EtagMiddleware<S>(S);

#[derive(Debug, Display)]
#[display(fmt = "'object hash equal to any of {:?}'", _0)]
pub struct IfMatch(Vec<u64>);

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
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Request = ServiceRequest;
    type Response = ServiceResponse<Body>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let inner = self.0.call(req);

        Box::pin(async move {
            let response = inner.await?;
            let req = response.request();

            let if_none_match = header(req.headers(), "If-None-Match")
                .ok()
                .flatten()
                .unwrap_or("")
                .split(',')
                .collect::<Vec<_>>();
            let if_match = header(req.headers(), "If-Match")
                .ok()
                .flatten()
                .unwrap_or("")
                .split(',')
                .collect::<Vec<_>>();

            // PATCH requires `If-Match`, always. Actually checking if they match is up to the
            // actual endpoint though!
            let request_method = req.method();

            if request_method == Method::PATCH || request_method == Method::DELETE {
                if if_match.is_empty() {
                    warn!("PATCH or DELETE request without If-Match header");
                } else {
                    debug!("If-Match values are {:?}", if_match);
                }
            }
            if request_method == Method::GET {
                if if_none_match.is_empty() {
                    info!("GET without If-None-Match header")
                }
            }

            if response.status() == StatusCode::OK {
                // we'll just assume that we always set the value correctly
                if let Ok(Some(etag)) = header(response.headers(), "ETag") {
                    if request_method == Method::PATCH && if_match.contains(&etag) {
                        return Ok(response.into_response(HttpResponse::NotModified().finish()))
                    }

                    if request_method == Method::GET && if_none_match.contains(&etag) {
                        return Ok(response.into_response(HttpResponse::NotModified().finish()))
                    }
                }
            }

            Ok(response)
        })
    }
}
