//! Account/Authentication related user action
//!
//! Includes:
//! * Registration
//! * Deletion of own account
//! * Modification of own account

pub use self::patch::PatchMe;
use crate::{error::Result, User};
use jsonwebtoken::{DecodingKey, EncodingKey};
use legacy::LegacyAuthenticatedUser;
use log::warn;
use pointercrate_core::error::CoreError;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod delete;
mod get;
pub mod legacy;
mod patch;

pub enum AuthenticatedUser {
    Legacy(LegacyAuthenticatedUser),
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct AccessClaims {
    pub id: i32,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct CSRFClaims {
    pub id: i32,
    pub exp: u64,
    pub iat: u64,
}

impl AuthenticatedUser {
    pub fn into_inner(self) -> User {
        match self {
            AuthenticatedUser::Legacy(legacy) => legacy.into_user(),
        }
    }

    pub fn inner(&self) -> &User {
        match self {
            AuthenticatedUser::Legacy(legacy) => legacy.user(),
        }
    }

    fn jwt_secret(&self) -> Vec<u8> {
        let mut key: Vec<u8> = pointercrate_core::config::secret();
        key.extend(self.salt());
        key
    }

    pub fn generate_access_token(&self) -> String {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &AccessClaims { id: self.inner().id },
            &EncodingKey::from_secret(&self.jwt_secret()),
        )
        .unwrap()
    }

    pub fn validate_access_token(self, token: &str) -> Result<Self> {
        // TODO: maybe one day do something with this
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_exp = false;
        validation.required_spec_claims = HashSet::default();

        jsonwebtoken::decode::<AccessClaims>(token, &DecodingKey::from_secret(&self.jwt_secret()), &validation)
            .map_err(|err| {
                warn!("Token validation FAILED for account {}: {}", self.inner(), err);

                CoreError::Unauthorized.into()
            })
            .and_then(move |token_data| {
                // sanity check, should never fail
                if token_data.claims.id != self.inner().id {
                    log::error!(
                        "Access token for user {} decoded successfully even though user {} is logged in",
                        token_data.claims.id,
                        self.inner()
                    );

                    Err(CoreError::Unauthorized.into())
                } else {
                    Ok(self)
                }
            })
    }

    pub fn generate_csrf_token(&self) -> String {
        let start = SystemTime::now();
        let since_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards (and this is probably gonna bite me in the ass when it comes to daytimesaving crap)");

        let claim = CSRFClaims {
            id: self.inner().id,
            iat: since_epoch.as_secs(),
            exp: (since_epoch + Duration::from_secs(3600)).as_secs(),
        };

        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claim,
            &EncodingKey::from_secret(&pointercrate_core::config::secret()),
        )
        .unwrap()
    }

    pub fn validate_csrf_token(&self, token: &str) -> Result<()> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_exp = false;
        validation.required_spec_claims = HashSet::new();

        jsonwebtoken::decode::<CSRFClaims>(token, &DecodingKey::from_secret(&pointercrate_core::config::secret()), &validation)
            .map_err(|err| {
                warn!("Access token validation FAILED for account {}: {}", self.inner(), err);

                CoreError::Unauthorized.into()
            })
            .and_then(|token_data| {
                if token_data.claims.id != self.inner().id {
                    warn!(
                        "User {} attempt to authenticate using CSRF token generated for user {}",
                        self.inner(),
                        token_data.claims.id
                    );

                    Err(CoreError::Unauthorized.into())
                } else {
                    Ok(())
                }
            })
    }

    fn salt(&self) -> Vec<u8> {
        match self {
            AuthenticatedUser::Legacy(legacy) => legacy.salt(),
        }
    }

    pub fn verify_password(self, password: &str) -> Result<Self> {
        match &self {
            AuthenticatedUser::Legacy(legacy) => legacy.verify(password).map(|_| self),
            _ => Err(CoreError::Unauthorized.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AuthenticatedUser, User};

    fn patrick() -> AuthenticatedUser {
        AuthenticatedUser::legacy(
            User {
                id: 0,
                name: "Patrick".to_string(),
                permissions: 0,
                display_name: None,
                youtube_channel: None,
            },
            bcrypt::hash("bad password", bcrypt::DEFAULT_COST).unwrap(),
        )
    }

    fn jacob() -> AuthenticatedUser {
        AuthenticatedUser::legacy(
            User {
                id: 1,
                name: "".to_string(),
                permissions: 0,
                display_name: None,
                youtube_channel: None,
            },
            bcrypt::hash("bad password", bcrypt::DEFAULT_COST).unwrap(),
        )
    }

    #[test]
    fn test_password() {
        assert!(patrick().verify_password("bad password").is_ok());
        assert!(patrick().verify_password("lksafdölksad").is_err());
        assert!(patrick().verify_password("bad password with suffix").is_err());
    }

    #[test]
    fn test_csrf_token() {
        let patrick = patrick();
        let jacob = jacob();

        let patricks_csrf_token = patrick.generate_csrf_token();

        // make sure only the correct user can decode them
        assert!(patrick.validate_csrf_token(&patricks_csrf_token).is_ok());
        assert!(jacob.validate_csrf_token(&patricks_csrf_token).is_err());

        assert!(patrick.validate_access_token(&patricks_csrf_token).is_err());
        assert!(jacob.validate_access_token(&patricks_csrf_token).is_err());
    }

    #[test]
    fn test_access_token() {
        let patrick = patrick();
        let jacob = jacob();

        let patricks_access_token = patrick.generate_access_token();

        assert!(patrick.validate_access_token(&patricks_access_token).is_ok());
        assert!(jacob.validate_access_token(&patricks_access_token).is_err());
    }
}
