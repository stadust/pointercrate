use crate::{config, error::Result};
use log::trace;
use sqlx::{pool::PoolConnection, postgres::PgPoolOptions, PgConnection, Pool, Postgres, Transaction};

pub struct PointercratePool {
    connection_pool: Pool<Postgres>,
}

impl PointercratePool {
    pub fn clone_inner(&self) -> Pool<Postgres> {
        self.connection_pool.clone()
    }

    pub async fn init() -> Self {
        PointercratePool {
            connection_pool: PgPoolOptions::default()
                .max_connections(20)
                .max_lifetime(Some(std::time::Duration::from_secs(60 * 60 * 24)))
                .idle_timeout(Some(std::time::Duration::from_secs(60 * 5)))
                .connect(&config::database_url())
                .await
                .expect("Failed to connect to pointercrate database"),
        }
    }

    /// Gets a connection from the connection pool
    pub async fn connection(&self) -> Result<PoolConnection<Postgres>> {
        let mut connection = self.connection_pool.acquire().await?;

        audit_connection(&mut *connection, 0).await?;

        Ok(connection)
    }

    pub async fn transaction(&self) -> Result<Transaction<'static, Postgres>> {
        let mut connection = self.connection_pool.begin().await?;

        audit_connection(&mut *connection, 0).await?;

        Ok(connection)
    }
}

pub async fn audit_connection(connection: &mut PgConnection, user_id: i32) -> Result<()> {
    trace!(
        "Creating connection of which usage will be attributed to user {} in audit logs",
        user_id
    );

    sqlx::query!("CREATE TEMPORARY TABLE IF NOT EXISTS active_user (id INTEGER)")
        .execute(&mut *connection)
        .await?;
    sqlx::query!("DELETE FROM active_user").execute(&mut *connection).await?;
    sqlx::query!("INSERT INTO active_user (id) VALUES ($1)", user_id)
        .execute(connection)
        .await?;

    Ok(())
}
