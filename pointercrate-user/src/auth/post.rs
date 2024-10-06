use crate::Result;
use sqlx::PgConnection;

use crate::auth::AuthenticatedUser;

impl AuthenticatedUser {
    /// Invalidates all access tokens for the given account
    ///
    /// Works by incrementing the account's generation ID, which is part of every access token (and
    /// a generation ID mismatch causes the token validation to fail).
    pub async fn invalidate_all_tokens(self, connection: &mut PgConnection) -> Result<()> {
        log::warn!("Invalidating all tokens for user {}", self.user());

        sqlx::query!(
            "UPDATE members SET generation = generation + 1 WHERE member_id = $1",
            self.user().id
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}
