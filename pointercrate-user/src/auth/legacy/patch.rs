use pointercrate_core::error::CoreError;
use sqlx::PgConnection;

use crate::error::UserError;

use super::LegacyAuthenticatedUser;

impl LegacyAuthenticatedUser {
    pub async fn set_password(&mut self, password: String, connection: &mut PgConnection) -> Result<(), UserError> {
        Self::validate_password(&password)?;

        log::info!("Setting new password for user {}", self.user);

        // it is safe to unwrap here because the only errors that can happen are
        // 'BcryptError::CostNotAllowed' (won't happen because DEFAULT_COST is obviously allowed)
        // or errors that happen during internally parsing the hash the library itself just
        // generated. Obviously, an error there is a bug in the library, so return InternalServerError
        self.password_hash =
            bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(|_| CoreError::internal_server_error("bcrypt library bug"))?;

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
