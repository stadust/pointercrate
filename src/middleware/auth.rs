use actix_web::{
    middleware::{Middleware, Started},
    Error, HttpRequest,
};
use crate::{error::PointercrateError, state::PointercrateState};
use serde_derive::Deserialize;

#[derive(Debug)]
pub enum Authorization {
    Unauthorized,
    Basic(String, String),
    Token(String),
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub id: i32,
}

pub struct Authorizer;

impl Middleware<PointercrateState> for Authorizer {
    fn start(&self, req: &HttpRequest<PointercrateState>) -> Result<Started, Error> {
        let authorization = if let Some(auth) = req.headers().get("Authorization") {
            let auth = auth
                .to_str()
                .map_err(|_| PointercrateError::InvalidHeaderValue { header: "Authorization" })?;

            let parts = auth.split(' ').collect::<Vec<_>>();

            match &parts[..] {
                ["Basic", basic_auth] => {
                    let decoded = base64::decode(basic_auth)
                        .map_err(|_| ())
                        .and_then(|bytes| String::from_utf8(bytes).map_err(|_| ()))
                        .map_err(|_| PointercrateError::InvalidHeaderValue { header: "Authorization" })?;

                    if let [username, password] = &decoded.split(':').collect::<Vec<_>>()[..] {
                        Authorization::Basic(username.to_string(), password.to_string())
                    } else {
                        return Err(PointercrateError::InvalidHeaderValue { header: "Authorization" })?
                    }
                },
                ["Bearer", token] => Authorization::Token(token.to_string()),
                _ => return Err(PointercrateError::InvalidHeaderValue { header: "Authorization" })?,
            }
        } else {
            Authorization::Unauthorized
        };

        req.extensions_mut().insert(authorization);

        Ok(Started::Done)
    }
}
