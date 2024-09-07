//! Deleting your own account

use crate::{auth::AuthenticatedUser, error::Result};
use log::warn;
use sqlx::PgConnection;

impl AuthenticatedUser {
    pub async fn delete(self, connection: &mut PgConnection) -> Result<()> {
        warn!("Self-Deleting user account {}", self.inner());

        self.into_inner().delete(connection).await
    }
}
