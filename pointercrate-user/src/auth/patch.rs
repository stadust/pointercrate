use crate::{
    auth::AuthenticatedUser,
    error::{Result, UserError},
    patch::PatchUser,
    User,
};
use pointercrate_core::util::{non_nullable, nullable};
use serde::Deserialize;
use sqlx::PgConnection;
use std::fmt::{Debug, Formatter};

#[derive(Deserialize)]
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

impl AuthenticatedUser {
    pub async fn apply_patch(mut self, patch: PatchMe, connection: &mut PgConnection) -> Result<User> {
        if let Some(password) = patch.password {
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

    pub async fn set_password(&mut self, password: String, connection: &mut PgConnection) -> Result<()> {
        match self {
            AuthenticatedUser::Legacy(legacy) => legacy.set_password(password, connection).await,
            _ => Err(UserError::NonLegacyAccount),
        }
    }
}
