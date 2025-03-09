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
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash, generation, google_account_id FROM members WHERE member_id = $1"#,
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => {
                let user = construct_from_row!(row);

                let auth_type = match row.google_account_id {
                    Some(_) => AuthenticationType::oauth(user),
                    None => AuthenticationType::legacy(user, row.password_hash),
                };

                Ok(AuthenticatedUser {
                    gen: row.generation,
                    auth_type,
                    auth_artifact: NoAuth,
                })
            },
        }
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<Self> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash, generation, google_account_id FROM members WHERE members.name = $1"#,
            name.to_string()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => {
                let user = construct_from_row!(row);

                let auth_type = match row.google_account_id {
                    Some(_) => AuthenticationType::oauth(user),
                    None => AuthenticationType::legacy(user, row.password_hash),
                };

                Ok(AuthenticatedUser {
                    gen: row.generation,
                    auth_type,
                    auth_artifact: NoAuth,
                })
            },
        }
    }
}
