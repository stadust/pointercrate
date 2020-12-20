use crate::{config, documentation, gd::PgCache, model::user::AuthenticatedUser, ratelimit::Ratelimits, Result};
use chrono::Duration;
use log::trace;
use reqwest::Client;
use sqlx::{pool::PoolConnection, postgres::PgPoolOptions, PgConnection, Pool, Postgres, Transaction};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct PointercrateState {
    pub documentation_toc: Arc<String>,
    pub documentation_topics: Arc<HashMap<String, String>>,

    pub guidelines_toc: Arc<String>,
    pub guidelines_topics: Arc<HashMap<String, String>>,

    pub secret: Arc<Vec<u8>>,
    pub connection_pool: Pool<Postgres>,
    pub ratelimits: Ratelimits,

    pub http_client: Client,
    pub webhook_url: Option<Arc<String>>,
    pub gd_integration: PgCache,
}

impl PointercrateState {
    /// Initializes the global pointercrate application state
    ///
    /// Loads in the API documentation files and values from config files. Also establishes database
    /// connections
    pub async fn initialize() -> PointercrateState {
        let documentation_toc = Arc::new(documentation::read_table_of_contents(&config::documentation_location()).unwrap());
        let documentation_topics = Arc::new(documentation::read_topics(&config::documentation_location()).unwrap());

        let guidelines_toc = Arc::new(documentation::read_table_of_contents(&config::guidelines_location()).unwrap());
        let guidelines_topics = Arc::new(documentation::read_topics(&config::guidelines_location()).unwrap());

        let connection_pool = PgPoolOptions::default()
            .max_connections(8)
            .max_lifetime(Some(std::time::Duration::from_secs(60 * 60 * 24)))
            .connect(&config::database_url())
            .await
            .expect("Failed to connect to pointercrate database");

        PointercrateState {
            gd_integration: PgCache::new(connection_pool.clone(), Duration::minutes(30)),
            documentation_toc,
            documentation_topics,
            guidelines_toc,
            guidelines_topics,
            connection_pool,
            secret: Arc::new(config::secret()),
            ratelimits: Ratelimits::initialize(),
            http_client: Client::builder().build().expect("Failed to create reqwest client"),
            webhook_url: std::env::var("DISCORD_WEBHOOK").ok().map(Arc::new),
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

    /// Prepares this connection such that all audit log entries generated while using it are
    /// attributed to the givne authenticated user
    pub async fn audited_connection(&self, user: &AuthenticatedUser) -> Result<PoolConnection<Postgres>> {
        let mut connection = self.connection_pool.acquire().await?;

        audit_connection(&mut *connection, user.inner().id).await?;

        Ok(connection)
    }

    /// Prepares this transaction connection such that all audit log entries generated while using
    /// it are attributed to the given authenticated user
    pub async fn audited_transaction(&self, user: &AuthenticatedUser) -> Result<Transaction<'static, Postgres>> {
        let mut connection = self.connection_pool.begin().await?;

        audit_connection(&mut *connection, user.inner().id).await?;

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
