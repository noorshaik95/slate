use crate::config::ElasticsearchConfig;
use crate::search::errors::SearchError;
use elasticsearch::{
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    indices::{IndicesCreateParts, IndicesExistsParts},
    Elasticsearch,
};
use serde_json::json;
use std::time::Duration;
use tracing::{error, info, warn};
use url::Url;

/// ElasticSearch client wrapper with retry logic
pub struct ElasticsearchClient {
    client: Elasticsearch,
    index_name: String,
    max_retries: u32,
    timeout: Duration,
}

impl ElasticsearchClient {
    /// Creates a new ElasticSearch client
    pub fn new(config: &ElasticsearchConfig) -> Result<Self, SearchError> {
        let url = Url::parse(&config.url).map_err(|e| {
            SearchError::ConnectionError(format!("Invalid ElasticSearch URL: {}", e))
        })?;

        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| SearchError::ConnectionError(format!("Failed to build transport: {}", e)))?;

        let client = Elasticsearch::new(transport);

        Ok(Self {
            client,
            index_name: config.index.clone(),
            max_retries: config.max_retries,
            timeout: Duration::from_secs(config.timeout_seconds),
        })
    }

    /// Ensures the index exists with proper mappings
    pub async fn ensure_index(&self) -> Result<(), SearchError> {
        // Check if index exists
        let exists_response = self
            .client
            .indices()
            .exists(IndicesExistsParts::Index(&[&self.index_name]))
            .send()
            .await
            .map_err(|e| SearchError::IndexError(format!("Failed to check index existence: {}", e)))?;

        if exists_response.status_code().is_success() {
            info!("ElasticSearch index '{}' already exists", self.index_name);
            return Ok(());
        }

        // Create index with mappings
        info!("Creating ElasticSearch index '{}'", self.index_name);
        
        let mappings = json!({
            "mappings": {
                "properties": {
                    "resource_id": { "type": "keyword" },
                    "resource_name": { "type": "text", "analyzer": "standard" },
                    "resource_type": { "type": "keyword" },
                    "description": { "type": "text", "analyzer": "standard" },
                    "module_id": { "type": "keyword" },
                    "module_name": { "type": "text", "analyzer": "standard" },
                    "lesson_id": { "type": "keyword" },
                    "lesson_name": { "type": "text", "analyzer": "standard" },
                    "course_id": { "type": "keyword" },
                    "hierarchical_path": { "type": "text" },
                    "published": { "type": "boolean" },
                    "created_at": { "type": "date" },
                    "updated_at": { "type": "date" }
                }
            },
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 0,
                "analysis": {
                    "analyzer": {
                        "standard": {
                            "type": "standard"
                        }
                    }
                }
            }
        });

        let create_response = self
            .client
            .indices()
            .create(IndicesCreateParts::Index(&self.index_name))
            .body(mappings)
            .send()
            .await
            .map_err(|e| SearchError::IndexError(format!("Failed to create index: {}", e)))?;

        if !create_response.status_code().is_success() {
            let error_text = create_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SearchError::IndexError(format!(
                "Failed to create index: {}",
                error_text
            )));
        }

        info!("Successfully created ElasticSearch index '{}'", self.index_name);
        Ok(())
    }

    /// Executes an operation with retry logic
    pub async fn with_retry<F, T, Fut>(&self, operation: F) -> Result<T, SearchError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, SearchError>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if attempts <= self.max_retries {
                        let backoff = Duration::from_millis(100 * 2_u64.pow(attempts - 1));
                        warn!(
                            "ElasticSearch operation failed (attempt {}/{}), retrying in {:?}",
                            attempts,
                            self.max_retries + 1,
                            backoff
                        );
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            SearchError::InternalError("Operation failed with no error details".to_string())
        }))
    }

    /// Gets the underlying ElasticSearch client
    pub fn client(&self) -> &Elasticsearch {
        &self.client
    }

    /// Gets the index name
    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    /// Gets the maximum number of retries
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Gets the timeout duration
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = ElasticsearchConfig {
            url: "http://localhost:9200".to_string(),
            index: "test_content".to_string(),
            max_retries: 2,
            timeout_seconds: 30,
        };

        let client = ElasticsearchClient::new(&config);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.index_name(), "test_content");
        assert_eq!(client.max_retries(), 2);
        assert_eq!(client.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_invalid_url() {
        let config = ElasticsearchConfig {
            url: "invalid-url".to_string(),
            index: "test_content".to_string(),
            max_retries: 2,
            timeout_seconds: 30,
        };

        let client = ElasticsearchClient::new(&config);
        assert!(client.is_err());
    }
}
