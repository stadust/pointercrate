use sqlx::PgConnection;

use crate::auth::AuthenticatedUser;
use crate::Result;

impl AuthenticatedUser {
    pub async fn basic_auth(username: &str, password: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        log::info!("We are expected to perform basic authentication for user {}", username);

        Self::by_name(username, connection).await?.verify_password(password)
    }
}
