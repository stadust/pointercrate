use pointercrate_core::error::CoreError;
use sqlx::{Error, PgConnection};

use crate::auth::{AuthenticatedUser, AuthenticationType, PasswordOrBrowser};
use crate::Result;
use crate::User;

use super::ValidatedGoogleCredentials;

impl AuthenticatedUser<PasswordOrBrowser> {
    pub async fn by_validated_google_creds(creds: &ValidatedGoogleCredentials, connection: &mut PgConnection) -> Result<Self> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash, generation, google_account_id FROM members WHERE google_account_id = $1"#,
            creds.google_account_id()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(AuthenticatedUser {
                gen: row.generation,
                auth_type: AuthenticationType::oauth(construct_from_row!(row)),
                auth_artifact: PasswordOrBrowser(false),
            }),
        }
    }
}
