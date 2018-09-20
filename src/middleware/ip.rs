use actix_web::{
    middleware::{Middleware, Started},
    HttpRequest, Result,
};
use error::PointercrateError;
use ipnetwork::IpNetwork;
use std::net::{IpAddr, Ipv4Addr};

struct IpResolve;

impl<S> Middleware<S> for IpResolve {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        if let Some(sockaddr) = req.peer_addr() {
            // We'll have apache reverse-proxying for us, so we gotta check this
            if IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) == sockaddr.ip() {
                if let Some(forwarded_for) = req.headers().get("X-FORWARDED-FOR") {
                    let remote_addr: IpAddr = forwarded_for
                        .to_str()
                        .map_err(|_| ())
                        .and_then(|addr| addr.parse().map_err(|_| ()))
                        .map_err(|_| PointercrateError::InvalidHeaderValue { header: "X-FORWARDED-FOR" })?;

                    req.extensions_mut().insert::<IpNetwork>(remote_addr.into());
                } else {
                    return Err(PointercrateError::Unauthorized.into())
                }
            } else {
                req.extensions_mut().insert::<IpNetwork>(sockaddr.ip().into())
            }
        } else {
            return Err(PointercrateError::Unauthorized.into())
        }

        Ok(Started::Done)
    }
}
