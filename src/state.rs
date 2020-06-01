use crate::{config, documentation, model::user::AuthenticatedUser, ratelimit::Ratelimits, Result};
use gdcf::Gdcf;
use gdcf_diesel::Cache;
use gdrs::BoomlingsClient;
use log::trace;
use reqwest::Client;
use sqlx::{
    pool::{Builder, PoolConnection},
    PgConnection, Pool, Transaction,
};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct PointercrateState {
    pub documentation_toc: Arc<String>,
    pub documentation_topics: Arc<HashMap<String, String>>,
    pub secret: Arc<Vec<u8>>,
    pub connection_pool: Pool<PgConnection>,
    pub ratelimits: Ratelimits,

    pub http_client: Client,
    pub webhook_url: Option<Arc<String>>,

    pub gdcf: Gdcf<BoomlingsClient, Cache>,
}

impl PointercrateState {
    /// Initializes the global pointercrate application state
    ///
    /// Loads in the API documentation files and values from config files. Also establishes database
    /// connections
    pub async fn initialize() -> PointercrateState {
        let documentation_toc = Arc::new(documentation::read_table_of_contents().unwrap());
        let documentation_topics = Arc::new(documentation::read_documentation_topics().unwrap());

        let connection_pool = Builder::default()
            .max_size(8)
            .max_lifetime(Some(Duration::from_secs(60 * 60 * 24)))
            .build(&config::database_url())
            .await
            .expect("Failed to connect to pointercrate database");

        let gdcf_url = std::env::var("GDCF_DATABASE_URL").expect("GDCF_DATABASE_URL is not set");

        let cache = Cache::postgres(gdcf_url).expect("GDCF database connection failed");
        let client = BoomlingsClient::new();

        cache.initialize().unwrap();

        PointercrateState {
            documentation_toc,
            documentation_topics,
            connection_pool,
            secret: Arc::new(config::secret()),
            ratelimits: Ratelimits::initialize(),
            http_client: Client::builder().build().expect("Failed to create reqwest client"),
            webhook_url: std::env::var("DISCORD_WEBHOOK").ok().map(Arc::new),
            gdcf: Gdcf::new(client, cache),
        }
    }

    /// Gets a connection from the connection pool
    pub async fn connection(&self) -> Result<PoolConnection<PgConnection>> {
        let mut connection = self.connection_pool.acquire().await?;

        audit_connection(&mut connection, 0).await?;

        Ok(connection)
    }

    pub async fn transaction(&self) -> Result<Transaction<PoolConnection<PgConnection>>> {
        let mut connection = self.connection_pool.begin().await?;

        audit_connection(&mut connection, 0).await?;

        Ok(connection)
    }

    /// Prepares this connection such that all audit log entries generated while using it are
    /// attributed to the givne authenticated user
    pub async fn audited_connection(&self, user: &AuthenticatedUser) -> Result<PoolConnection<PgConnection>> {
        let mut connection = self.connection_pool.acquire().await?;

        audit_connection(&mut connection, user.inner().id).await?;

        Ok(connection)
    }

    /// Prepares this transaction connection such that all audit log entries generated while using
    /// it are attributed to the givne authenticated user
    pub async fn audited_transaction(&self, user: &AuthenticatedUser) -> Result<Transaction<PoolConnection<PgConnection>>> {
        let mut connection = self.connection_pool.begin().await?;

        audit_connection(&mut connection, user.inner().id).await?;

        Ok(connection)
    }
}

pub async fn audit_connection(connection: &mut PgConnection, user_id: i32) -> Result<()> {
    trace!(
        "Creating connection of which usage will be attributed to user {} in audit logs",
        user_id
    );

    sqlx::query!("CREATE TEMPORARY TABLE IF NOT EXISTS active_user (id INTEGER)")
        .execute(connection)
        .await?;
    sqlx::query!("DELETE FROM active_user").execute(connection).await?;
    sqlx::query!("INSERT INTO active_user (id) VALUES ($1)", user_id)
        .execute(connection)
        .await?;

    Ok(())
}
