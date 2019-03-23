use crate::{error::PointercrateError, model::user::User, state::PointercrateState};
use actix_web::{
    http::Method,
    middleware::{Middleware, Started},
    Error, HttpRequest,
};
use derive_more::Display;
use log::{debug, warn};
use serde_derive::{Deserialize, Serialize};

/// Enum representing a parsed `Authorization` header
#[derive(Debug)]
pub enum Authorization {
    /// No `Authorization` header has been provided
    Unauthorized,

    /// The chosen authorization method was `Basic`
    Basic { username: String, password: String },

    /// The chosen authorization method was `Bearer`
    Token {
        access_token: String,
        csrf_token: Option<String>,
    },
}

#[derive(Debug)]
pub enum AuthType {
    Basic,
    Token,
}

pub trait TAuthType: Send + Sync + 'static {
    fn auth_type() -> AuthType;
}

#[derive(Debug)]
pub struct Basic;

#[derive(Debug)]
pub struct Token;

impl TAuthType for Basic {
    fn auth_type() -> AuthType {
        AuthType::Basic
    }
}

impl TAuthType for Token {
    fn auth_type() -> AuthType {
        AuthType::Token
    }
}

/// The user that made an authorized request
#[derive(Debug, Serialize, Hash, Display)]
#[serde(transparent)]
pub struct Me(pub User);

impl PartialEq<User> for Me {
    fn eq(&self, other: &User) -> bool {
        &self.0 == other
    }
}

impl PartialEq<Me> for User {
    fn eq(&self, other: &Me) -> bool {
        self == &other.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CSRFClaims {
    pub id: i32,
    pub exp: u64,
    pub iat: u64,
}

/// Actix-Web Middleware that deals with Authorization headers
///
/// This middleware tries to process any `Authorization` header before the request itself is
/// processed and stores an [`Authorization`] object in the [`HttpRequest`]'s extension map.
///
/// + Basic Authorization: In case of basic authorization, this middleware strips the `Basic`
/// identifier, base64 decodes the header value, extracts the username and password combo and
/// constructs an [`Authorization::Basic`] variant. Should the header not be valid base64, or the
/// decoded header not be valid UTF-8, or the `username:password` be somehow malformed,
/// [`PointercrateError::InvalidHeaderValue`] is returned and request processing is aborted
/// + Token Authorization: In case of token authorization, this middleware strips the `Bearer`
/// identifier and constructs a [`Authorization::Token`] variant with the remaining string. Should
/// he header for some reason only consists of the string `Bearer`,
/// [`PointercrateError::InvalidHeaderValue`] is returned and request processing is aborted.
/// + Random nonsense in the header: [`PointercrateError::InvalidHeaderValue`] is returned and
/// request processing is aborted.
/// + No authorization: The [`Authorization::Unauthoried`] variant is
/// constructed
#[derive(Debug)]
pub struct Authorizer;

impl Middleware<PointercrateState> for Authorizer {
    fn start(&self, req: &HttpRequest<PointercrateState>) -> Result<Started, Error> {
        let authorization = if let Some(auth) = header!(req, "Authorization") {
            let parts = auth.split(' ').collect::<Vec<_>>();

            match &parts[..] {
                ["Basic", basic_auth] => {
                    let decoded = base64::decode(basic_auth)
                        .map_err(|_| ())
                        .and_then(|bytes| String::from_utf8(bytes).map_err(|_| ()))
                        .map_err(|_| {
                            warn!("Malformed 'Authorization' header");

                            PointercrateError::InvalidHeaderValue {
                                header: "Authorization",
                            }
                        })?;

                    if let [username, password] = &decoded.split(':').collect::<Vec<_>>()[..] {
                        debug!("Found basic authorization!");

                        Authorization::Basic {
                            username: username.to_string(),
                            password: password.to_string(),
                        }
                    } else {
                        warn!("Malformed 'Authorization' header");

                        return Err(PointercrateError::InvalidHeaderValue {
                            header: "Authorization",
                        })?
                    }
                },
                ["Bearer", token] => {
                    debug!("Found token (Bearer) authorization");

                    Authorization::Token {
                        access_token: token.to_string(),
                        csrf_token: None,
                    }
                },
                _ => {
                    warn!("Malformed 'Authorization' header");
                    return Err(PointercrateError::InvalidHeaderValue {
                        header: "Authorization",
                    })?
                },
            }
        } else {
            debug!("Found no authorization header, testing for cookie based authorization!");

            if let Some(token_cookie) = req.cookie("access_token") {
                debug!("Found 'access_token' cookie");

                let token = token_cookie.value();

                if *req.method() == Method::GET {
                    debug!("GET request, the cookie is enough");

                    Authorization::Token {
                        access_token: token.to_string(),
                        csrf_token: None,
                    }
                } else {
                    debug!("Non-GET request, testing X-CSRF-TOKEN header");
                    // if we're doing cookie based authorization, there needs to be a X-CSRF-TOKEN
                    // header set, unless we're in GET requests, in which case everything is fine
                    // :tm:

                    match header!(req, "X-CSRF-TOKEN") {
                        Some(csrf_token) =>
                            Authorization::Token {
                                access_token: token.to_string(),
                                csrf_token: Some(csrf_token.to_string()),
                            },
                        None => {
                            warn!("Cookie based authentication was used, but no CSRF-token was provided. This is either because the requested endpoint does not required authorization (likely) or an CSRF attack (unlikely)");
                            // Here's the thing: We cannot simply abort the request here, as this
                            // could be a POST request that doesn't
                            // require authentication. The browser would
                            // send the cookie along anyway, but there'd be no csrf token (because
                            // why would there be, the request doesn't
                            // request auth). We therefore act as if not
                            // even the cookie was set
                            Authorization::Unauthorized
                        },
                    }
                }
            } else {
                debug!("No cookie found, we're unauthorized!");

                Authorization::Unauthorized
            }
        };

        req.extensions_mut().insert(authorization);

        Ok(Started::Done)
    }
}
