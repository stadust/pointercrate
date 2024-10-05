use sqlx::PgConnection;

use super::{generate_salt, OAuth2AuthenticatedUser};
#[cfg(feature = "oauth2")]
use crate::auth::oauth2::GoogleUserInfo;
use crate::auth::AuthenticatedUser;
use crate::Result;

impl OAuth2AuthenticatedUser {
    pub async fn invalidate_all_tokens(self, connection: &mut PgConnection) -> Result<()> {
        log::warn!("Invalidating all tokens for user {}", self.user);
        let b64_salt = generate_salt()?;
        sqlx::query!(
            "UPDATE members SET password_hash = $1 WHERE member_id = $2",
            b64_salt,
            self.user().id
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}

impl AuthenticatedUser {
    #[cfg(feature = "oauth2")]
    pub(in crate::auth::oauth2) async fn register_oauth(
        guser_info: GoogleUserInfo, connection: &mut PgConnection,
    ) -> Result<AuthenticatedUser> {
        use crate::User;
        // This will never conflict with an existing user
        // According to Google, the account ID is always unique
        // https://developers.google.com/identity/openid-connect/openid-connect#an-id-tokens-payload
        let name = format!("{}#{}", guser_info.name, guser_info.id);
        let b64_salt = generate_salt()?;

        let id = sqlx::query!(
            "INSERT INTO
                members (name, display_name, google_account_id, password_hash)
            VALUES
                ($1, $2, $3, $4) RETURNING member_id
            ",
            name,
            guser_info.name,
            guser_info.id,
            b64_salt
        )
        .fetch_one(connection)
        .await?
        .member_id;

        Ok(Self::oauth2(
            User {
                id,
                name,
                permissions: 0,
                display_name: Some(guser_info.name),
                youtube_channel: None,
            },
            guser_info.id,
            b64_salt,
        ))
    }
}
