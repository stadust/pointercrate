use crate::{
    auth::AuthenticatedUser,
    error::{Result, UserError},
    patch::PatchUser,
    User,
};
use pointercrate_core::{
    error::CoreError,
    util::{non_nullable, nullable},
};
use serde::Deserialize;
use sqlx::PgConnection;
use std::fmt::{Debug, Formatter};

use super::{AuthenticationType, PasswordOrBrowser};

#[derive(Deserialize, Default)]
pub struct PatchMe {
    #[serde(default, deserialize_with = "non_nullable")]
    pub(super) password: Option<String>,

    #[serde(default, deserialize_with = "nullable")]
    pub(super) display_name: Option<Option<String>>,

    #[serde(default, deserialize_with = "nullable")]
    pub(super) youtube_channel: Option<Option<String>>,
}

impl PatchMe {
    pub fn changes_password(&self) -> bool {
        self.password.is_some()
    }
}

// manual debug impl to ensure that the password field is never printed anywhere
impl Debug for PatchMe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatchMe")
            .field("display_name", &self.display_name)
            .field("youtube_channel", &self.youtube_channel)
            .finish()
    }
}

impl AuthenticatedUser<PasswordOrBrowser> {
    pub async fn apply_patch(mut self, patch: PatchMe, connection: &mut PgConnection) -> Result<User> {
        if let Some(password) = patch.password {
            if !self.auth_artifact.is_password() {
                return Err(CoreError::Unauthorized.into());
            }

            self.set_password(password, connection).await?;
        }

        self.into_user()
            .apply_patch(
                PatchUser {
                    display_name: patch.display_name,
                    youtube_channel: patch.youtube_channel,
                    permissions: None,
                },
                connection,
            )
            .await
    }

    async fn set_password(&mut self, password: String, connection: &mut PgConnection) -> Result<()> {
        match &mut self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.set_password(password, connection).await?,
            _ => return Err(UserError::NonLegacyAccount),
        }

        // needed to invalidate existing access tokens
        self.increment_generation_id(connection).await
    }

    pub(super) async fn increment_generation_id(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!(
            "UPDATE members SET generation = generation + 1 WHERE member_id = $1",
            self.user().id
        )
        .execute(connection)
        .await?;

        self.gen += 1;

        Ok(())
    }

    #[cfg(feature = "oauth2")]
    pub async fn link_google_account(
        &mut self, creds: &super::oauth::ValidatedGoogleCredentials, connection: &mut PgConnection,
    ) -> Result<()> {
        match &mut self.auth_type {
            AuthenticationType::Legacy(legacy) => legacy.set_linked_google_account(creds, connection).await?,
            _ => return Err(CoreError::Unauthorized.into()),
        }

        self.increment_generation_id(connection).await
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "legacy_accounts")]
    #[sqlx::test(migrations = "../migrations")]
    async fn test_password_change_invalidates_tokens(mut conn: sqlx::pool::PoolConnection<sqlx::Postgres>) {
        use crate::auth::{legacy::Registration, AccessClaims, AuthenticatedUser, PatchMe};

        let patrick = AuthenticatedUser::register(
            Registration {
                name: "Patrick".to_string(),
                password: "bad password".to_string(),
            },
            &mut conn,
        )
        .await
        .unwrap();

        let token = patrick.generate_programmatic_access_token();

        let patrick = patrick
            .apply_patch(
                PatchMe {
                    password: Some("worse password".into()),
                    ..Default::default()
                },
                &mut conn,
            )
            .await
            .unwrap();
        let patrick = AuthenticatedUser::by_id(patrick.id, &mut conn).await.unwrap();

        assert!(patrick.validate_api_access(AccessClaims::decode(&token).unwrap()).is_err());
    }
}
