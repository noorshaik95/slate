use crate::progress::{CompletionStatus, ProgressReportFilter, ProgressTracker};
use crate::proto::content::{
    progress_service_server::ProgressService, GetProgressReportRequest, GetProgressRequest,
    LessonProgress, MarkCompleteRequest, ModuleProgress, ProgressReport, ProgressResponse,
    ResourceCompletion, ResourceProgress, StudentProgress,
};
use chrono::DateTime;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// gRPC service implementation for ProgressService
pub struct ProgressServiceImpl {
    tracker: Arc<ProgressTracker>,
}

impl ProgressServiceImpl {
    /// Creates a new ProgressServiceImpl
    pub fn new(tracker: Arc<ProgressTracker>) -> Self {
        Self { tracker }
    }

    /// Extracts user_id from gRPC metadata
    fn extract_user_id(request: &Request<impl std::fmt::Debug>) -> Result<Uuid, Status> {
        let metadata = request.metadata();
        let user_id_str = metadata
            .get("user-id")
            .ok_or_else(|| Status::unauthenticated("Missing user-id in metadata"))?
            .to_str()
            .map_err(|_| Status::invalid_argument("Invalid user-id format"))?;

        Uuid::parse_str(user_id_str)
            .map_err(|_| Status::invalid_argument("Invalid user-id UUID format"))
    }
}

#[tonic::async_trait]
impl ProgressService for ProgressServiceImpl {
    /// Marks a resource as complete or incomplete
    ///
    /// Requirements: 18.2
    /// - Wires ProgressTracker.mark_complete to gRPC handler
    /// - Adds request validation and error handling
    async fn mark_complete(
        &self,
        request: Request<MarkCompleteRequest>,
    ) -> Result<Response<()>, Status> {
        let user_id = Self::extract_user_id(&request)?;
        let req = request.into_inner();

        // Parse resource_id
        let resource_id = Uuid::parse_str(&req.resource_id)
            .map_err(|_| Status::invalid_argument("Invalid resource_id format"))?;

        // Mark complete or incomplete
        self.tracker
            .mark_complete(user_id, resource_id, req.completed)
            .await
            .map_err(|e| match e {
                crate::progress::ProgressError::ResourceNotFound(_) => {
                    Status::not_found(e.to_string())
                }
                crate::progress::ProgressError::ResourceNotPublished => {
                    Status::failed_precondition(e.to_string())
                }
                _ => Status::internal(format!("Failed to mark complete: {}", e)),
            })?;

        Ok(Response::new(()))
    }

    /// Gets progress for a student in a course
    ///
    /// Requirements: 18.2
    /// - Wires ProgressTracker.get_progress to gRPC handler
    /// - Adds request validation and error handling
    async fn get_progress(
        &self,
        request: Request<GetProgressRequest>,
    ) -> Result<Response<ProgressResponse>, Status> {
        let user_id = Self::extract_user_id(&request)?;
        let req = request.into_inner();

        // Parse course_id
        let course_id = Uuid::parse_str(&req.course_id)
            .map_err(|_| Status::invalid_argument("Invalid course_id format"))?;

        // Get progress
        let progress = self
            .tracker
            .get_progress(user_id, course_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get progress: {}", e)))?;

        // Convert to proto response
        let modules = progress
            .modules
            .into_iter()
            .map(|m| {
                let lessons = m
                    .lessons
                    .into_iter()
                    .map(|l| {
                        let resources = l
                            .resources
                            .into_iter()
                            .map(|r| ResourceProgress {
                                resource_id: r.resource_id.to_string(),
                                resource_name: r.resource_name,
                                completed: r.completed,
                                completed_at: r.completed_at.map(|dt| prost_types::Timestamp {
                                    seconds: dt.timestamp(),
                                    nanos: dt.timestamp_subsec_nanos() as i32,
                                }),
                            })
                            .collect();

                        LessonProgress {
                            lesson_id: l.lesson_id.to_string(),
                            lesson_name: l.lesson_name,
                            percentage: l.percentage,
                            resources,
                        }
                    })
                    .collect();

                ModuleProgress {
                    module_id: m.module_id.to_string(),
                    module_name: m.module_name,
                    percentage: m.percentage,
                    lessons,
                }
            })
            .collect();

        Ok(Response::new(ProgressResponse {
            overall_percentage: progress.overall_percentage,
            modules,
        }))
    }

    /// Gets progress report for instructors
    ///
    /// Requirements: 18.2
    /// - Wires ProgressTracker.generate_report to gRPC handler
    /// - Adds request validation and error handling
    async fn get_progress_report(
        &self,
        request: Request<GetProgressReportRequest>,
    ) -> Result<Response<ProgressReport>, Status> {
        let req = request.into_inner();

        // Parse course_id
        let course_id = Uuid::parse_str(&req.course_id)
            .map_err(|_| Status::invalid_argument("Invalid course_id format"))?;

        // Parse optional module_id
        let module_id = if !req.module_id.is_empty() {
            Some(
                Uuid::parse_str(&req.module_id)
                    .map_err(|_| Status::invalid_argument("Invalid module_id format"))?,
            )
        } else {
            None
        };

        // Parse optional date range
        let start_date = if !req.start_date.is_empty() {
            Some(
                DateTime::parse_from_rfc3339(&req.start_date)
                    .map_err(|_| Status::invalid_argument("Invalid start_date format"))?
                    .with_timezone(&chrono::Utc),
            )
        } else {
            None
        };

        let end_date = if !req.end_date.is_empty() {
            Some(
                DateTime::parse_from_rfc3339(&req.end_date)
                    .map_err(|_| Status::invalid_argument("Invalid end_date format"))?
                    .with_timezone(&chrono::Utc),
            )
        } else {
            None
        };

        // Parse completion status
        let completion_status = CompletionStatus::from_str(&req.completion_status);

        // Build filter
        let filter = ProgressReportFilter {
            course_id,
            start_date,
            end_date,
            module_id,
            completion_status,
        };

        // Generate report
        let report = self
            .tracker
            .generate_report(filter)
            .await
            .map_err(|e| Status::internal(format!("Failed to generate report: {}", e)))?;

        // Convert to proto response
        let students = report
            .students
            .into_iter()
            .map(|s| {
                let resources = s
                    .resources
                    .into_iter()
                    .map(|r| ResourceCompletion {
                        resource_id: r.resource_id.to_string(),
                        resource_name: r.resource_name,
                        completed: r.completed,
                        completed_at: r.completed_at.map(|dt| prost_types::Timestamp {
                            seconds: dt.timestamp(),
                            nanos: dt.timestamp_subsec_nanos() as i32,
                        }),
                        time_spent_seconds: r.time_spent_seconds,
                    })
                    .collect();

                StudentProgress {
                    student_id: s.student_id.to_string(),
                    completion_percentage: s.completion_percentage,
                    resources,
                }
            })
            .collect();

        Ok(Response::new(ProgressReport {
            total_students: report.total_students,
            average_completion_percentage: report.average_completion_percentage,
            students,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_service_creation() {
        // This is a placeholder test to ensure the module compiles
        // Real tests would require database setup
    }
}
