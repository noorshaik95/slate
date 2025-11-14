package repository

import (
	"context"
	"database/sql"
	"regexp"
	"testing"
	"time"

	"slate/services/assignment-grading-service/internal/models"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
)

func TestGradeRepository_Create(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewGradeRepository(db)
	ctx := context.Background()

	grade := &models.Grade{
		SubmissionID:  "SUB-001",
		StudentID:     "STUDENT-001",
		AssignmentID:  "ASSIGN-001",
		Score:         85.0,
		AdjustedScore: 85.0,
		Feedback:      "Good work",
		Status:        models.GradeStatusDraft,
		GradedBy:      "INSTRUCTOR-001",
	}

	mock.ExpectExec(regexp.QuoteMeta(`INSERT INTO grades`)).
		WithArgs(
			sqlmock.AnyArg(), // id
			grade.SubmissionID,
			grade.StudentID,
			grade.AssignmentID,
			grade.Score,
			grade.AdjustedScore,
			grade.Feedback,
			grade.Status,
			sqlmock.AnyArg(), // graded_at
			grade.GradedBy,
			sqlmock.AnyArg(), // created_at
			sqlmock.AnyArg(), // updated_at
		).
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.Create(ctx, grade)

	assert.NoError(t, err)
	assert.NotEmpty(t, grade.ID)
	assert.NotNil(t, grade.GradedAt)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGradeRepository_GetByID(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewGradeRepository(db)
	ctx := context.Background()

	id := "test-id"
	now := time.Now()

	rows := sqlmock.NewRows([]string{
		"id", "submission_id", "student_id", "assignment_id", "score", "adjusted_score",
		"feedback", "status", "graded_at", "published_at", "graded_by", "created_at", "updated_at",
	}).AddRow(
		id, "SUB-001", "STUDENT-001", "ASSIGN-001", 85.0, 85.0,
		"Good", models.GradeStatusDraft, &now, nil, "INSTRUCTOR-001", now, now,
	)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, submission_id, student_id, assignment_id, score, adjusted_score`)).
		WithArgs(id).
		WillReturnRows(rows)

	grade, err := repo.GetByID(ctx, id)

	assert.NoError(t, err)
	assert.NotNil(t, grade)
	assert.Equal(t, id, grade.ID)
	assert.Equal(t, 85.0, grade.Score)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGradeRepository_GetBySubmission(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewGradeRepository(db)
	ctx := context.Background()

	t.Run("grade exists", func(t *testing.T) {
		now := time.Now()
		rows := sqlmock.NewRows([]string{
			"id", "submission_id", "student_id", "assignment_id", "score", "adjusted_score",
			"feedback", "status", "graded_at", "published_at", "graded_by", "created_at", "updated_at",
		}).AddRow(
			"grade-id", "SUB-001", "STUDENT-001", "ASSIGN-001", 90.0, 90.0,
			"Excellent", models.GradeStatusPublished, &now, &now, "INSTRUCTOR-001", now, now,
		)

		mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, submission_id, student_id, assignment_id, score, adjusted_score`)).
			WithArgs("SUB-001").
			WillReturnRows(rows)

		grade, err := repo.GetBySubmission(ctx, "SUB-001")

		assert.NoError(t, err)
		assert.NotNil(t, grade)
		assert.Equal(t, "grade-id", grade.ID)
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("grade not found", func(t *testing.T) {
		mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, submission_id, student_id, assignment_id, score, adjusted_score`)).
			WithArgs("SUB-002").
			WillReturnError(sql.ErrNoRows)

		grade, err := repo.GetBySubmission(ctx, "SUB-002")

		assert.NoError(t, err)
		assert.Nil(t, grade)
		assert.NoError(t, mock.ExpectationsWereMet())
	})
}

func TestGradeRepository_Update(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewGradeRepository(db)
	ctx := context.Background()

	now := time.Now()
	grade := &models.Grade{
		ID:            "test-id",
		Score:         95.0,
		AdjustedScore: 95.0,
		Feedback:      "Updated feedback",
		Status:        models.GradeStatusPublished,
		PublishedAt:   &now,
	}

	mock.ExpectExec(regexp.QuoteMeta(`UPDATE grades`)).
		WithArgs(
			grade.ID,
			grade.Score,
			grade.AdjustedScore,
			grade.Feedback,
			grade.Status,
			grade.PublishedAt,
			sqlmock.AnyArg(), // updated_at
		).
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.Update(ctx, grade)

	assert.NoError(t, err)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGradeRepository_ListByStudent(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewGradeRepository(db)
	ctx := context.Background()

	now := time.Now()
	rows := sqlmock.NewRows([]string{
		"id", "submission_id", "student_id", "assignment_id", "score", "adjusted_score",
		"feedback", "status", "graded_at", "published_at", "graded_by", "created_at", "updated_at",
	}).
		AddRow("id1", "SUB-001", "STUDENT-001", "ASSIGN-001", 85.0, 85.0, "Good", models.GradeStatusPublished, &now, &now, "INST-001", now, now).
		AddRow("id2", "SUB-002", "STUDENT-001", "ASSIGN-002", 90.0, 90.0, "Great", models.GradeStatusPublished, &now, &now, "INST-001", now, now)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT g.id, g.submission_id, g.student_id, g.assignment_id, g.score, g.adjusted_score`)).
		WithArgs("STUDENT-001", "COURSE-001").
		WillReturnRows(rows)

	grades, err := repo.ListByStudent(ctx, "STUDENT-001", "COURSE-001")

	assert.NoError(t, err)
	assert.Len(t, grades, 2)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGradeRepository_GetStatistics(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewGradeRepository(db)
	ctx := context.Background()

	// Mock total submissions count
	countRows := sqlmock.NewRows([]string{"count"}).AddRow(10)
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT COUNT(*) FROM submissions WHERE assignment_id = $1`)).
		WithArgs("ASSIGN-001").
		WillReturnRows(countRows)

	// Mock basic stats
	statsRows := sqlmock.NewRows([]string{"count", "min", "max", "avg"}).
		AddRow(8, 70.0, 95.0, 82.5)
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT COUNT(*), COALESCE(MIN(adjusted_score), 0), COALESCE(MAX(adjusted_score), 0), COALESCE(AVG(adjusted_score), 0)`)).
		WithArgs("ASSIGN-001").
		WillReturnRows(statsRows)

	// Mock scores for median and std dev
	scoresRows := sqlmock.NewRows([]string{"adjusted_score"}).
		AddRow(70.0).
		AddRow(75.0).
		AddRow(80.0).
		AddRow(82.0).
		AddRow(85.0).
		AddRow(87.0).
		AddRow(90.0).
		AddRow(95.0)
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT adjusted_score FROM grades WHERE assignment_id = $1 AND status = 'published' ORDER BY adjusted_score`)).
		WithArgs("ASSIGN-001").
		WillReturnRows(scoresRows)

	stats, err := repo.GetStatistics(ctx, "ASSIGN-001")

	assert.NoError(t, err)
	assert.NotNil(t, stats)
	assert.Equal(t, 10, stats.TotalSubmissions)
	assert.Equal(t, 8, stats.GradedCount)
	assert.Equal(t, 82.5, stats.Mean)
	assert.Equal(t, 70.0, stats.MinScore)
	assert.Equal(t, 95.0, stats.MaxScore)
	assert.Greater(t, stats.Median, 0.0)
	assert.Greater(t, stats.StdDeviation, 0.0)
	assert.NoError(t, mock.ExpectationsWereMet())
}
