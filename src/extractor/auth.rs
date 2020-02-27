//! FromRequest implementations that perform authorization

use crate::{
    error::{JsonError, PointercrateError},
    model::user::{AuthenticatedUser, Authorization},
    state::PointercrateState,
    util::header,
};
use actix_web::{
    dev::{Payload, PayloadStream},
    http::Method,
    FromRequest, HttpMessage, HttpRequest,
};
use futures::future::{err, ready, Either, Ready};
use log::{debug, error, warn};
use std::{future::Future, pin::Pin};

pub struct TokenAuth(pub AuthenticatedUser);
pub struct BasicAuth(pub AuthenticatedUser);

impl FromRequest for TokenAuth {
    type Config = ();
    type Error = JsonError;
    type Future = Either<Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>, Ready<Result<Self, Self::Error>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload<PayloadStream>) -> Self::Future {
        let state = req.app_data::<PointercrateState>().unwrap().clone();

        if state.secret.is_empty() {
            error!("No application secret set-up, rejecting all authentication token authentications");

            return Either::Right(err(PointercrateError::Unauthorized.into()))
        }

        let auth = match process_authorization_header(&req) {
            Ok(auth) => auth,
            Err(error) => return Either::Right(err(error.into())),
        };

        Either::Left(Box::pin(async move {
            let mut connection = state.connection().await?;

            Ok(TokenAuth(
                AuthenticatedUser::token_auth(&auth, &state.secret, &mut connection).await?,
            ))
        }))
    }
}

impl FromRequest for BasicAuth {
    type Config = ();
    type Error = JsonError;
    type Future = Either<Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>, Ready<Result<Self, Self::Error>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload<PayloadStream>) -> Self::Future {
        let state = req.app_data::<PointercrateState>().unwrap().clone();

        let auth = match process_authorization_header(&req) {
            Ok(auth) => auth,
            Err(error) => return Either::Right(err(error.into())),
        };

        Either::Left(Box::pin(async move {
            let mut connection = state.connection().await?;

            Ok(BasicAuth(AuthenticatedUser::basic_auth(&auth, &mut connection).await?))
        }))
    }
}

impl FromRequest for Authorization {
    type Config = ();
    type Error = JsonError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload<PayloadStream>) -> Self::Future {
        ready(process_authorization_header(req).map_err(JsonError))
    }
}

fn process_authorization_header(request: &HttpRequest) -> Result<Authorization, PointercrateError> {
    if let Some(auth) = header(request.headers(), "Authorization")? {
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
                        username: (*username).to_string(),
                        password: (*password).to_string(),
                    })
                } else {
                    warn!("Malformed 'Authorization' header");

                    Err(PointercrateError::InvalidHeaderValue { header: "Authorization" })
                }
            },
            ["Bearer", token] => {
                debug!("Found token (Bearer) authorization");

                Ok(Authorization::Token {
                    access_token: (*token).to_string(),
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

                match header(request.headers(), "X-CSRF-TOKEN")? {
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
