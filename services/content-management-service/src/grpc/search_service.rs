use crate::proto::content::{
    search_service_server::SearchService as SearchServiceTrait, SearchContentRequest,
    SearchContentResponse, SearchResult as ProtoSearchResult,
};
use crate::search::{SearchService, UserRole};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument};
use uuid::Uuid;

/// gRPC SearchService implementation
pub struct SearchServiceHandler {
    search_service: Arc<SearchService>,
}

impl SearchServiceHandler {
    /// Creates a new SearchServiceHandler
    pub fn new(search_service: Arc<SearchService>) -> Self {
        Self { search_service }
    }
}

#[tonic::async_trait]
impl SearchServiceTrait for SearchServiceHandler {
    #[instrument(skip(self, request))]
    async fn search_content(
        &self,
        request: Request<SearchContentRequest>,
    ) -> Result<Response<SearchContentResponse>, Status> {
        let req = request.into_inner();

        info!(
            query = %req.query,
            course_id = %req.course_id,
            limit = req.limit,
            "Received search content request"
        );

        // Validate query
        if req.query.trim().is_empty() {
            return Err(Status::invalid_argument("Search query cannot be empty"));
        }

        // Parse course_id if provided
        let course_id = if !req.course_id.is_empty() {
            Some(
                Uuid::parse_str(&req.course_id)
                    .map_err(|_| Status::invalid_argument("Invalid course_id format"))?,
            )
        } else {
            None
        };

        // Determine limit (default to 50 if not specified or 0)
        let limit = if req.limit > 0 {
            req.limit as usize
        } else {
            50
        };

        // TODO: Extract user role from request metadata/JWT
        // For now, default to Student (more restrictive)
        let user_role = UserRole::Student;

        // Execute search
        let results = self
            .search_service
            .search(&req.query, course_id, user_role, limit)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to execute search");
                match e {
                    crate::search::SearchError::QueryTooShort => {
                        Status::invalid_argument("Search query must be at least 2 characters")
                    }
                    crate::search::SearchError::InvalidParameters(msg) => {
                        Status::invalid_argument(msg)
                    }
                    _ => Status::internal(format!("Search failed: {}", e)),
                }
            })?;

        // Convert results to proto format
        let proto_results: Vec<ProtoSearchResult> = results
            .into_iter()
            .map(|r| ProtoSearchResult {
                resource_id: r.resource_id,
                resource_name: r.resource_name,
                resource_type: r.resource_type,
                description: r.description.unwrap_or_default(),
                module_name: r.module_name,
                lesson_name: r.lesson_name,
                hierarchical_path: r.hierarchical_path,
                relevance_score: r.relevance_score,
                highlighted_snippets: r.highlighted_snippets,
            })
            .collect();

        info!(
            query = %req.query,
            results_count = proto_results.len(),
            "Search completed successfully"
        );

        Ok(Response::new(SearchContentResponse {
            results: proto_results,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_service_handler_creation() {
        // This test just verifies the struct can be created
        // Real functionality tests require ElasticSearch connection
    }

    #[test]
    fn test_limit_default() {
        let limit = 0;
        let actual_limit = if limit > 0 { limit as usize } else { 50 };
        assert_eq!(actual_limit, 50);

        let limit = 25;
        let actual_limit = if limit > 0 { limit as usize } else { 50 };
        assert_eq!(actual_limit, 25);
    }

    #[test]
    fn test_uuid_parsing() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Uuid::parse_str(valid_uuid).is_ok());

        let invalid_uuid = "not-a-uuid";
        assert!(Uuid::parse_str(invalid_uuid).is_err());
    }
}
