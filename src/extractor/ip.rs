use crate::{error::PointercrateError, Result};
use actix_web::{
    dev::{Payload, PayloadStream},
    error::PayloadError,
    web::Bytes,
    Error, FromRequest, HttpRequest,
};
use futures::{
    future::{err, ok, ready, Ready},
    Stream,
};
use log::{error, warn};
use std::net::{IpAddr, Ipv4Addr};

pub struct Ip(pub IpAddr);

impl FromRequest for Ip {
    type Config = ();
    type Error = PointercrateError;
    type Future = Ready<Result<Ip>>;

    fn from_request(request: &HttpRequest, _payload: &mut Payload<PayloadStream>) -> Self::Future {
        if let Some(sockaddr) = request.peer_addr() {
            // We'll have nginx reverse-proxying for us, so we gotta check this
            if IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) == sockaddr.ip() {
                match request.headers().get("X-FORWARDED-FOR") {
                    Some(value) =>
                        ready(
                            value
                                .to_str()
                                .map_err(|_| PointercrateError::InvalidHeaderValue { header: "X-FORWARDED-FOR" })
                                .and_then(|forwarded_for| {
                                    forwarded_for
                                        .parse()
                                        .map_err(|_| PointercrateError::InvalidHeaderValue { header: "X-FORWARDED-FOR" })
                                })
                                .map(Ip),
                        ),
                    None =>
                        if cfg!(debug_assertions) {
                            warn!(
                                "Request from local machine, but no 'X-FORWARDED-FOR' header is set. Allowing, since this is a debug build"
                            );

                            ok(Ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).into()))
                        } else {
                            error!(
                                "Request from local machine, but no 'X-FORWARDED-FOR' header is set. Since this is a release build, this \
                                 is a configuration error!"
                            );

                            err(PointercrateError::InternalServerError.into())
                        },
                }
            } else {
                ok(Ip(sockaddr.ip().into()))
            }
        } else {
            warn!("Remote address for request to {} not retrievable, aborting!", request.uri());

            err(PointercrateError::Unauthorized.into())
        }
    }
}
