use crate::error::{JsonError, PointercrateError};
use actix_web::{
    dev::{Payload, PayloadStream},
    FromRequest, HttpRequest,
};
use derive_more::Display;
use futures::future::{err, ready, Ready};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Debug, Display)]
#[display(fmt = "'object hash equal to any of {:?}'", _0)]
pub struct IfMatch(Vec<u64>);

impl IfMatch {
    pub fn require_etag_match<H: Hash>(&self, h: &H) -> Result<(), PointercrateError> {
        let mut hasher = DefaultHasher::new();
        h.hash(&mut hasher);

        if self.0.contains(&hasher.finish()) {
            Ok(())
        } else {
            Err(PointercrateError::PreconditionFailed)
        }
    }
}

impl FromRequest for IfMatch {
    type Config = ();
    type Error = JsonError;
    type Future = Ready<Result<Self, JsonError>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload<PayloadStream>) -> Self::Future {
        let header = match req.headers().get("If-Match") {
            Some(value) =>
                match value.to_str() {
                    Ok(value) => value,
                    Err(_) => return err(PointercrateError::InvalidHeaderValue { header: "If-Match" }.into()),
                },
            None => return err(PointercrateError::PreconditionRequired.into()),
        };

        ready(
            header
                .split(',')
                .map(|hash| {
                    hash.parse()
                        .map_err(|_| PointercrateError::InvalidHeaderValue { header: "If-Match" })
                })
                .collect::<Result<_, _>>()
                .map(IfMatch)
                .map_err(JsonError),
        )
    }
}
