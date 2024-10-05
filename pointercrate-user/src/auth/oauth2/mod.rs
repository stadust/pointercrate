use base64::Engine;
use getrandom::getrandom;
use pointercrate_core::error::CoreError;

use crate::Result;
use crate::User;

use super::AuthenticatedUser;

#[cfg(feature = "oauth2")]
pub mod get;
#[cfg(feature = "oauth2")]
mod patch;
mod post;

pub struct OAuth2AuthenticatedUser {
    user: User,
    google_account_id: String,
    b64_salt: String,
}

#[cfg(feature = "oauth2")]
#[derive(serde::Deserialize)]
pub struct GoogleUserInfo {
    #[serde(rename = "sub")]
    pub id: String,
    pub name: String,
}

impl OAuth2AuthenticatedUser {
    pub fn into_user(self) -> User {
        self.user
    }

    pub fn user(&self) -> &User {
        &self.user
    }

    pub fn salt(&self) -> Vec<u8> {
        // unwrap okay: we trust our own database to not contain nonsense here
        base64::prelude::BASE64_STANDARD.decode(&self.b64_salt).unwrap()
    }
}

impl AuthenticatedUser {
    pub fn oauth2(user: User, google_account_id: String, b64_salt: String) -> Self {
        AuthenticatedUser::OAuth2(OAuth2AuthenticatedUser {
            user,
            google_account_id,
            b64_salt,
        })
    }
}

fn generate_salt() -> Result<String> {
    let mut salt = [0u8; 16];
    getrandom(&mut salt).map_err(|err| CoreError::internal_server_error(err.to_string()))?;
    Ok(base64::prelude::BASE64_STANDARD.encode(salt))
}
