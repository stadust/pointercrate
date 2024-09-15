//! Deleting your own account

use crate::{auth::AuthenticatedUser, error::Result};
use log::warn;
use sqlx::PgConnection;

impl AuthenticatedUser {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        warn!("Self-Deleting user account {}", self.user());

        self.into_user().delete(connection).await
    }
}
