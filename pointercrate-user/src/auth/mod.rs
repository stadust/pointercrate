//! Account/Authentication related user action
//!
//! Includes:
//! * Registration
//! * Deletion of own account
//! * Modification of own account

pub use self::{patch::PatchMe, post::Registration};
use crate::{
    error::{Result, UserError},
    User,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use log::{debug, warn};
use pointercrate_core::error::CoreError;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod delete;
mod get;
mod patch;
mod post;

pub struct AuthenticatedUser {
    user: User,
    password_hash: Option<String>,
    email_address: Option<String>,
    pub google_account_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct AccessClaims {
    pub id: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChangeEmailClaims {
    pub id: i32,
    pub email: String,
    pub exp: u64,
    pub iat: u64,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct CSRFClaims {
    pub id: i32,
    pub exp: u64,
    pub iat: u64,
}

impl AuthenticatedUser {
    pub fn into_inner(self) -> User {
        self.user
    }

    pub fn inner(&self) -> &User {
        &self.user
    }

    pub fn email_address(&self) -> Option<&str> {
        self.email_address.as_deref()
    }

    pub fn is_google_linked(&self) -> bool {
        self.google_account_id.is_some()
    }

    pub fn validate_password(password: &str) -> Result<()> {
        if password.len() < 10 {
            return Err(UserError::InvalidPassword);
        }

        Ok(())
    }

    fn jwt_secret(&self) -> Vec<u8> {
        let mut key: Vec<u8> = pointercrate_core::config::secret();
        key.extend(self.password_salt());
        key
    }

    pub fn generate_access_token(&self) -> String {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &AccessClaims { id: self.user.id },
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
                warn!("Token validation FAILED for account {}: {}", self.user, err);

                CoreError::Unauthorized.into()
            })
            .and_then(move |token_data| {
                // sanity check, should never fail
                if token_data.claims.id != self.user.id {
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

    pub fn generate_change_email_token(&self, email: String) -> String {
        let start = SystemTime::now();
        let since_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards (and this is probably gonna bite me in the ass when it comes to daytimesaving crap)");

        let claim = ChangeEmailClaims {
            id: self.user.id,
            email,
            iat: since_epoch.as_secs(),
            exp: (since_epoch + Duration::from_secs(3600)).as_secs(),
        };

        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claim,
            &EncodingKey::from_secret(&self.jwt_secret()),
        )
        .unwrap()
    }

    pub fn validate_change_email_token(&self, token: &str) -> Result<String> {
        jsonwebtoken::decode::<ChangeEmailClaims>(
            token,
            &DecodingKey::from_secret(&self.jwt_secret()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|err| {
            warn!("Change email token validation FAILED for account {}: {}", self.user, err);

            CoreError::Unauthorized.into()
        })
        .and_then(|token_data| {
            // sanity check, should never fail
            if token_data.claims.id != self.user.id {
                log::error!(
                    "Token for user {} decoded successfully even though user {} is logged in",
                    token_data.claims.id,
                    self.inner()
                );

                Err(CoreError::Unauthorized.into())
            } else {
                Ok(token_data.claims.email)
            }
        })
    }

    pub fn generate_csrf_token(&self) -> String {
        let start = SystemTime::now();
        let since_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards (and this is probably gonna bite me in the ass when it comes to daytimesaving crap)");

        let claim = CSRFClaims {
            id: self.user.id,
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
                warn!("Access token validation FAILED for account {}: {}", self.user, err);

                CoreError::Unauthorized.into()
            })
            .and_then(|token_data| {
                if token_data.claims.id != self.user.id {
                    warn!(
                        "User {} attempt to authenticate using CSRF token generated for user {}",
                        self.user, token_data.claims.id
                    );

                    Err(CoreError::Unauthorized.into())
                } else {
                    Ok(())
                }
            })
    }

    fn password_salt(&self) -> Vec<u8> {
        if self.password_hash.is_none() {
            return Vec::new();
        }

        let raw_parts: Vec<_> = self.password_hash.as_ref().unwrap().split('$').filter(|s| !s.is_empty()).collect();

        match &raw_parts[..] {
            [_, _, hash] => b64::decode(&hash[..22]),
            _ => unreachable!(),
        }
    }

    pub fn verify_password(self, password: &str) -> Result<Self> {
        debug!("Verifying a password!");

        if self.password_hash.is_none() {
            warn!("Attempt to verify password for account {} with no password hash set", self.user);

            return Err(CoreError::Unauthorized.into());
        }

        let valid = bcrypt::verify(password, &self.password_hash.as_ref().unwrap()).map_err(|err| {
            warn!("Password verification FAILED for account {}: {}", self.user, err);

            UserError::Core(CoreError::Unauthorized)
        })?;

        if valid {
            debug!("Password correct, proceeding");

            Ok(self)
        } else {
            warn!("Potentially malicious log-in attempt to account {}", self.user);

            Err(CoreError::Unauthorized.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AuthenticatedUser, User};

    fn patrick() -> AuthenticatedUser {
        AuthenticatedUser {
            user: User {
                id: 0,
                name: "Patrick".to_string(),
                permissions: 0,
                display_name: None,
                youtube_channel: None,
            },
            password_hash: Some(bcrypt::hash("bad password", bcrypt::DEFAULT_COST).unwrap()),
            email_address: None,
            google_account_id: None,
        }
    }

    fn jacob() -> AuthenticatedUser {
        AuthenticatedUser {
            user: User {
                id: 1,
                name: "Jacob".to_string(),
                permissions: 0,
                display_name: None,
                youtube_channel: None,
            },
            password_hash: Some(bcrypt::hash("bad password", bcrypt::DEFAULT_COST).unwrap()),
            email_address: None,
            google_account_id: None,
        }
    }

    #[test]
    fn test_change_email_token() {
        let patrick = patrick();
        let jacob = jacob();

        let token = patrick.generate_change_email_token("patrick@pointercrate.com".to_string());
        let validation_result = patrick.validate_change_email_token(&token);

        assert!(validation_result.is_ok());
        assert_eq!(validation_result.unwrap(), "patrick@pointercrate.com".to_string());
        assert!(jacob.validate_change_email_token(&token).is_err());
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

        // make sure they arent usable in other places that require tokens
        assert!(patrick.validate_change_email_token(&patricks_csrf_token).is_err());
        assert!(jacob.validate_change_email_token(&patricks_csrf_token).is_err());

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

// This code is copied from https://github.com/Keats/rust-bcrypt/blob/master/src/b64.rs
// with slight modifications (removal of `encode` and error handling)
mod b64 {
    use std::collections::HashMap;

    use base64::{engine::general_purpose::STANDARD, Engine};
    use lazy_static::lazy_static;

    // Decoding table from bcrypt base64 to standard base64 and standard -> bcrypt
    // Bcrypt has its own base64 alphabet
    // ./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789
    lazy_static! {
        #[allow(unused_results)]
        static ref BCRYPT_TO_STANDARD: HashMap<char, &'static str> = {
            let mut m = HashMap::new();
            m.insert('/', "B");
            m.insert('.', "A");
            m.insert('1', "3");
            m.insert('0', "2");
            m.insert('3', "5");
            m.insert('2', "4");
            m.insert('5', "7");
            m.insert('4', "6");
            m.insert('7', "9");
            m.insert('6', "8");
            m.insert('9', "/");
            m.insert('8', "+");
            m.insert('A', "C");
            m.insert('C', "E");
            m.insert('B', "D");
            m.insert('E', "G");
            m.insert('D', "F");
            m.insert('G', "I");
            m.insert('F', "H");
            m.insert('I', "K");
            m.insert('H', "J");
            m.insert('K', "M");
            m.insert('J', "L");
            m.insert('M', "O");
            m.insert('L', "N");
            m.insert('O', "Q");
            m.insert('N', "P");
            m.insert('Q', "S");
            m.insert('P', "R");
            m.insert('S', "U");
            m.insert('R', "T");
            m.insert('U', "W");
            m.insert('T', "V");
            m.insert('W', "Y");
            m.insert('V', "X");
            m.insert('Y', "a");
            m.insert('X', "Z");
            m.insert('Z', "b");
            m.insert('a', "c");
            m.insert('c', "e");
            m.insert('b', "d");
            m.insert('e', "g");
            m.insert('d', "f");
            m.insert('g', "i");
            m.insert('f', "h");
            m.insert('i', "k");
            m.insert('h', "j");
            m.insert('k', "m");
            m.insert('j', "l");
            m.insert('m', "o");
            m.insert('l', "n");
            m.insert('o', "q");
            m.insert('n', "p");
            m.insert('q', "s");
            m.insert('p', "r");
            m.insert('s', "u");
            m.insert('r', "t");
            m.insert('u', "w");
            m.insert('t', "v");
            m.insert('w', "y");
            m.insert('v', "x");
            m.insert('y', "0");
            m.insert('x', "z");
            m.insert('z', "1");
            m
        };
    }

    // Can potentially panic if the hash given contains invalid characters
    pub(super) fn decode(hash: &str) -> Vec<u8> {
        let mut res = String::with_capacity(hash.len());
        for ch in hash.chars() {
            res.push_str(BCRYPT_TO_STANDARD.get(&ch).unwrap())
        }

        // Bcrypt base64 has no padding but standard has
        // so we need to actually add padding ourselves
        if hash.len() % 4 > 0 {
            let padding = 4 - hash.len() % 4;
            for _ in 0..padding {
                res.push('=');
            }
        }

        // if we had non standard chars, it would have errored before
        STANDARD.decode(&res).unwrap()
    }
}
