package repository

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	"slate/services/assignment-grading-service/internal/models"

	"github.com/google/uuid"
)

// AssignmentRepository defines the interface for assignment data access
type AssignmentRepository interface {
	Create(ctx context.Context, assignment *models.Assignment) error
	GetByID(ctx context.Context, id string) (*models.Assignment, error)
	Update(ctx context.Context, assignment *models.Assignment) error
	Delete(ctx context.Context, id string) error
	ListByCourse(ctx context.Context, courseID string, page, pageSize int) ([]*models.Assignment, int, error)
	HasSubmissions(ctx context.Context, id string) (bool, error)
}

type assignmentRepository struct {
	db *sql.DB
}

// NewAssignmentRepository creates a new assignment repository
func NewAssignmentRepository(db *sql.DB) AssignmentRepository {
	return &assignmentRepository{db: db}
}

// Create creates a new assignment
func (r *assignmentRepository) Create(ctx context.Context, assignment *models.Assignment) error {
	assignment.ID = uuid.New().String()
	assignment.CreatedAt = time.Now()
	assignment.UpdatedAt = time.Now()

	query := `
		INSERT INTO assignments (
			id, course_id, title, description, max_points, due_date,
			late_penalty_percent, max_late_days, created_at, updated_at
		) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
	`

	_, err := r.db.ExecContext(ctx, query,
		assignment.ID, assignment.CourseID, assignment.Title, assignment.Description,
		assignment.MaxPoints, assignment.DueDate, assignment.LatePolicy.PenaltyPercentPerDay,
		assignment.LatePolicy.MaxLateDays, assignment.CreatedAt, assignment.UpdatedAt,
	)

	if err != nil {
		return fmt.Errorf("failed to create assignment: %w", err)
	}

	return nil
}

// GetByID retrieves an assignment by ID
func (r *assignmentRepository) GetByID(ctx context.Context, id string) (*models.Assignment, error) {
	query := `
		SELECT id, course_id, title, description, max_points, due_date,
			   late_penalty_percent, max_late_days, created_at, updated_at
		FROM assignments
		WHERE id = $1
	`

	assignment := &models.Assignment{}
	err := r.db.QueryRowContext(ctx, query, id).Scan(
		&assignment.ID, &assignment.CourseID, &assignment.Title, &assignment.Description,
		&assignment.MaxPoints, &assignment.DueDate, &assignment.LatePolicy.PenaltyPercentPerDay,
		&assignment.LatePolicy.MaxLateDays, &assignment.CreatedAt, &assignment.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("assignment not found")
	}

	if err != nil {
		return nil, fmt.Errorf("failed to get assignment: %w", err)
	}

	return assignment, nil
}

// Update updates an existing assignment
func (r *assignmentRepository) Update(ctx context.Context, assignment *models.Assignment) error {
	assignment.UpdatedAt = time.Now()

	query := `
		UPDATE assignments
		SET title = $2, description = $3, max_points = $4, due_date = $5,
			late_penalty_percent = $6, max_late_days = $7, updated_at = $8
		WHERE id = $1
	`

	result, err := r.db.ExecContext(ctx, query,
		assignment.ID, assignment.Title, assignment.Description, assignment.MaxPoints,
		assignment.DueDate, assignment.LatePolicy.PenaltyPercentPerDay,
		assignment.LatePolicy.MaxLateDays, assignment.UpdatedAt,
	)

	if err != nil {
		return fmt.Errorf("failed to update assignment: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("assignment not found")
	}

	return nil
}

// Delete deletes an assignment
func (r *assignmentRepository) Delete(ctx context.Context, id string) error {
	query := `DELETE FROM assignments WHERE id = $1`

	result, err := r.db.ExecContext(ctx, query, id)
	if err != nil {
		return fmt.Errorf("failed to delete assignment: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("assignment not found")
	}

	return nil
}

// ListByCourse lists assignments for a course with pagination
func (r *assignmentRepository) ListByCourse(ctx context.Context, courseID string, page, pageSize int) ([]*models.Assignment, int, error) {
	// Get total count
	var total int
	countQuery := `SELECT COUNT(*) FROM assignments WHERE course_id = $1`
	if err := r.db.QueryRowContext(ctx, countQuery, courseID).Scan(&total); err != nil {
		return nil, 0, fmt.Errorf("failed to count assignments: %w", err)
	}

	// Get paginated results
	offset := (page - 1) * pageSize
	query := `
		SELECT id, course_id, title, description, max_points, due_date,
			   late_penalty_percent, max_late_days, created_at, updated_at
		FROM assignments
		WHERE course_id = $1
		ORDER BY due_date DESC
		LIMIT $2 OFFSET $3
	`

	rows, err := r.db.QueryContext(ctx, query, courseID, pageSize, offset)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to list assignments: %w", err)
	}
	defer rows.Close()

	var assignments []*models.Assignment
	for rows.Next() {
		assignment := &models.Assignment{}
		err := rows.Scan(
			&assignment.ID, &assignment.CourseID, &assignment.Title, &assignment.Description,
			&assignment.MaxPoints, &assignment.DueDate, &assignment.LatePolicy.PenaltyPercentPerDay,
			&assignment.LatePolicy.MaxLateDays, &assignment.CreatedAt, &assignment.UpdatedAt,
		)
		if err != nil {
			return nil, 0, fmt.Errorf("failed to scan assignment: %w", err)
		}
		assignments = append(assignments, assignment)
	}

	return assignments, total, nil
}

// HasSubmissions checks if an assignment has any submissions
func (r *assignmentRepository) HasSubmissions(ctx context.Context, id string) (bool, error) {
	query := `SELECT EXISTS(SELECT 1 FROM submissions WHERE assignment_id = $1)`

	var exists bool
	err := r.db.QueryRowContext(ctx, query, id).Scan(&exists)
	if err != nil {
		return false, fmt.Errorf("failed to check submissions: %w", err)
	}

	return exists, nil
}
