use pointercrate_core::error::CoreError;
use sqlx::PgConnection;

use crate::auth::AuthenticatedUser;
use crate::Result;

use super::{generate_salt, get::get_oauth2_id};

impl AuthenticatedUser {
    pub async fn upgrade_legacy_account(self, oauth_code: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        if !self.is_legacy() {
            return Err(CoreError::Unauthorized.into());
        }

        let guser_info = get_oauth2_id(oauth_code).await?;
        let b64_salt = generate_salt()?;

        sqlx::query!(
            "UPDATE members SET google_account_id = $1, password_hash = $2 WHERE member_id = $3",
            guser_info.id,
            b64_salt,
            self.user().id
        )
        .execute(connection)
        .await?;

        Ok(AuthenticatedUser::oauth2(self.into_user(), guser_info.id, b64_salt))
    }
}
