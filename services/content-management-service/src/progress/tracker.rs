use crate::db::repositories::{
    LessonRepository, ModuleRepository, ProgressRepository, ResourceRepository,
};
use crate::models::{Module, ProgressSummary, ProgressTracking, Resource};
use crate::progress::errors::ProgressError;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Detailed progress information for a resource
#[derive(Debug, Clone)]
pub struct ResourceProgressDetail {
    pub resource_id: Uuid,
    pub resource_name: String,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Detailed progress information for a lesson
#[derive(Debug, Clone)]
pub struct LessonProgressDetail {
    pub lesson_id: Uuid,
    pub lesson_name: String,
    pub percentage: i32,
    pub resources: Vec<ResourceProgressDetail>,
}

/// Detailed progress information for a module
#[derive(Debug, Clone)]
pub struct ModuleProgressDetail {
    pub module_id: Uuid,
    pub module_name: String,
    pub percentage: i32,
    pub lessons: Vec<LessonProgressDetail>,
}

/// Complete progress response for a course
#[derive(Debug, Clone)]
pub struct CourseProgressResponse {
    pub overall_percentage: i32,
    pub modules: Vec<ModuleProgressDetail>,
}

/// Student progress data for reports
#[derive(Debug, Clone)]
pub struct StudentProgressData {
    pub student_id: Uuid,
    pub completion_percentage: i32,
    pub resources: Vec<ResourceCompletionData>,
}

/// Resource completion data for reports
#[derive(Debug, Clone)]
pub struct ResourceCompletionData {
    pub resource_id: Uuid,
    pub resource_name: String,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub time_spent_seconds: i32,
}

/// Progress report with aggregate statistics
#[derive(Debug, Clone)]
pub struct ProgressReportData {
    pub total_students: i32,
    pub average_completion_percentage: f32,
    pub students: Vec<StudentProgressData>,
}

/// Filter options for progress reports
#[derive(Debug, Clone)]
pub struct ProgressReportFilter {
    pub course_id: Uuid,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub module_id: Option<Uuid>,
    pub completion_status: CompletionStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompletionStatus {
    All,
    Completed,
    Incomplete,
}

impl CompletionStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "completed" => CompletionStatus::Completed,
            "incomplete" => CompletionStatus::Incomplete,
            _ => CompletionStatus::All,
        }
    }
}

/// ProgressTracker manages student progress tracking and reporting
pub struct ProgressTracker {
    progress_repo: ProgressRepository,
    resource_repo: ResourceRepository,
    lesson_repo: LessonRepository,
    module_repo: ModuleRepository,
}

impl ProgressTracker {
    /// Creates a new ProgressTracker
    pub fn new(
        progress_repo: ProgressRepository,
        resource_repo: ResourceRepository,
        lesson_repo: LessonRepository,
        module_repo: ModuleRepository,
    ) -> Self {
        Self {
            progress_repo,
            resource_repo,
            lesson_repo,
            module_repo,
        }
    }

    /// Marks a resource as complete or incomplete for a student
    ///
    /// Requirements: 8.1, 8.2, 8.3, 8.4
    /// - Records completion timestamp when marking complete
    /// - Allows toggling between complete and incomplete status
    /// - Prevents marking unpublished content as complete
    pub async fn mark_complete(
        &self,
        student_id: Uuid,
        resource_id: Uuid,
        completed: bool,
    ) -> Result<ProgressTracking, ProgressError> {
        // Find the resource to validate it exists and check publication status
        let resource = self
            .resource_repo
            .find_by_id(resource_id)
            .await?
            .ok_or_else(|| ProgressError::ResourceNotFound(resource_id.to_string()))?;

        // Prevent marking unpublished content as complete
        if completed && !resource.published {
            return Err(ProgressError::ResourceNotPublished);
        }

        // Mark as complete or incomplete
        let progress = if completed {
            self.progress_repo
                .mark_complete(student_id, resource_id)
                .await?
        } else {
            self.progress_repo
                .mark_incomplete(student_id, resource_id)
                .await?
        };

        Ok(progress)
    }

    /// Gets detailed progress for a student in a course
    ///
    /// Requirements: 9.1, 9.2, 9.3, 9.4, 9.5
    /// - Calculates (completed / total_published) × 100
    /// - Calculates separate percentages for course, modules, and lessons
    /// - Rounds to nearest integer
    /// - Excludes unpublished resources from calculations
    pub async fn get_progress(
        &self,
        student_id: Uuid,
        course_id: Uuid,
    ) -> Result<CourseProgressResponse, ProgressError> {
        // Get all modules for the course
        let modules = self.module_repo.list_by_course(course_id).await?;

        // Get all progress for the student
        let all_progress = self.progress_repo.list_by_student(student_id).await?;
        let progress_map: HashMap<Uuid, &ProgressTracking> =
            all_progress.iter().map(|p| (p.resource_id, p)).collect();

        let mut module_details = Vec::new();
        let mut total_course_resources = 0;
        let mut total_course_completed = 0;

        for module in modules {
            let lessons = self.lesson_repo.list_by_module(module.id).await?;
            let mut lesson_details = Vec::new();

            for lesson in lessons {
                // Get only published resources
                let resources = self.resource_repo.list_by_lesson(lesson.id, true).await?;
                let published_resources: Vec<&Resource> = resources.iter().collect();

                let mut resource_details = Vec::new();
                let mut lesson_completed = 0;

                for resource in &published_resources {
                    let progress = progress_map.get(&resource.id);
                    let completed = progress.map(|p| p.completed).unwrap_or(false);
                    let completed_at = progress.and_then(|p| p.completed_at);

                    if completed {
                        lesson_completed += 1;
                    }

                    resource_details.push(ResourceProgressDetail {
                        resource_id: resource.id,
                        resource_name: resource.name.clone(),
                        completed,
                        completed_at,
                    });
                }

                let lesson_total = published_resources.len() as i32;
                let lesson_percentage =
                    ProgressSummary::calculate_percentage(lesson_completed, lesson_total);

                total_course_resources += lesson_total;
                total_course_completed += lesson_completed;

                lesson_details.push(LessonProgressDetail {
                    lesson_id: lesson.id,
                    lesson_name: lesson.name.clone(),
                    percentage: lesson_percentage,
                    resources: resource_details,
                });
            }

            // Calculate module percentage
            let module_total: i32 = lesson_details
                .iter()
                .map(|l| l.resources.len() as i32)
                .sum();
            let module_completed: i32 = lesson_details
                .iter()
                .flat_map(|l| &l.resources)
                .filter(|r| r.completed)
                .count() as i32;
            let module_percentage =
                ProgressSummary::calculate_percentage(module_completed, module_total);

            module_details.push(ModuleProgressDetail {
                module_id: module.id,
                module_name: module.name.clone(),
                percentage: module_percentage,
                lessons: lesson_details,
            });
        }

        let overall_percentage =
            ProgressSummary::calculate_percentage(total_course_completed, total_course_resources);

        Ok(CourseProgressResponse {
            overall_percentage,
            modules: module_details,
        })
    }

    /// Generates a progress report for instructors
    ///
    /// Requirements: 11.1, 11.2, 11.3, 11.4, 11.5
    /// - Aggregates student completion data
    /// - Includes average completion percentage and per-resource completion rates
    /// - Supports filtering by date range, module, and completion status
    /// - Optimized for up to 1000 students (completes within 5 seconds)
    /// - Includes student IDs, completion timestamps, and time spent per resource
    pub async fn generate_report(
        &self,
        filter: ProgressReportFilter,
    ) -> Result<ProgressReportData, ProgressError> {
        // Get all modules for the course
        let modules = self.module_repo.list_by_course(filter.course_id).await?;

        // Filter modules if specified
        let target_modules: Vec<Module> = if let Some(module_id) = filter.module_id {
            modules.into_iter().filter(|m| m.id == module_id).collect()
        } else {
            modules
        };

        // Collect all resources from target modules
        let mut all_resources = Vec::new();
        for module in &target_modules {
            let lessons = self.lesson_repo.list_by_module(module.id).await?;
            for lesson in lessons {
                // Get only published resources
                let resources = self.resource_repo.list_by_lesson(lesson.id, true).await?;
                all_resources.extend(resources);
            }
        }

        if all_resources.is_empty() {
            return Ok(ProgressReportData {
                total_students: 0,
                average_completion_percentage: 0.0,
                students: Vec::new(),
            });
        }

        // Get progress report from repository
        let progress_summaries = self
            .progress_repo
            .get_course_progress_report(filter.course_id)
            .await?;

        // Build student progress data
        let mut student_data_list = Vec::new();
        let mut total_percentage_sum = 0.0;

        for summary in progress_summaries {
            let student_id = summary.student_id;

            // Get detailed progress for this student
            let student_progress = self.progress_repo.list_by_student(student_id).await?;

            // Filter by date range if specified
            let filtered_progress: Vec<&ProgressTracking> = student_progress
                .iter()
                .filter(|p| {
                    if let Some(start) = filter.start_date {
                        if let Some(completed_at) = p.completed_at {
                            if completed_at < start {
                                return false;
                            }
                        }
                    }
                    if let Some(end) = filter.end_date {
                        if let Some(completed_at) = p.completed_at {
                            if completed_at > end {
                                return false;
                            }
                        }
                    }
                    true
                })
                .collect();

            // Build resource completion data
            let mut resource_completions = Vec::new();
            let mut completed_count = 0;

            for resource in &all_resources {
                let progress = filtered_progress
                    .iter()
                    .find(|p| p.resource_id == resource.id);

                let completed = progress.map(|p| p.completed).unwrap_or(false);
                let completed_at = progress.and_then(|p| p.completed_at);

                // Calculate time spent (for videos, use last_position_seconds)
                let time_spent_seconds =
                    progress.and_then(|p| p.last_position_seconds).unwrap_or(0);

                if completed {
                    completed_count += 1;
                }

                resource_completions.push(ResourceCompletionData {
                    resource_id: resource.id,
                    resource_name: resource.name.clone(),
                    completed,
                    completed_at,
                    time_spent_seconds,
                });
            }

            let completion_percentage =
                ProgressSummary::calculate_percentage(completed_count, all_resources.len() as i32);

            // Apply completion status filter
            let include_student = match filter.completion_status {
                CompletionStatus::All => true,
                CompletionStatus::Completed => completion_percentage == 100,
                CompletionStatus::Incomplete => completion_percentage < 100,
            };

            if include_student {
                total_percentage_sum += completion_percentage as f32;
                student_data_list.push(StudentProgressData {
                    student_id,
                    completion_percentage,
                    resources: resource_completions,
                });
            }
        }

        let total_students = student_data_list.len() as i32;
        let average_completion_percentage = if total_students > 0 {
            total_percentage_sum / total_students as f32
        } else {
            0.0
        };

        Ok(ProgressReportData {
            total_students,
            average_completion_percentage,
            students: student_data_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_status_from_str() {
        assert_eq!(
            CompletionStatus::from_str("completed"),
            CompletionStatus::Completed
        );
        assert_eq!(
            CompletionStatus::from_str("COMPLETED"),
            CompletionStatus::Completed
        );
        assert_eq!(
            CompletionStatus::from_str("incomplete"),
            CompletionStatus::Incomplete
        );
        assert_eq!(
            CompletionStatus::from_str("INCOMPLETE"),
            CompletionStatus::Incomplete
        );
        assert_eq!(CompletionStatus::from_str("all"), CompletionStatus::All);
        assert_eq!(CompletionStatus::from_str(""), CompletionStatus::All);
        assert_eq!(CompletionStatus::from_str("invalid"), CompletionStatus::All);
    }
}
