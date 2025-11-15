use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{info, warn};

/// DatabasePool manages PostgreSQL connections
#[derive(Clone)]
pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    /// Minimum number of connections in the pool
    pub const MIN_CONNECTIONS: u32 = 5;

    /// Maximum number of connections in the pool
    pub const MAX_CONNECTIONS: u32 = 20;

    /// Connection timeout in seconds
    pub const CONNECTION_TIMEOUT_SECS: u64 = 30;

    /// Maximum retry attempts for database connection
    pub const MAX_RETRY_ATTEMPTS: u32 = 5;

    /// Initial retry delay in milliseconds
    pub const INITIAL_RETRY_DELAY_MS: u64 = 100;

    /// Creates a new database pool with the given connection URL
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .min_connections(Self::MIN_CONNECTIONS)
            .max_connections(Self::MAX_CONNECTIONS)
            .acquire_timeout(Duration::from_secs(Self::CONNECTION_TIMEOUT_SECS))
            .connect(database_url)
            .await
            .context("Failed to create database pool")?;

        Ok(Self { pool })
    }

    /// Creates a new database pool with retry logic and exponential backoff
    pub async fn new_with_retry(database_url: &str) -> Result<Self> {
        let mut retry_count = 0;
        let mut delay_ms = Self::INITIAL_RETRY_DELAY_MS;

        loop {
            match Self::new(database_url).await {
                Ok(pool) => {
                    info!("Successfully connected to database");
                    return Ok(pool);
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= Self::MAX_RETRY_ATTEMPTS {
                        return Err(e).context(format!(
                            "Failed to connect to database after {} attempts",
                            Self::MAX_RETRY_ATTEMPTS
                        ));
                    }

                    warn!(
                        "Failed to connect to database (attempt {}/{}): {}. Retrying in {}ms...",
                        retry_count, Self::MAX_RETRY_ATTEMPTS, e, delay_ms
                    );

                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    
                    // Exponential backoff with max 60 seconds
                    delay_ms = std::cmp::min(delay_ms * 2, 60_000);
                }
            }
        }
    }

    /// Returns a reference to the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Runs database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations...");
        
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run database migrations")?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Checks if the database connection is healthy
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Database health check failed")?;
        Ok(())
    }

    /// Closes the database pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
}
