package repository

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	"slate/services/assignment-grading-service/internal/models"

	"github.com/google/uuid"
)

// SubmissionRepository defines the interface for submission data access
type SubmissionRepository interface {
	Create(ctx context.Context, submission *models.Submission) error
	GetByID(ctx context.Context, id string) (*models.Submission, error)
	GetByAssignmentAndStudent(ctx context.Context, assignmentID, studentID string) (*models.Submission, error)
	Update(ctx context.Context, submission *models.Submission) error
	ListByAssignment(ctx context.Context, assignmentID, sortBy, order string) ([]*models.Submission, error)
	ListByStudent(ctx context.Context, studentID, courseID string) ([]*models.Submission, error)
}

type submissionRepository struct {
	db *sql.DB
}

// NewSubmissionRepository creates a new submission repository
func NewSubmissionRepository(db *sql.DB) SubmissionRepository {
	return &submissionRepository{db: db}
}

// Create creates a new submission (or updates if exists due to UNIQUE constraint)
func (r *submissionRepository) Create(ctx context.Context, submission *models.Submission) error {
	if submission.ID == "" {
		submission.ID = uuid.New().String()
	}
	submission.CreatedAt = time.Now()
	submission.UpdatedAt = time.Now()

	query := `
		INSERT INTO submissions (
			id, assignment_id, student_id, file_path, submitted_at,
			status, is_late, days_late, created_at, updated_at
		) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
		ON CONFLICT (assignment_id, student_id)
		DO UPDATE SET
			file_path = EXCLUDED.file_path,
			submitted_at = EXCLUDED.submitted_at,
			status = EXCLUDED.status,
			is_late = EXCLUDED.is_late,
			days_late = EXCLUDED.days_late,
			updated_at = EXCLUDED.updated_at
		RETURNING id
	`

	err := r.db.QueryRowContext(ctx, query,
		submission.ID, submission.AssignmentID, submission.StudentID, submission.FilePath,
		submission.SubmittedAt, submission.Status, submission.IsLate, submission.DaysLate,
		submission.CreatedAt, submission.UpdatedAt,
	).Scan(&submission.ID)

	if err != nil {
		return fmt.Errorf("failed to create submission: %w", err)
	}

	return nil
}

// GetByID retrieves a submission by ID
func (r *submissionRepository) GetByID(ctx context.Context, id string) (*models.Submission, error) {
	query := `
		SELECT id, assignment_id, student_id, file_path, submitted_at,
			   status, is_late, days_late, created_at, updated_at
		FROM submissions
		WHERE id = $1
	`

	submission := &models.Submission{}
	err := r.db.QueryRowContext(ctx, query, id).Scan(
		&submission.ID, &submission.AssignmentID, &submission.StudentID, &submission.FilePath,
		&submission.SubmittedAt, &submission.Status, &submission.IsLate, &submission.DaysLate,
		&submission.CreatedAt, &submission.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("submission not found")
	}

	if err != nil {
		return nil, fmt.Errorf("failed to get submission: %w", err)
	}

	return submission, nil
}

// GetByAssignmentAndStudent retrieves a submission by assignment and student
func (r *submissionRepository) GetByAssignmentAndStudent(ctx context.Context, assignmentID, studentID string) (*models.Submission, error) {
	query := `
		SELECT id, assignment_id, student_id, file_path, submitted_at,
			   status, is_late, days_late, created_at, updated_at
		FROM submissions
		WHERE assignment_id = $1 AND student_id = $2
	`

	submission := &models.Submission{}
	err := r.db.QueryRowContext(ctx, query, assignmentID, studentID).Scan(
		&submission.ID, &submission.AssignmentID, &submission.StudentID, &submission.FilePath,
		&submission.SubmittedAt, &submission.Status, &submission.IsLate, &submission.DaysLate,
		&submission.CreatedAt, &submission.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, nil // Not found is not an error
	}

	if err != nil {
		return nil, fmt.Errorf("failed to get submission: %w", err)
	}

	return submission, nil
}

// Update updates an existing submission
func (r *submissionRepository) Update(ctx context.Context, submission *models.Submission) error {
	submission.UpdatedAt = time.Now()

	query := `
		UPDATE submissions
		SET file_path = $2, status = $3, is_late = $4, days_late = $5, updated_at = $6
		WHERE id = $1
	`

	result, err := r.db.ExecContext(ctx, query,
		submission.ID, submission.FilePath, submission.Status, submission.IsLate,
		submission.DaysLate, submission.UpdatedAt,
	)

	if err != nil {
		return fmt.Errorf("failed to update submission: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("submission not found")
	}

	return nil
}

// ListByAssignment lists submissions for an assignment with sorting
func (r *submissionRepository) ListByAssignment(ctx context.Context, assignmentID, sortBy, order string) ([]*models.Submission, error) {
	// Validate sort parameters
	if sortBy == "" {
		sortBy = "submitted_at"
	}
	if order == "" {
		order = "DESC"
	}

	// Prevent SQL injection by validating sortBy
	validSortFields := map[string]bool{
		"submitted_at": true,
		"student_id":   true,
		"status":       true,
	}
	if !validSortFields[sortBy] {
		sortBy = "submitted_at"
	}

	// Validate order
	if order != "ASC" && order != "DESC" {
		order = "DESC"
	}

	// #nosec G201 - sortBy and order are validated against allowlists above
	query := fmt.Sprintf(`
		SELECT id, assignment_id, student_id, file_path, submitted_at,
			   status, is_late, days_late, created_at, updated_at
		FROM submissions
		WHERE assignment_id = $1
		ORDER BY %s %s
	`, sortBy, order)

	rows, err := r.db.QueryContext(ctx, query, assignmentID)
	if err != nil {
		return nil, fmt.Errorf("failed to list submissions: %w", err)
	}
	defer rows.Close()

	var submissions []*models.Submission
	for rows.Next() {
		submission := &models.Submission{}
		err := rows.Scan(
			&submission.ID, &submission.AssignmentID, &submission.StudentID, &submission.FilePath,
			&submission.SubmittedAt, &submission.Status, &submission.IsLate, &submission.DaysLate,
			&submission.CreatedAt, &submission.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan submission: %w", err)
		}
		submissions = append(submissions, submission)
	}

	return submissions, nil
}

// ListByStudent lists submissions for a student in a course
func (r *submissionRepository) ListByStudent(ctx context.Context, studentID, courseID string) ([]*models.Submission, error) {
	query := `
		SELECT s.id, s.assignment_id, s.student_id, s.file_path, s.submitted_at,
			   s.status, s.is_late, s.days_late, s.created_at, s.updated_at
		FROM submissions s
		JOIN assignments a ON s.assignment_id = a.id
		WHERE s.student_id = $1 AND a.course_id = $2
		ORDER BY s.submitted_at DESC
	`

	rows, err := r.db.QueryContext(ctx, query, studentID, courseID)
	if err != nil {
		return nil, fmt.Errorf("failed to list student submissions: %w", err)
	}
	defer rows.Close()

	var submissions []*models.Submission
	for rows.Next() {
		submission := &models.Submission{}
		err := rows.Scan(
			&submission.ID, &submission.AssignmentID, &submission.StudentID, &submission.FilePath,
			&submission.SubmittedAt, &submission.Status, &submission.IsLate, &submission.DaysLate,
			&submission.CreatedAt, &submission.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan submission: %w", err)
		}
		submissions = append(submissions, submission)
	}

	return submissions, nil
}
