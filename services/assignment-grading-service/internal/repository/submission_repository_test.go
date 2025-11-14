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

func TestSubmissionRepository_Create(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewSubmissionRepository(db)
	ctx := context.Background()

	submission := &models.Submission{
		AssignmentID: "ASSIGN-001",
		StudentID:    "STUDENT-001",
		FilePath:     "/path/to/file.pdf",
		SubmittedAt:  time.Now(),
		Status:       models.StatusSubmitted,
		IsLate:       false,
		DaysLate:     0,
	}

	rows := sqlmock.NewRows([]string{"id"}).AddRow("new-id")

	mock.ExpectQuery(regexp.QuoteMeta(`INSERT INTO submissions`)).
		WithArgs(
			sqlmock.AnyArg(), // id
			submission.AssignmentID,
			submission.StudentID,
			submission.FilePath,
			submission.SubmittedAt,
			submission.Status,
			submission.IsLate,
			submission.DaysLate,
			sqlmock.AnyArg(), // created_at
			sqlmock.AnyArg(), // updated_at
		).
		WillReturnRows(rows)

	err = repo.Create(ctx, submission)

	assert.NoError(t, err)
	assert.Equal(t, "new-id", submission.ID)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSubmissionRepository_GetByID(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewSubmissionRepository(db)
	ctx := context.Background()

	id := "test-id"
	now := time.Now()

	rows := sqlmock.NewRows([]string{
		"id", "assignment_id", "student_id", "file_path", "submitted_at",
		"status", "is_late", "days_late", "created_at", "updated_at",
	}).AddRow(
		id, "ASSIGN-001", "STUDENT-001", "/path/file.pdf", now,
		models.StatusSubmitted, false, 0, now, now,
	)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, assignment_id, student_id, file_path, submitted_at`)).
		WithArgs(id).
		WillReturnRows(rows)

	submission, err := repo.GetByID(ctx, id)

	assert.NoError(t, err)
	assert.NotNil(t, submission)
	assert.Equal(t, id, submission.ID)
	assert.Equal(t, "ASSIGN-001", submission.AssignmentID)
	assert.Equal(t, "STUDENT-001", submission.StudentID)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSubmissionRepository_GetByAssignmentAndStudent(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewSubmissionRepository(db)
	ctx := context.Background()

	t.Run("submission exists", func(t *testing.T) {
		now := time.Now()
		rows := sqlmock.NewRows([]string{
			"id", "assignment_id", "student_id", "file_path", "submitted_at",
			"status", "is_late", "days_late", "created_at", "updated_at",
		}).AddRow(
			"sub-id", "ASSIGN-001", "STUDENT-001", "/path/file.pdf", now,
			models.StatusSubmitted, false, 0, now, now,
		)

		mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, assignment_id, student_id, file_path, submitted_at`)).
			WithArgs("ASSIGN-001", "STUDENT-001").
			WillReturnRows(rows)

		submission, err := repo.GetByAssignmentAndStudent(ctx, "ASSIGN-001", "STUDENT-001")

		assert.NoError(t, err)
		assert.NotNil(t, submission)
		assert.Equal(t, "sub-id", submission.ID)
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("submission not found", func(t *testing.T) {
		mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, assignment_id, student_id, file_path, submitted_at`)).
			WithArgs("ASSIGN-001", "STUDENT-002").
			WillReturnError(sql.ErrNoRows)

		submission, err := repo.GetByAssignmentAndStudent(ctx, "ASSIGN-001", "STUDENT-002")

		assert.NoError(t, err)
		assert.Nil(t, submission)
		assert.NoError(t, mock.ExpectationsWereMet())
	})
}

func TestSubmissionRepository_Update(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewSubmissionRepository(db)
	ctx := context.Background()

	submission := &models.Submission{
		ID:       "test-id",
		FilePath: "/new/path.pdf",
		Status:   models.StatusGraded,
		IsLate:   true,
		DaysLate: 2,
	}

	mock.ExpectExec(regexp.QuoteMeta(`UPDATE submissions`)).
		WithArgs(
			submission.ID,
			submission.FilePath,
			submission.Status,
			submission.IsLate,
			submission.DaysLate,
			sqlmock.AnyArg(), // updated_at
		).
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.Update(ctx, submission)

	assert.NoError(t, err)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSubmissionRepository_ListByAssignment(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewSubmissionRepository(db)
	ctx := context.Background()

	now := time.Now()
	rows := sqlmock.NewRows([]string{
		"id", "assignment_id", "student_id", "file_path", "submitted_at",
		"status", "is_late", "days_late", "created_at", "updated_at",
	}).
		AddRow("id1", "ASSIGN-001", "STUDENT-001", "/path1.pdf", now, models.StatusSubmitted, false, 0, now, now).
		AddRow("id2", "ASSIGN-001", "STUDENT-002", "/path2.pdf", now, models.StatusSubmitted, true, 1, now, now)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, assignment_id, student_id, file_path, submitted_at`)).
		WithArgs("ASSIGN-001").
		WillReturnRows(rows)

	submissions, err := repo.ListByAssignment(ctx, "ASSIGN-001", "submitted_at", "DESC")

	assert.NoError(t, err)
	assert.Len(t, submissions, 2)
	assert.Equal(t, "STUDENT-001", submissions[0].StudentID)
	assert.Equal(t, "STUDENT-002", submissions[1].StudentID)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSubmissionRepository_ListByStudent(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewSubmissionRepository(db)
	ctx := context.Background()

	now := time.Now()
	rows := sqlmock.NewRows([]string{
		"id", "assignment_id", "student_id", "file_path", "submitted_at",
		"status", "is_late", "days_late", "created_at", "updated_at",
	}).
		AddRow("id1", "ASSIGN-001", "STUDENT-001", "/path1.pdf", now, models.StatusSubmitted, false, 0, now, now).
		AddRow("id2", "ASSIGN-002", "STUDENT-001", "/path2.pdf", now, models.StatusGraded, false, 0, now, now)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT s.id, s.assignment_id, s.student_id, s.file_path, s.submitted_at`)).
		WithArgs("STUDENT-001", "COURSE-001").
		WillReturnRows(rows)

	submissions, err := repo.ListByStudent(ctx, "STUDENT-001", "COURSE-001")

	assert.NoError(t, err)
	assert.Len(t, submissions, 2)
	assert.NoError(t, mock.ExpectationsWereMet())
}
