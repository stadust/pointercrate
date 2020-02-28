use super::{Permissions, User};
use crate::{
    util::{non_nullable, nullable},
    Result,
};
use log::info;
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize)]
pub struct PatchUser {
    #[serde(default, deserialize_with = "nullable")]
    pub display_name: Option<Option<String>>,

    #[serde(default, deserialize_with = "nullable")]
    #[allow(clippy::option_option)]
    pub youtube_channel: Option<Option<String>>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[allow(clippy::option_option)]
    pub permissions: Option<Permissions>,
}

impl User {
    /// Must run inside a transaction
    pub async fn apply_patch(mut self, patch: PatchUser, connection: &mut PgConnection) -> Result<Self> {
        info!("Applying patch {:?} to {}", patch, self);

        if let Some(permissions) = patch.permissions {
            self.set_permissions(permissions, connection).await?;
        }

        if let Some(display_name) = patch.display_name {
            match display_name {
                Some(display_name) => self.set_display_name(display_name, connection).await?,
                None => self.reset_display_name(connection).await?,
            }
        }

        if let Some(youtube_channel) = patch.youtube_channel {
            match youtube_channel {
                Some(youtube_channel) => self.set_youtube_channel(youtube_channel, connection).await?,
                None => self.reset_youtube_channel(connection).await?,
            }
        }

        Ok(self)
    }

    pub async fn set_permissions(&mut self, permissions: Permissions, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!(
            "UPDATE members SET permissions = cast($1::integer as BIT(16)) WHERE member_id = $2", // FIXME(sqlx)
            permissions.bits() as i32,
            self.id
        )
        .execute(connection)
        .await?;

        self.permissions = permissions;

        Ok(())
    }

    pub async fn set_display_name(&mut self, display_name: String, connection: &mut PgConnection) -> Result<()> {
        Self::validate_name(&display_name)?;

        sqlx::query!("UPDATE members SET display_name = $1 WHERE member_id = $2", display_name, self.id)
            .execute(connection)
            .await?;

        self.display_name = Some(display_name);

        Ok(())
    }

    pub async fn reset_display_name(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE members SET display_name = NULL WHERE member_id = $1", self.id)
            .execute(connection)
            .await?;

        self.display_name = None;

        Ok(())
    }

    pub async fn set_youtube_channel(&mut self, youtube_channel: String, connection: &mut PgConnection) -> Result<()> {
        let youtube_channel = crate::video::validate_channel(&youtube_channel)?;

        sqlx::query!(
            "UPDATE members SET youtube_channel = $1::text WHERE member_id = $2",
            youtube_channel,
            self.id
        )
        .execute(connection)
        .await?;

        self.youtube_channel = Some(youtube_channel);

        Ok(())
    }

    pub async fn reset_youtube_channel(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE members SET youtube_channel = NULL WHERE member_id = $1", self.id)
            .execute(connection)
            .await?;

        self.youtube_channel = None;

        Ok(())
    }
}
