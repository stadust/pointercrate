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

#[cfg(test)]
mod tests {
    // this is fine, as tests are always ran with --all-features
    #[cfg(feature = "legacy_accounts")]
    #[sqlx::test(migrations = "../migrations")]
    fn test_invalidate_all_tokens(mut conn: sqlx::pool::PoolConnection<sqlx::Postgres>) {
        use crate::auth::{legacy::Registration, AuthenticatedUser};

        let registration = Registration {
            name: "Patrick".to_string(),
            password: "very bad password".to_string(),
        };

        let patrick = AuthenticatedUser::register(registration, &mut conn).await.unwrap();
        let patricks_id = patrick.user().id;
        let patricks_clone = AuthenticatedUser::by_id(patricks_id, &mut conn).await.unwrap();

        let access_token = patrick.generate_programmatic_access_token();
        assert!(patricks_clone.validate_programmatic_access_token(&access_token).is_ok()); // sanity check

        patrick.invalidate_all_tokens(&mut conn).await.unwrap();

        let patricks_clone = AuthenticatedUser::by_id(patricks_id, &mut conn).await.unwrap();
        assert!(patricks_clone.validate_programmatic_access_token(&access_token).is_err());
    }
}
