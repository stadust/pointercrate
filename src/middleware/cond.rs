//! Module containing middleware for dealing with HTTP preconditions

use crate::error::PointercrateError;
use actix_web::{
    dev::HttpResponseBuilder,
    http::Method,
    middleware::{Middleware, Response, Started},
    Error, HttpRequest, HttpResponse,
};
use derive_more::Display;
use log::{debug, warn};
use serde::Serialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Debug)]
pub struct Precondition;

#[derive(Debug, Display)]
#[display(fmt = "'object hash equal to any of {:?}'", _0)]
pub struct IfMatch(Vec<u64>);

impl<S> Middleware<S> for Precondition {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
        // We only need the values _after_ the requests completes and we can compare them against
        // the response, but we also want to abort early if they are malformed, which is why we
        // simply retrieve them once here. (There really is no advantage to already parsing them
        // here and then storing them in the request because we'd just add the overhead of cloning
        // the headers)
        let if_match = header!(req, "If-Match");
        let _ = header!(req, "If-None-Match");

        // PATCH requires `If-Match`, always. Actually checking if they match is up to the
        // actual endpoint though!
        if req.method() == Method::PATCH || req.method() == Method::DELETE {
            match if_match {
                None => {
                    warn!("PATCH or DELETE request without conditional header");

                    return Err(PointercrateError::PreconditionRequired)?
                },
                Some(if_match) => {
                    let mut hashes = Vec::new();

                    for hash in if_match.split(',') {
                        match hash.parse::<u64>() {
                            Err(err) => {
                                warn!("Malformed 'If-Match' header value {:?}: {}", hash, err);

                                return Err(PointercrateError::InvalidHeaderValue {
                                    header: "If-Match",
                                })?
                            },
                            Ok(hash) => hashes.push(hash),
                        }
                    }

                    debug!("IfMatch values are {:?}", hashes);

                    req.extensions_mut().insert(IfMatch(hashes));
                },
            }
        }

        Ok(Started::Done)
    }

    fn response(&self, req: &HttpRequest<S>, resp: HttpResponse) -> Result<Response, Error> {
        let if_match = header!(req, "If-Match")
            .unwrap_or("")
            .split(',')
            .collect::<Vec<_>>();
        let if_none_match = header!(req, "If-None-Match")
            .unwrap_or("")
            .split(',')
            .collect::<Vec<_>>();

        if let Some(etag) = header!(resp, "ETag") {
            match *req.method() {
                Method::GET if if_none_match.contains(&etag) =>
                    Ok(Response::Done(HttpResponse::NotModified().finish())),
                Method::PATCH if if_match.contains(&etag) =>
                    Ok(Response::Done(HttpResponse::NotModified().finish())),
                _ => Ok(Response::Done(resp)),
            }
        } else {
            Ok(Response::Done(resp))
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
