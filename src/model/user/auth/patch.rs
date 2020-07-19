use crate::{
    model::user::{auth::AuthenticatedUser, patch::PatchUser},
    util::{non_nullable, nullable},
    Result,
};
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
    pub async fn apply_patch(mut self, patch: PatchMe, connection: &mut PgConnection) -> Result<Self> {
        if let Some(password) = patch.password {
            self.set_password(password, connection).await?;
        }

        self.user = self
            .user
            .apply_patch(
                PatchUser {
                    display_name: patch.display_name,
                    youtube_channel: patch.youtube_channel,
                    permissions: None,
                },
                connection,
            )
            .await?;

        Ok(self)
    }

    pub async fn set_password(&mut self, password: String, connection: &mut PgConnection) -> Result<()> {
        Self::validate_password(&password)?;

        // it is safe to unwrap here because the only errors that can happen are
        // 'BcryptError::CostNotAllowed' (won't happen because DEFAULT_COST is obviously allowed)
        // or errors that happen during internally parsing the hash the library itself just
        // generated. Obviously, an error there is a bug in the library, so we definitely wanna panic since
        // we're dealing with passwords
        self.password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

        sqlx::query!(
            "UPDATE members SET password_hash = $1 WHERE member_id = $2",
            self.password_hash,
            self.user.id
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}
