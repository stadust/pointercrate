use crate::{config, documentation, ratelimit::Ratelimits};
use sqlx::{pool::Builder, PgConnection, Pool};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct PointercrateState {
    pub documentation_toc: Arc<String>,
    pub documentation_topics: Arc<HashMap<String, String>>,
    pub secret: Arc<Vec<u8>>,
    pub connection_pool: Pool<PgConnection>,
    pub ratelimits: Ratelimits,
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

        PointercrateState {
            documentation_toc,
            documentation_topics,
            connection_pool,
            secret: Arc::new(config::secret()),
            ratelimits: Ratelimits::initialize(),
        }
    }
}
