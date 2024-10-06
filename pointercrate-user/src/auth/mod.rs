//! Account/Authentication related user action
//!
//! Includes:
//! * Registration
//! * Deletion of own account
//! * Modification of own account

pub use self::patch::PatchMe;
use crate::{error::Result, User};
use jsonwebtoken::{errors::ErrorKind, DecodingKey, EncodingKey, Validation};
use legacy::LegacyAuthenticatedUser;
use pointercrate_core::{error::CoreError, util::csprng_u64};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod delete;
mod get;
pub mod legacy;
mod patch;
mod post;

pub struct AuthenticatedUser {
    gen: i64,
    pub auth_type: AuthenticationType,
}

pub enum AuthenticationType {
    Legacy(LegacyAuthenticatedUser),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)] // to prevent a CSRF token to work as an access token
struct AccessClaims {
    /// The id of the pointercrate account this token authenticates
    ///
    /// Stored as a string as that's a requirement of [`jsonwebtoken`]'s validation facilities
    sub: String,

    /// An optional session ID.
    ///
    /// Access tokens without associated session IDs cannot be used in the web interface. The
    /// session ID is found again in the CSRF token, to ensure CSRF tokens regenerate with
    /// each new session.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    session_uuid: Option<u64>,

    /// A generation ID that allows token invalidation. Tokens are only accepted if the generation
    /// ID in the token matches the current generation id on the [`AuthenticatedUser`]
    gen: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct CSRFClaims {
    sub: String, // we're using the jsonwebtoken library's validation to check this field, which expect it to be a string
    exp: u64,
    session: u64,
}

impl AuthenticatedUser {
    pub fn into_user(self) -> User {
        match self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.into_user(),
        }
    }

    pub fn user(&self) -> &User {
        match &self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.user(),
        }
    }

    fn jwt_secret(&self) -> Vec<u8> {
        let mut key: Vec<u8> = pointercrate_core::config::secret();
        key.extend(self.salt());
        key
    }

    pub fn generate_jwt<C: Serialize>(&self, claims: &C) -> String {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret()),
        )
        .unwrap()
    }

    pub fn validate_jwt<C: DeserializeOwned>(&self, jwt: &str, mut validation: Validation) -> Result<C> {
        validation.sub = Some(self.user().id.to_string());
        validation.required_spec_claims.insert("sub".to_string());

        jsonwebtoken::decode::<C>(jwt, &DecodingKey::from_secret(&self.jwt_secret()), &validation)
            .map_err(|err| {
                if err.into_kind() == ErrorKind::InvalidSubject {
                    CoreError::internal_server_error(format!(
                        "Token for user with id {:?} decoded successfully using key for user with id {}",
                        Self::peek_jwt_sub(jwt),
                        self.user().id
                    ))
                    .into()
                } else {
                    CoreError::Unauthorized.into()
                }
            })
            .map(|token_data| token_data.claims)
    }

    pub fn peek_jwt_sub(jwt: &str) -> Result<i32> {
        // Well this is reassuring. However, we only extract the id, and ensure the remaining
        // values of the token are not even stored by using `struct _Unsafe` (serde will ignore
        // superfluous fields during deserialization since its not tagged `deny_unknown_fields`)
        let mut no_validation = Validation::default();
        no_validation.insecure_disable_signature_validation();
        no_validation.validate_exp = false;
        no_validation.set_required_spec_claims(&["sub"]);

        #[derive(Deserialize)]
        struct _Unsafe {
            sub: String,
        }

        jsonwebtoken::decode::<_Unsafe>(jwt, &DecodingKey::from_secret(b""), &no_validation)
            .map_err(|_| CoreError::Unauthorized)?
            .claims
            .sub
            .parse()
            .map_err(|_| CoreError::Unauthorized.into())
    }

    /// Generates an access token that can be used for programmatic access to the pointercrate API.
    ///
    /// These tokens are not tied to a user session, and as such cannot be used for administrative
    /// user account actions.
    pub fn generate_programmatic_access_token(&self) -> String {
        self.generate_jwt(&AccessClaims {
            sub: self.user().id.to_string(),
            session_uuid: None,
            gen: self.gen,
        })
    }

    pub fn validate_programmatic_access_token(self, token: &str) -> Result<Self> {
        self.validate_access_token(token).map(|_| self)
    }

    pub fn generate_token_pair(&self) -> Result<(String, String)> {
        let session_uuid = csprng_u64()?;
        let csrf_exp = (SystemTime::now() + Duration::from_secs(7 * 24 * 3600))
            .duration_since(UNIX_EPOCH)
            .expect("7 days in the future is earlier than the Unix Epoch. Wtf?")
            .as_secs();

        let access_claims = AccessClaims {
            sub: self.user().id.to_string(),
            session_uuid: Some(session_uuid),
            gen: self.gen,
        };

        let csrf_claims = CSRFClaims {
            sub: self.user().id.to_string(),
            exp: csrf_exp,
            session: session_uuid,
        };

        let access_token = self.generate_jwt(&access_claims);
        let csrf_token = self.generate_jwt(&csrf_claims);

        Ok((access_token, csrf_token))
    }

    pub fn validate_token_pair(self, access_token: &str, csrf_token: &str) -> Result<Self> {
        let access_claims = self.validate_access_token(access_token)?;
        let csrf_claims = self.validate_jwt::<CSRFClaims>(csrf_token, Validation::default())?;

        match access_claims.session_uuid {
            Some(session_uuid) if csrf_claims.session == session_uuid => Ok(self),
            _ => Err(CoreError::Unauthorized.into()),
        }
    }

    fn validate_access_token(&self, token: &str) -> Result<AccessClaims> {
        // No expiry on access tokens
        let mut validation = Validation::default();
        validation.validate_exp = false;
        validation.required_spec_claims = HashSet::new();

        self.validate_jwt::<AccessClaims>(token, validation).and_then(|access_claims| {
            if access_claims.gen != self.gen {
                Err(CoreError::Unauthorized.into())
            } else {
                Ok(access_claims)
            }
        })
    }

    fn salt(&self) -> Vec<u8> {
        match &self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.salt(),
        }
    }

    pub fn verify_password(self, password: &str) -> Result<Self> {
        match &self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.verify(password).map(|_| self),
            _ => Err(CoreError::Unauthorized.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{AuthenticatedUser, User};

    use super::AuthenticationType;

    fn patrick() -> AuthenticatedUser {
        AuthenticatedUser {
            auth_type: AuthenticationType::legacy(
                User {
                    id: 0,
                    name: "Patrick".to_string(),
                    permissions: 0,
                    display_name: None,
                    youtube_channel: None,
                },
                bcrypt::hash("bad password", bcrypt::DEFAULT_COST).unwrap(),
            ),
            gen: 0,
        }
    }

    fn jacob() -> AuthenticatedUser {
        AuthenticatedUser {
            auth_type: AuthenticationType::legacy(
                User {
                    id: 1,
                    name: "".to_string(),
                    permissions: 0,
                    display_name: None,
                    youtube_channel: None,
                },
                bcrypt::hash("bad password", bcrypt::DEFAULT_COST).unwrap(),
            ),
            gen: 0,
        }
    }

    #[test]
    fn test_password() {
        assert!(patrick().verify_password("bad password").is_ok());
        assert!(patrick().verify_password("lksafdölksad").is_err());
        assert!(patrick().verify_password("bad password with suffix").is_err());
    }

    #[test]
    fn test_token_pair() {
        let patrick = patrick();

        let (patricks_access_token, patricks_csrf_token) = patrick.generate_token_pair().unwrap();

        // make sure only the correct user can decode them
        let patrick = patrick.validate_token_pair(&patricks_access_token, &patricks_csrf_token).unwrap();
        assert!(jacob().validate_token_pair(&patricks_access_token, &patricks_csrf_token).is_err());

        // Make sure the tokens with session uuid also work as programmatic access tokens
        let patrick = patrick.validate_programmatic_access_token(&patricks_access_token).unwrap();

        // Make sure csrf tokens don't work as access tokens
        assert!(patrick.validate_programmatic_access_token(&patricks_csrf_token).is_err());
        assert!(jacob().validate_programmatic_access_token(&patricks_csrf_token).is_err());
    }

    #[test]
    fn test_cannot_transfer_csrf_tokens_across_sessions() {
        let patrick = patrick();

        let (access_token, _) = patrick.generate_token_pair().unwrap();
        let (_, csrf_token) = patrick.generate_token_pair().unwrap();

        assert!(patrick.validate_token_pair(&access_token, &csrf_token).is_err());
    }

    #[test]
    fn test_cannot_use_programmatic_token_with_csrf_token() {
        let patrick = patrick();

        let (_, csrf_token) = patrick.generate_token_pair().unwrap();
        let access_token = patrick.generate_programmatic_access_token();

        assert!(patrick.validate_token_pair(&access_token, &csrf_token).is_err());
    }

    #[test]
    fn test_programmatic_access_token() {
        let patrick = patrick();
        let jacob = jacob();

        let patricks_access_token = patrick.generate_programmatic_access_token();

        assert!(patrick.validate_programmatic_access_token(&patricks_access_token).is_ok());
        assert!(jacob.validate_programmatic_access_token(&patricks_access_token).is_err());
    }

    #[test]
    fn test_generation_id_change_invalidates_tokens() {
        let mut p = patrick();
        let access_token = p.generate_programmatic_access_token();
        p.gen = 1;
        assert!(p.validate_programmatic_access_token(&access_token).is_err());

        let mut p = patrick();
        let (access_token, csrf_token) = p.generate_token_pair().unwrap();
        p.gen = 1;
        assert!(p.validate_token_pair(&access_token, &csrf_token).is_err());
    }

    #[test]
    fn test_peek_jwt_sub() {
        let patrick = patrick();

        let (access_token, csrf_token) = patrick.generate_token_pair().unwrap();
        assert_eq!(AuthenticatedUser::peek_jwt_sub(&access_token).unwrap(), patrick.user().id);
        assert_eq!(AuthenticatedUser::peek_jwt_sub(&csrf_token).unwrap(), patrick.user().id);

        let access_token = patrick.generate_programmatic_access_token();
        assert_eq!(AuthenticatedUser::peek_jwt_sub(&access_token).unwrap(), patrick.user().id);
    }
}
