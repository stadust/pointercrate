//! Deleting your own account

use crate::{model::user::auth::AuthenticatedUser, Result};
use log::warn;
use sqlx::PgConnection;

impl AuthenticatedUser {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        warn!("Self-Deleting user account {}", self.user);

        self.user.delete(connection).await
    }
}
