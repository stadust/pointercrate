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
use pointercrate_core::error::CoreError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)] // to prevent a CSRF token to work as an access token
struct AccessClaims {
    sub: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct CSRFClaims {
    sub: String,  // we're using the jsonwebtoken library's validation to check this field, which expect it to be a string
    exp: u64,
}

impl AuthenticatedUser {
    pub fn into_user(self) -> User {
        match self {
            AuthenticatedUser::Legacy(legacy) => legacy.into_user(),
        }
    }

    pub fn user(&self) -> &User {
        match self {
            AuthenticatedUser::Legacy(legacy) => legacy.user(),
        }
    }

    fn jwt_secret(&self) -> Vec<u8> {
        let mut key: Vec<u8> = crate::config::secret();
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
            sub: String
        }

        jsonwebtoken::decode::<_Unsafe>(jwt, &DecodingKey::from_secret(b""), &no_validation)
            .map_err(|_| CoreError::Unauthorized)?
            .claims
            .sub
            .parse()
            .map_err(|_| CoreError::Unauthorized.into())
    }

    pub fn generate_access_token(&self) -> String {
        self.generate_jwt(&AccessClaims {
            sub: self.user().id.to_string()
        })
    }

    pub fn validate_access_token(self, token: &str) -> Result<Self> {
        let mut validation = Validation::default();
        validation.validate_exp = false;
        validation.required_spec_claims = HashSet::new();

        self.validate_jwt::<AccessClaims>(token, validation).map(|_| self)
    }

    pub fn generate_csrf_token(&self) -> String {
        let start = SystemTime::now();
        let exp = (start + Duration::from_secs(3600))
            .duration_since(UNIX_EPOCH)
            .expect("one hour in the future is earlier than the Unix Epoch. Wtf?")
            .as_secs();

        let claim = CSRFClaims {
            sub: self.user().id.to_string(),
            exp,
        };

        self.generate_jwt(&claim)
    }

    pub fn validate_csrf_token(&self, token: &str) -> Result<()> {
        self.validate_jwt::<CSRFClaims>(&token, Validation::default()).map(|_| ())
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
    use crate::auth::{AuthenticatedUser, User};

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

    #[test]
    fn test_peek_jwt_sub() {
        let patrick = patrick();

        let patricks_csrf_token = patrick.generate_csrf_token();
        assert_eq!(AuthenticatedUser::peek_jwt_sub(&patricks_csrf_token).unwrap(), patrick.user().id)
    }
}
