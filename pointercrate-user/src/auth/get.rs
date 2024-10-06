use crate::{
    auth::{AuthenticatedUser, AuthenticationType},
    error::Result,
    User,
};
use log::{debug, info};
use pointercrate_core::error::CoreError;
use sqlx::{Error, PgConnection};

impl AuthenticatedUser {
    pub async fn token_auth(access_token: &str, csrf_token: Option<&str>, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform token authentication");

        let sub = AuthenticatedUser::peek_jwt_sub(access_token)?;

        debug!("The token identified the user with id {}, validating...", sub);

        // Note that at this point we haven't validated the access token OR the csrf token yet.
        // However, the key they are signed with encompasses the password salt for the user they supposedly
        // identify, so we need to retrieve that.
        let user = Self::by_id(sub, connection).await?;

        match csrf_token {
            Some(csrf_token) => user.validate_token_pair(access_token, csrf_token),
            None => user.validate_programmatic_access_token(access_token),
        }
    }

    pub(in crate::auth) async fn by_id(id: i32, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
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
            }),
        }
    }

    pub(in crate::auth) async fn by_name(name: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
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
            }),
        }
    }
}
