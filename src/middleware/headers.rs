//! Module containing middleware for dealing with HTTP preconditions

use crate::{error::PointercrateError, model::user::Authorization};
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
pub struct Headers;
pub struct HeadersPrecondition<S>(S);

#[derive(Debug, Display)]
#[display(fmt = "'object hash equal to any of {:?}'", _0)]
pub struct IfMatch(Vec<u64>);

#[derive(Debug)]
pub struct Accept(pub Vec<Mime>);

#[derive(Debug)]
pub struct ContentType(pub Option<Mime>);

impl<S> Transform<S> for Headers
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<Body>, Error = Error>,
    S::Future: 'static,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Request = ServiceRequest;
    type Response = ServiceResponse<Body>;
    type Transform = HeadersPrecondition<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(HeadersPrecondition(service))
    }
}

impl<S> Service for HeadersPrecondition<S>
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
            Err(pc_err) => return Either::Left(err(pc_err.into())),
        };

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
///
/// Returns the `If-None-Match` values.
fn process_headers(request: &ServiceRequest) -> Result<Option<Vec<u64>>, PointercrateError> {
    // We only need the values for this header _after_ the request completes since they are only
    // relevant for comparison with ETag values, but we also want to abort early if they are malformed,
    // which is why we simply retrieve them once here.
    let if_none_match = parse_list_of_header_values(request, "If-None-Match")?;
    let if_match = parse_list_of_header_values(request, "If-Match")?;
    let accepts = parse_list_of_header_values(request, "Accept")?;
    let content_type = header!(request, "Content-Type")
        .map(|value| {
            value
                .parse::<Mime>()
                .map_err(|_| PointercrateError::InvalidHeaderValue { header: "Content-Type" })
        })
        .transpose()?;

    // PATCH requires `If-Match`, always. Actually checking if they match is up to the
    // actual endpoint though!
    let method = request.method();

    if method == Method::PATCH || method == Method::DELETE {
        if if_match.is_none() {
            warn!("PATCH or DELETE request without If-Match header");
        } else {
            debug!("If-Match values are {:?}", if_match);
        }
    }
    if method == Method::GET {
        if if_none_match.is_none() {
            info!("GET without If-None-Match header")
        }
    }

    let mut extensions = request.extensions_mut();

    if let Some(if_match) = if_match {
        extensions.insert(IfMatch(if_match));
    }

    extensions.insert(Accept(accepts.unwrap_or(Vec::new())));
    extensions.insert(ContentType(content_type));
    extensions.insert(process_authorization_header(request)?);

    Ok(if_none_match)
}

fn parse_list_of_header_values<T: FromStr>(request: &ServiceRequest, header: &'static str) -> Result<Option<Vec<T>>, PointercrateError>
where
    T::Err: std::error::Error,
{
    let value = match header!(request, header) {
        None => return Ok(None),
        Some(value) => value,
    };

    value
        .split(',')
        .map(|value| value.parse())
        .collect::<Result<_, _>>()
        .map_err(|error| {
            warn!("Malformed '{}' header value in {}: {}", header, value, error);

            PointercrateError::InvalidHeaderValue { header }
        })
        .map(Some)
}

fn process_authorization_header(request: &ServiceRequest) -> Result<Authorization, PointercrateError> {
    if let Some(auth) = header!(request, "Authorization") {
        let parts = auth.split(' ').collect::<Vec<_>>();

        match &parts[..] {
            ["Basic", basic_auth] => {
                let decoded = base64::decode(basic_auth)
                    .map_err(|_| ())
                    .and_then(|bytes| String::from_utf8(bytes).map_err(|_| ()))
                    .map_err(|_| {
                        warn!("Malformed 'Authorization' header");

                        PointercrateError::InvalidHeaderValue { header: "Authorization" }
                    })?;

                if let [username, password] = &decoded.split(':').collect::<Vec<_>>()[..] {
                    debug!("Found basic authorization!");

                    Ok(Authorization::Basic {
                        username: username.to_string(),
                        password: password.to_string(),
                    })
                } else {
                    warn!("Malformed 'Authorization' header");

                    Err(PointercrateError::InvalidHeaderValue { header: "Authorization" })
                }
            },
            ["Bearer", token] => {
                debug!("Found token (Bearer) authorization");

                Ok(Authorization::Token {
                    access_token: token.to_string(),
                    csrf_token: None,
                })
            },
            _ => {
                warn!("Malformed 'Authorization' header");

                Err(PointercrateError::InvalidHeaderValue { header: "Authorization" })
            },
        }
    } else {
        debug!("Found no authorization header, testing for cookie based authorization!");

        if let Some(token_cookie) = request.cookie("access_token") {
            debug!("Found 'access_token' cookie");

            let token = token_cookie.value();

            if request.method() == Method::GET {
                debug!("GET request, the cookie is enough");

                Ok(Authorization::Token {
                    access_token: token.to_string(),
                    csrf_token: None,
                })
            } else {
                debug!("Non-GET request, testing X-CSRF-TOKEN header");
                // if we're doing cookie based authorization, there needs to be a X-CSRF-TOKEN
                // header set, unless we're in GET requests, in which case everything is fine
                // :tm:

                match header!(request, "X-CSRF-TOKEN") {
                    Some(csrf_token) =>
                        Ok(Authorization::Token {
                            access_token: token.to_string(),
                            csrf_token: Some(csrf_token.to_string()),
                        }),
                    None => {
                        warn!(
                            "Cookie based authentication was used, but no CSRF-token was provided. This is either because the requested \
                             endpoint does not required authorization (likely) or an CSRF attack (unlikely)"
                        );
                        // Here's the thing: We cannot simply abort the request here, as this
                        // could be a POST request that doesn't
                        // require authentication. The browser would
                        // send the cookie along anyway, but there'd be no csrf token (because
                        // why would there be, the request doesn't
                        // request auth). We therefore act as if not
                        // even the cookie was set
                        Ok(Authorization::Unauthorized)
                    },
                }
            }
        } else {
            debug!("No cookie found, we're unauthorized!");

            Ok(Authorization::Unauthorized)
        }
    }
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
