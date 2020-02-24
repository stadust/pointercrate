use crate::error::PointercrateError;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use bitflags::_core::task::{Context, Poll};
use futures::future::{err, ok, Either, Ready};
use log::{error, warn};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Copy, Clone)]
pub struct IpResolve;
pub struct IpResolveMiddleware<S>(S);

impl<S, B> Transform<S> for IpResolve
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Transform = IpResolveMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(IpResolveMiddleware(service))
    }
}

impl<S, B> Service for IpResolveMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;
    type Request = S::Request;
    type Response = S::Response;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let ip_address = ip_from_request(&req);

        match ip_address {
            Err(pc_err) => return Either::Right(err(pc_err.into())),
            Ok(ip) => req.extensions_mut().insert::<IpAddr>(ip),
        }

        Either::Left(self.0.call(req))
    }
}

fn ip_from_request(request: &ServiceRequest) -> Result<IpAddr, PointercrateError> {
    if let Some(sockaddr) = request.peer_addr() {
        // We'll have nginx reverse-proxying for us, so we gotta check this
        if IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) == sockaddr.ip() {
            if let Some(forwarded_for) = header!(request, "X-FORWARDED-FOR") {
                let remote_addr: IpAddr = forwarded_for
                    .parse()
                    .map_err(|_| PointercrateError::InvalidHeaderValue { header: "X-FORWARDED-FOR" })?;

                Ok(remote_addr.into())
            } else if cfg!(debug_assertions) {
                warn!("Request from local machine, but no 'X-FORWARDED-FOR' header is set. Allowing, since this is a debug build");

                Ok(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).into())
            } else {
                error!(
                    "Request from local machine, but no 'X-FORWARDED-FOR' header is set. Since this is a release build, this is a \
                     configuration error!"
                );

                Err(PointercrateError::InternalServerError.into())
            }
        } else {
            Ok(sockaddr.ip().into())
        }
    } else {
        warn!("Remote address for request to {} not retrievable, aborting!", request.uri());

        Err(PointercrateError::Unauthorized.into())
    }
}
