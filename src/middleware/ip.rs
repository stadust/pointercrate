use crate::error::PointercrateError;
use actix_web::{
    middleware::{Middleware, Started},
    HttpRequest, Result,
};
use ipnetwork::IpNetwork;
use log::{error, warn};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Copy, Clone)]
pub struct IpResolve;

impl<S> Middleware<S> for IpResolve {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        if let Some(sockaddr) = req.peer_addr() {
            // We'll have apache reverse-proxying for us, so we gotta check this
            if IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) == sockaddr.ip() {
                if let Some(forwarded_for) = header!(req, "X-FORWARDED-FOR") {
                    let remote_addr: IpAddr = forwarded_for.parse().map_err(|_| {
                        PointercrateError::InvalidHeaderValue {
                            header: "X-FORWARDED-FOR",
                        }
                    })?;

                    req.extensions_mut().insert::<IpNetwork>(remote_addr.into());
                } else if cfg!(debug_assertions) {
                    warn!("Request from local machine, but no 'X-FORWARDED-FOR' header is set. Allowing, since this is a debug build");

                    req.extensions_mut()
                        .insert::<IpNetwork>(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).into());
                } else {
                    error!("Request from local machine, but no 'X-FORWARDED-FOR' header is set. Since this is a release build, this is a configuration error!");

                    return Err(PointercrateError::InternalServerError.into())
                }
            } else {
                req.extensions_mut()
                    .insert::<IpNetwork>(sockaddr.ip().into())
            }
        } else {
            warn!(
                "Remote address for request to {} not retrievable, aborting!",
                req.uri()
            );

            return Err(PointercrateError::Unauthorized.into())
        }

        Ok(Started::Done)
    }
}
