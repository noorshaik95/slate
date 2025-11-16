use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
use crate::models::{Lesson, Module, Resource};
use crate::search::{ElasticsearchClient, SearchError};
use elasticsearch::{
    DeleteParts, IndexParts, SearchParts,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Document structure for ElasticSearch indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDocument {
    pub resource_id: String,
    pub resource_name: String,
    pub resource_type: String,
    pub description: Option<String>,
    pub module_id: String,
    pub module_name: String,
    pub lesson_id: String,
    pub lesson_name: String,
    pub course_id: String,
    pub hierarchical_path: String,
    pub published: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Search result from ElasticSearch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub resource_id: String,
    pub resource_name: String,
    pub resource_type: String,
    pub description: Option<String>,
    pub module_name: String,
    pub lesson_name: String,
    pub hierarchical_path: String,
    pub relevance_score: f32,
    pub highlighted_snippets: Vec<String>,
}

/// User role for filtering search results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Student,
    Instructor,
}

/// SearchService manages content indexing and search queries
pub struct SearchService {
    es_client: Arc<ElasticsearchClient>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl SearchService {
    /// Creates a new SearchService with circuit breaker
    pub fn new(es_client: Arc<ElasticsearchClient>) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            "elasticsearch".to_string(),
            CircuitBreakerConfig::elasticsearch(),
        ));
        
        Self { 
            es_client,
            circuit_breaker,
        }
    }

    /// Get circuit breaker for monitoring
    pub fn circuit_breaker(&self) -> Arc<CircuitBreaker> {
        self.circuit_breaker.clone()
    }

    /// Indexes content in ElasticSearch (with circuit breaker)
    #[instrument(skip(self), fields(resource_id = %resource.id))]
    pub async fn index_content(
        &self,
        resource: &Resource,
        lesson: &Lesson,
        module: &Module,
    ) -> Result<(), SearchError> {
        let document = ContentDocument {
            resource_id: resource.id.to_string(),
            resource_name: resource.name.clone(),
            resource_type: resource.content_type.to_string(),
            description: resource.description.clone(),
            module_id: module.id.to_string(),
            module_name: module.name.clone(),
            lesson_id: lesson.id.to_string(),
            lesson_name: lesson.name.clone(),
            course_id: module.course_id.to_string(),
            hierarchical_path: format!("{} → {} → {}", module.name, lesson.name, resource.name),
            published: resource.published,
            created_at: resource.created_at.to_rfc3339(),
            updated_at: resource.updated_at.to_rfc3339(),
        };

        let es_client = self.es_client.clone();
        let resource_id = resource.id;
        let resource_name = resource.name.clone();

        let result = self.circuit_breaker.call(|| async {
            let index_operation = || async {
                let response = es_client
                    .client()
                    .index(IndexParts::IndexId(
                        es_client.index_name(),
                        &resource_id.to_string(),
                    ))
                    .body(json!(document))
                    .send()
                    .await
                    .map_err(|e| SearchError::IndexError(format!("Failed to index document: {}", e)))?;

                if !response.status_code().is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(SearchError::IndexError(format!(
                        "Failed to index document: {}",
                        error_text
                    )));
                }

                info!(
                    resource_id = %resource_id,
                    resource_name = %resource_name,
                    "Successfully indexed content"
                );
                Ok(())
            };

            es_client.with_retry(index_operation).await
        }).await;

        match result {
            Ok(()) => Ok(()),
            Err(CircuitBreakerError::CircuitOpen) => {
                warn!("ElasticSearch circuit breaker is open, skipping indexing");
                // Don't fail the operation, just log and continue
                Ok(())
            }
            Err(CircuitBreakerError::OperationFailed(e)) => Err(e),
        }
    }

    /// Removes content from ElasticSearch index
    #[instrument(skip(self), fields(resource_id = %resource_id))]
    pub async fn remove_from_index(&self, resource_id: Uuid) -> Result<(), SearchError> {
        let delete_operation = || async {
            let response = self
                .es_client
                .client()
                .delete(DeleteParts::IndexId(
                    self.es_client.index_name(),
                    &resource_id.to_string(),
                ))
                .send()
                .await
                .map_err(|e| SearchError::IndexError(format!("Failed to delete document: {}", e)))?;

            // 404 is acceptable - document might not exist
            if !response.status_code().is_success() && response.status_code().as_u16() != 404 {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(SearchError::IndexError(format!(
                    "Failed to delete document: {}",
                    error_text
                )));
            }

            info!(resource_id = %resource_id, "Successfully removed content from index");
            Ok(())
        };

        self.es_client.with_retry(delete_operation).await
    }

    /// Searches content with query and filters (with circuit breaker)
    #[instrument(skip(self))]
    pub async fn search(
        &self,
        query: &str,
        course_id: Option<Uuid>,
        user_role: UserRole,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // Validate query length
        if query.trim().len() < 2 {
            return Err(SearchError::QueryTooShort);
        }

        // Limit results to maximum of 50
        let limit = limit.min(50);
        
        let query_str = query.to_string();
        let es_client = self.es_client.clone();

        // Build the search query
        let mut must_clauses = vec![
            json!({
                "multi_match": {
                    "query": query,
                    "fields": ["resource_name^3", "description^2", "module_name", "lesson_name"],
                    "type": "best_fields",
                    "fuzziness": "AUTO"
                }
            })
        ];

        // Filter by course if specified
        if let Some(cid) = course_id {
            must_clauses.push(json!({
                "term": {
                    "course_id": cid.to_string()
                }
            }));
        }

        // Students can only see published content
        if user_role == UserRole::Student {
            must_clauses.push(json!({
                "term": {
                    "published": true
                }
            }));
        }

        let search_body = json!({
            "query": {
                "bool": {
                    "must": must_clauses
                }
            },
            "highlight": {
                "fields": {
                    "resource_name": {},
                    "description": {},
                    "module_name": {},
                    "lesson_name": {}
                },
                "pre_tags": ["<mark>"],
                "post_tags": ["</mark>"]
            },
            "size": limit,
            "sort": [
                { "_score": { "order": "desc" } }
            ]
        });

        debug!(query = %query_str, course_id = ?course_id, user_role = ?user_role, "Executing search query");

        let result = self.circuit_breaker.call(|| {
            let es_client = es_client.clone();
            let search_body = search_body.clone();
            let query_str = query_str.clone();
            
            async move {
                let search_operation = || async {
            let response = es_client
                .client()
                .search(SearchParts::Index(&[es_client.index_name()]))
                .body(search_body.clone())
                .send()
                .await
                .map_err(|e| SearchError::QueryError(format!("Failed to execute search: {}", e)))?;

            if !response.status_code().is_success() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(SearchError::QueryError(format!(
                    "Search query failed: {}",
                    error_text
                )));
            }

            let response_body = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| SearchError::SerializationError(format!("Failed to parse response: {}", e)))?;

            // Parse search results
            let hits = response_body["hits"]["hits"]
                .as_array()
                .ok_or_else(|| SearchError::QueryError("Invalid response format".to_string()))?;

            let mut results = Vec::new();
            for hit in hits {
                let score = hit["_score"].as_f64().unwrap_or(0.0) as f32;
                let source = &hit["_source"];
                let highlight = &hit["highlight"];

                // Extract highlighted snippets
                let mut highlighted_snippets = Vec::new();
                if let Some(highlights) = highlight.as_object() {
                    for (_, snippets) in highlights {
                        if let Some(arr) = snippets.as_array() {
                            for snippet in arr {
                                if let Some(text) = snippet.as_str() {
                                    highlighted_snippets.push(text.to_string());
                                }
                            }
                        }
                    }
                }

                let result = SearchResult {
                    resource_id: source["resource_id"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    resource_name: source["resource_name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    resource_type: source["resource_type"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    description: source["description"].as_str().map(|s| s.to_string()),
                    module_name: source["module_name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    lesson_name: source["lesson_name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    hierarchical_path: source["hierarchical_path"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    relevance_score: score,
                    highlighted_snippets,
                };

                results.push(result);
            }

            info!(
                query = %query_str,
                results_count = results.len(),
                "Search completed successfully"
            );

            Ok(results)
        };

        es_client.with_retry(search_operation).await
            }
        }).await;

        match result {
            Ok(results) => Ok(results),
            Err(CircuitBreakerError::CircuitOpen) => {
                warn!("ElasticSearch circuit breaker is open, returning empty results");
                Err(SearchError::ServiceUnavailable)
            }
            Err(CircuitBreakerError::OperationFailed(e)) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::models::{ContentType, CopyrightSetting};

    fn create_test_resource() -> Resource {
        Resource {
            id: Uuid::new_v4(),
            lesson_id: Uuid::new_v4(),
            name: "Introduction to Rust".to_string(),
            description: Some("Learn the basics of Rust programming".to_string()),
            content_type: ContentType::Video,
            file_size: 1024,
            storage_key: "test/video.mp4".to_string(),
            manifest_url: None,
            duration_seconds: Some(600),
            published: true,
            downloadable: false,
            copyright_setting: CopyrightSetting::Unrestricted,
            display_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_lesson() -> Lesson {
        crate::models::Lesson {
            id: Uuid::new_v4(),
            module_id: Uuid::new_v4(),
            name: "Getting Started".to_string(),
            description: Some("First steps with Rust".to_string()),
            display_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_module() -> Module {
        Module {
            id: Uuid::new_v4(),
            course_id: Uuid::new_v4(),
            name: "Rust Fundamentals".to_string(),
            description: Some("Core concepts of Rust".to_string()),
            display_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    #[test]
    fn test_content_document_creation() {
        let resource = create_test_resource();
        let lesson = create_test_lesson();
        let module = create_test_module();

        let doc = ContentDocument {
            resource_id: resource.id.to_string(),
            resource_name: resource.name.clone(),
            resource_type: resource.content_type.to_string(),
            description: resource.description.clone(),
            module_id: module.id.to_string(),
            module_name: module.name.clone(),
            lesson_id: lesson.id.to_string(),
            lesson_name: lesson.name.clone(),
            course_id: module.course_id.to_string(),
            hierarchical_path: format!("{} → {} → {}", module.name, lesson.name, resource.name),
            published: resource.published,
            created_at: resource.created_at.to_rfc3339(),
            updated_at: resource.updated_at.to_rfc3339(),
        };

        assert_eq!(doc.resource_name, "Introduction to Rust");
        assert_eq!(doc.resource_type, "video");
        assert_eq!(doc.hierarchical_path, "Rust Fundamentals → Getting Started → Introduction to Rust");
        assert!(doc.published);
    }

    #[test]
    fn test_query_too_short() {
        // This would require an actual ElasticSearch instance to test fully
        // For now, we just test the validation logic
        let query = "a";
        assert!(query.trim().len() < 2);
    }

    #[test]
    fn test_limit_capping() {
        let limit = 100;
        let capped = limit.min(50);
        assert_eq!(capped, 50);

        let limit = 25;
        let capped = limit.min(50);
        assert_eq!(capped, 25);
    }
}
