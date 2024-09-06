#[cfg(feature = "legacy_accounts")]
use crate::{auth::AuthenticationMethod, error::UserError, User};
use crate::{
    auth::{patch::PatchMe, AuthenticatedUser},
    error::Result,
};
use log::warn;
#[cfg(feature = "legacy_accounts")]
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;

#[cfg(feature = "legacy_accounts")]
#[derive(Deserialize, Serialize)]
pub struct Registration {
    pub name: String,
    pub password: String,
}

impl AuthenticatedUser {
    #[cfg(feature = "legacy_accounts")]
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

                Ok(AuthenticatedUser {
                    user: User {
                        id,
                        name: registration.name,
                        permissions: 0,
                        display_name: None,
                        youtube_channel: None,
                    },
                    auth_method: AuthenticationMethod::Legacy { password_hash: hash },
                })
            },
            Err(err) => Err(err),
        }
    }

    pub async fn invalidate_all_tokens(self, password: &str, connection: &mut PgConnection) -> Result<()> {
        let patch = PatchMe {
            password: Some(password.to_string()),
            display_name: None,
            youtube_channel: None,
        };

        warn!("Invalidating all tokens for user {}", self.inner());

        self.apply_patch(patch, connection).await?;

        Ok(())
    }
}
