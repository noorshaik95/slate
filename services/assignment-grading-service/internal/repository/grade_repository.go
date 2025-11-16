package repository

import (
	"context"
	"database/sql"
	"fmt"
	"math"
	"time"

	"slate/services/assignment-grading-service/internal/models"

	"github.com/google/uuid"
)

// GradeRepository defines the interface for grade data access
type GradeRepository interface {
	Create(ctx context.Context, grade *models.Grade) error
	GetByID(ctx context.Context, id string) (*models.Grade, error)
	GetBySubmission(ctx context.Context, submissionID string) (*models.Grade, error)
	Update(ctx context.Context, grade *models.Grade) error
	ListByStudent(ctx context.Context, studentID, courseID string) ([]*models.Grade, error)
	ListByCourse(ctx context.Context, courseID string) ([]*models.Grade, error)
	GetStatistics(ctx context.Context, assignmentID string) (*GradeStatistics, error)
}

type GradeStatistics struct {
	TotalSubmissions int
	GradedCount      int
	Mean             float64
	Median           float64
	StdDeviation     float64
	MinScore         float64
	MaxScore         float64
}

type gradeRepository struct {
	db *sql.DB
}

// NewGradeRepository creates a new grade repository
func NewGradeRepository(db *sql.DB) GradeRepository {
	return &gradeRepository{db: db}
}

// Create creates a new grade
func (r *gradeRepository) Create(ctx context.Context, grade *models.Grade) error {
	grade.ID = uuid.New().String()
	now := time.Now()
	grade.CreatedAt = now
	grade.UpdatedAt = now
	grade.GradedAt = &now

	query := `
		INSERT INTO grades (
			id, submission_id, student_id, assignment_id, score, adjusted_score,
			feedback, status, graded_at, graded_by, created_at, updated_at
		) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
	`

	_, err := r.db.ExecContext(ctx, query,
		grade.ID, grade.SubmissionID, grade.StudentID, grade.AssignmentID,
		grade.Score, grade.AdjustedScore, grade.Feedback, grade.Status,
		grade.GradedAt, grade.GradedBy, grade.CreatedAt, grade.UpdatedAt,
	)

	if err != nil {
		return fmt.Errorf("failed to create grade: %w", err)
	}

	return nil
}

// GetByID retrieves a grade by ID
func (r *gradeRepository) GetByID(ctx context.Context, id string) (*models.Grade, error) {
	query := `
		SELECT id, submission_id, student_id, assignment_id, score, adjusted_score,
			   feedback, status, graded_at, published_at, graded_by, created_at, updated_at
		FROM grades
		WHERE id = $1
	`

	grade := &models.Grade{}
	err := r.db.QueryRowContext(ctx, query, id).Scan(
		&grade.ID, &grade.SubmissionID, &grade.StudentID, &grade.AssignmentID,
		&grade.Score, &grade.AdjustedScore, &grade.Feedback, &grade.Status,
		&grade.GradedAt, &grade.PublishedAt, &grade.GradedBy,
		&grade.CreatedAt, &grade.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("grade not found")
	}

	if err != nil {
		return nil, fmt.Errorf("failed to get grade: %w", err)
	}

	return grade, nil
}

// GetBySubmission retrieves a grade by submission ID
func (r *gradeRepository) GetBySubmission(ctx context.Context, submissionID string) (*models.Grade, error) {
	query := `
		SELECT id, submission_id, student_id, assignment_id, score, adjusted_score,
			   feedback, status, graded_at, published_at, graded_by, created_at, updated_at
		FROM grades
		WHERE submission_id = $1
	`

	grade := &models.Grade{}
	err := r.db.QueryRowContext(ctx, query, submissionID).Scan(
		&grade.ID, &grade.SubmissionID, &grade.StudentID, &grade.AssignmentID,
		&grade.Score, &grade.AdjustedScore, &grade.Feedback, &grade.Status,
		&grade.GradedAt, &grade.PublishedAt, &grade.GradedBy,
		&grade.CreatedAt, &grade.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, nil // Not found is not an error
	}

	if err != nil {
		return nil, fmt.Errorf("failed to get grade: %w", err)
	}

	return grade, nil
}

// Update updates an existing grade
func (r *gradeRepository) Update(ctx context.Context, grade *models.Grade) error {
	grade.UpdatedAt = time.Now()

	query := `
		UPDATE grades
		SET score = $2, adjusted_score = $3, feedback = $4, status = $5,
			published_at = $6, updated_at = $7
		WHERE id = $1
	`

	result, err := r.db.ExecContext(ctx, query,
		grade.ID, grade.Score, grade.AdjustedScore, grade.Feedback,
		grade.Status, grade.PublishedAt, grade.UpdatedAt,
	)

	if err != nil {
		return fmt.Errorf("failed to update grade: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("grade not found")
	}

	return nil
}

// ListByStudent lists grades for a student in a course
func (r *gradeRepository) ListByStudent(ctx context.Context, studentID, courseID string) ([]*models.Grade, error) {
	query := `
		SELECT g.id, g.submission_id, g.student_id, g.assignment_id, g.score, g.adjusted_score,
			   g.feedback, g.status, g.graded_at, g.published_at, g.graded_by, g.created_at, g.updated_at
		FROM grades g
		JOIN assignments a ON g.assignment_id = a.id
		WHERE g.student_id = $1 AND a.course_id = $2 AND g.status = 'published'
		ORDER BY a.due_date DESC
	`

	rows, err := r.db.QueryContext(ctx, query, studentID, courseID)
	if err != nil {
		return nil, fmt.Errorf("failed to list grades: %w", err)
	}
	defer rows.Close()

	var grades []*models.Grade
	for rows.Next() {
		grade := &models.Grade{}
		err := rows.Scan(
			&grade.ID, &grade.SubmissionID, &grade.StudentID, &grade.AssignmentID,
			&grade.Score, &grade.AdjustedScore, &grade.Feedback, &grade.Status,
			&grade.GradedAt, &grade.PublishedAt, &grade.GradedBy,
			&grade.CreatedAt, &grade.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan grade: %w", err)
		}
		grades = append(grades, grade)
	}

	return grades, nil
}

// ListByCourse lists all grades for a course
func (r *gradeRepository) ListByCourse(ctx context.Context, courseID string) ([]*models.Grade, error) {
	query := `
		SELECT g.id, g.submission_id, g.student_id, g.assignment_id, g.score, g.adjusted_score,
			   g.feedback, g.status, g.graded_at, g.published_at, g.graded_by, g.created_at, g.updated_at
		FROM grades g
		JOIN assignments a ON g.assignment_id = a.id
		WHERE a.course_id = $1 AND g.status = 'published'
		ORDER BY g.student_id, a.due_date
	`

	rows, err := r.db.QueryContext(ctx, query, courseID)
	if err != nil {
		return nil, fmt.Errorf("failed to list grades: %w", err)
	}
	defer rows.Close()

	var grades []*models.Grade
	for rows.Next() {
		grade := &models.Grade{}
		err := rows.Scan(
			&grade.ID, &grade.SubmissionID, &grade.StudentID, &grade.AssignmentID,
			&grade.Score, &grade.AdjustedScore, &grade.Feedback, &grade.Status,
			&grade.GradedAt, &grade.PublishedAt, &grade.GradedBy,
			&grade.CreatedAt, &grade.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan grade: %w", err)
		}
		grades = append(grades, grade)
	}

	return grades, nil
}

// GetStatistics calculates statistics for an assignment
func (r *gradeRepository) GetStatistics(ctx context.Context, assignmentID string) (*GradeStatistics, error) {
	// Get total submissions count
	var totalSubmissions int
	err := r.db.QueryRowContext(ctx,
		`SELECT COUNT(*) FROM submissions WHERE assignment_id = $1`,
		assignmentID,
	).Scan(&totalSubmissions)
	if err != nil {
		return nil, fmt.Errorf("failed to count submissions: %w", err)
	}

	// Get graded count and basic stats
	query := `
		SELECT COUNT(*), COALESCE(MIN(adjusted_score), 0), COALESCE(MAX(adjusted_score), 0), COALESCE(AVG(adjusted_score), 0)
		FROM grades
		WHERE assignment_id = $1 AND status = 'published'
	`

	var gradedCount int
	var minScore, maxScore, mean float64
	err = r.db.QueryRowContext(ctx, query, assignmentID).Scan(&gradedCount, &minScore, &maxScore, &mean)
	if err != nil {
		return nil, fmt.Errorf("failed to get basic stats: %w", err)
	}

	// Get all scores for median and std deviation calculation
	rows, err := r.db.QueryContext(ctx,
		`SELECT adjusted_score FROM grades WHERE assignment_id = $1 AND status = 'published' ORDER BY adjusted_score`,
		assignmentID,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to get scores: %w", err)
	}
	defer rows.Close()

	var scores []float64
	for rows.Next() {
		var score float64
		if err := rows.Scan(&score); err != nil {
			return nil, fmt.Errorf("failed to scan score: %w", err)
		}
		scores = append(scores, score)
	}

	// Calculate median
	median := 0.0
	if len(scores) > 0 {
		mid := len(scores) / 2
		if len(scores)%2 == 0 {
			median = (scores[mid-1] + scores[mid]) / 2
		} else {
			median = scores[mid]
		}
	}

	// Calculate standard deviation
	stdDev := 0.0
	if len(scores) > 0 {
		variance := 0.0
		for _, score := range scores {
			variance += math.Pow(score-mean, 2)
		}
		variance /= float64(len(scores))
		stdDev = math.Sqrt(variance)
	}

	return &GradeStatistics{
		TotalSubmissions: totalSubmissions,
		GradedCount:      gradedCount,
		Mean:             mean,
		Median:           median,
		StdDeviation:     stdDev,
		MinScore:         minScore,
		MaxScore:         maxScore,
	}, nil
}
