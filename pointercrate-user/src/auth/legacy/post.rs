#[cfg(feature = "legacy_accounts")]
pub use register::Registration;
use sqlx::PgConnection;

use crate::auth::LegacyAuthenticatedUser;
use crate::Result;

impl LegacyAuthenticatedUser {
    /// Invalidates all access tokens for the given account
    ///
    /// Works by re-hashing the password, and updating the `password_hash` field in the database
    /// with the new hash. Even rehashing the same password causes a new salt to be used, and this
    /// salt is part of the signing key for access tokens. Thus, changing the salt causes all old
    /// tokens to be invalidated.
    pub async fn invalidate_all_tokens(mut self, password: String, connection: &mut PgConnection) -> Result<()> {
        log::warn!("Invalidating all tokens for user {}", self.user);

        self.set_password(password, connection).await?;

        Ok(())
    }
}

#[cfg(feature = "legacy_accounts")]
mod register {
    use super::*;
    use crate::{auth::AuthenticatedUser, error::UserError, User};
    use serde::{Deserialize, Serialize};
    use sqlx::PgConnection;

    #[derive(Deserialize, Serialize)]
    pub struct Registration {
        pub name: String,
        pub password: String,
    }

    impl AuthenticatedUser {
        pub async fn register(registration: Registration, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
            log::info!("Attempting registration of new user under name {}", registration.name);

            log::trace!("Registration request is formally correct");

            match User::by_name(&registration.name, connection).await {
                Ok(_) => Err(UserError::NameTaken),
                Err(UserError::UserNotFoundName { .. }) => {
                    let hash = bcrypt::hash(&registration.password, bcrypt::DEFAULT_COST).unwrap();

                    let id = sqlx::query!(
                        "INSERT INTO members (name, password_hash) VALUES ($1, $2) RETURNING member_id",
                        registration.name,
                        hash
                    )
                    .fetch_one(connection)
                    .await?
                    .member_id;

                    log::info!("Newly registered user with name {} has been assigned ID {}", registration.name, id);

                    Ok(AuthenticatedUser::legacy(
                        User {
                            id,
                            name: registration.name,
                            permissions: 0,
                            display_name: None,
                            youtube_channel: None,
                        },
                        hash,
                    ))
                },
                Err(err) => Err(err),
            }
        }
    }
}
