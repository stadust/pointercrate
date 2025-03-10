use sqlx::PgConnection;

use crate::auth::legacy::LegacyAuthenticatedUser;
use crate::Result;

use super::ValidatedGoogleCredentials;

impl LegacyAuthenticatedUser {
    pub async fn set_linked_google_account(&mut self, creds: &ValidatedGoogleCredentials, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!(
            "UPDATE members SET google_account_id = $1 WHERE member_id = $2",
            creds.google_account_id(),
            self.user().id
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}
