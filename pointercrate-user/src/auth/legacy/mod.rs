use pointercrate_core::error::CoreError;

use crate::{error::UserError, User};

use super::AuthenticationType;

#[cfg(feature = "legacy_accounts")]
pub use post::Registration;

mod patch;
mod post;

pub struct LegacyAuthenticatedUser {
    user: User,
    password_hash: String,
}

impl LegacyAuthenticatedUser {
    pub fn into_user(self) -> User {
        self.user
    }

    pub fn user(&self) -> &User {
        &self.user
    }

    pub(super) fn verify(&self, password: &str) -> Result<(), UserError> {
        let valid = bcrypt::verify(password, &self.password_hash)
            .inspect_err(|bcrypt_err| {
                log::error!(
                    "Internal Error during password verification for account {}: {:?}",
                    self.user,
                    bcrypt_err
                )
            })
            .map_err(|_| UserError::Core(CoreError::Unauthorized))?;

        if valid {
            log::debug!("Password correct, proceeding");

            Ok(())
        } else {
            log::warn!("Wrong password for account {}", self.user);

            Err(UserError::Core(CoreError::Unauthorized))
        }
    }

    pub fn validate_password(password: &str) -> Result<(), UserError> {
        if password.len() < 10 {
            return Err(UserError::InvalidPassword);
        }

        Ok(())
    }
}

impl AuthenticationType {
    pub fn legacy(user: User, password_hash: String) -> Self {
        AuthenticationType::Legacy(LegacyAuthenticatedUser { user, password_hash })
    }
}
