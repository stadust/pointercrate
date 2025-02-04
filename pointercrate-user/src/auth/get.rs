use crate::{
    auth::{AuthenticatedUser, AuthenticationType},
    error::Result,
    User,
};
use pointercrate_core::error::CoreError;
use sqlx::{Error, PgConnection};

use super::NoAuth;

impl AuthenticatedUser<NoAuth> {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<Self> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash, generation FROM members WHERE member_id = $1"#,
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(AuthenticatedUser {
                gen: row.generation,
                auth_type: AuthenticationType::legacy(construct_from_row!(row), row.password_hash),
                auth_artifact: NoAuth,
            }),
        }
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<Self> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash, generation FROM members WHERE members.name = $1"#,
            name.to_string()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(AuthenticatedUser {
                gen: row.generation,
                auth_type: AuthenticationType::legacy(construct_from_row!(row), row.password_hash),
                auth_artifact: NoAuth,
            }),
        }
    }

    pub async fn by_google_account_id(google_account_id: &str, connection: &mut PgConnection) -> Result<Self> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash, generation, discord_account_id, google_account_id FROM members WHERE members.google_account_id = $1"#,
            google_account_id.to_string()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(AuthenticatedUser {
                gen: row.generation,
                auth_type: AuthenticationType::oauth2(construct_from_row!(row), row.google_account_id, row.discord_account_id),
                auth_artifact: NoAuth,
            }),
        }
    }

    pub async fn by_discord_account_id(discord_account_id: &str, connection: &mut PgConnection) -> Result<Self> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash, generation, discord_account_id, google_account_id FROM members WHERE members.discord_account_id = $1"#,
            discord_account_id.to_string()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(AuthenticatedUser {
                gen: row.generation,
                auth_type: AuthenticationType::oauth2(construct_from_row!(row), row.google_account_id, row.discord_account_id),
                auth_artifact: NoAuth,
            }),
        }
    }
}
