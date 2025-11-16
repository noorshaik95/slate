package service

import (
	"context"
	"time"

	"slate/services/assignment-grading-service/internal/models"
	"slate/services/assignment-grading-service/internal/repository"
)

// AssignmentService defines the interface for assignment business logic
type AssignmentService interface {
	CreateAssignment(ctx context.Context, courseID, title, description string, maxPoints float64, dueDate time.Time, latePolicy models.LatePolicy) (*models.Assignment, error)
	GetAssignment(ctx context.Context, id string) (*models.Assignment, error)
	UpdateAssignment(ctx context.Context, id, title, description string, maxPoints float64, dueDate time.Time, latePolicy models.LatePolicy) (*models.Assignment, error)
	DeleteAssignment(ctx context.Context, id string) error
	ListAssignments(ctx context.Context, courseID string, page, pageSize int) ([]*models.Assignment, int, error)
}

// SubmissionService defines the interface for submission business logic
type SubmissionService interface {
	SubmitAssignment(ctx context.Context, assignmentID, studentID string, fileContent []byte, fileName, contentType string) (*models.Submission, error)
	GetSubmission(ctx context.Context, id string) (*models.Submission, error)
	ListSubmissions(ctx context.Context, assignmentID, sortBy, order string) ([]*models.Submission, error)
	ListStudentSubmissions(ctx context.Context, studentID, courseID string) ([]*models.Submission, error)
}

// GradingService defines the interface for grading business logic
type GradingService interface {
	CreateGrade(ctx context.Context, submissionID string, score float64, feedback, gradedBy string) (*models.Grade, error)
	UpdateGrade(ctx context.Context, id string, score float64, feedback string) (*models.Grade, error)
	PublishGrade(ctx context.Context, id string) (*models.Grade, error)
	GetGrade(ctx context.Context, id string) (*models.Grade, error)
}

// GradebookService defines the interface for gradebook business logic
type GradebookService interface {
	GetStudentGradebook(ctx context.Context, studentID, courseID string) (*StudentGradebook, error)
	GetCourseGradebook(ctx context.Context, courseID string) (*CourseGradebook, error)
	GetGradeStatistics(ctx context.Context, assignmentID string) (*repository.GradeStatistics, error)
	ExportGrades(ctx context.Context, courseID, format string) ([]byte, error)
}

// StudentGradebook represents a student's gradebook
type StudentGradebook struct {
	StudentID    string
	CourseID     string
	Entries      []GradebookEntry
	TotalPoints  float64
	EarnedPoints float64
	Percentage   float64
	LetterGrade  string
}

// GradebookEntry represents a single gradebook entry
type GradebookEntry struct {
	AssignmentID    string
	AssignmentTitle string
	MaxPoints       float64
	Score           float64
	AdjustedScore   float64
	Status          string
	DueDate         time.Time
	SubmittedAt     *time.Time
	IsLate          bool
}

// CourseGradebook represents the gradebook for an entire course
type CourseGradebook struct {
	CourseID string
	Students []StudentSummary
}

// StudentSummary represents a student's grade summary
type StudentSummary struct {
	StudentID    string
	TotalPoints  float64
	EarnedPoints float64
	Percentage   float64
	LetterGrade  string
	Entries      []GradebookEntry
}
