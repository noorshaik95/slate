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

func TestAssignmentRepository_Create(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	dueDate := time.Now().Add(24 * time.Hour)
	assignment := &models.Assignment{
		CourseID:    "COURSE-001",
		Title:       "Assignment 1",
		Description: "Test assignment",
		MaxPoints:   100.0,
		DueDate:     dueDate,
		LatePolicy: models.LatePolicy{
			PenaltyPercentPerDay: 10,
			MaxLateDays:          3,
		},
	}

	mock.ExpectExec(regexp.QuoteMeta(`INSERT INTO assignments`)).
		WithArgs(
			sqlmock.AnyArg(), // id (UUID)
			assignment.CourseID,
			assignment.Title,
			assignment.Description,
			assignment.MaxPoints,
			assignment.DueDate,
			assignment.LatePolicy.PenaltyPercentPerDay,
			assignment.LatePolicy.MaxLateDays,
			sqlmock.AnyArg(), // created_at
			sqlmock.AnyArg(), // updated_at
		).
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.Create(ctx, assignment)

	assert.NoError(t, err)
	assert.NotEmpty(t, assignment.ID)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestAssignmentRepository_GetByID(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	id := "test-id"
	dueDate := time.Now()
	createdAt := time.Now()
	updatedAt := time.Now()

	rows := sqlmock.NewRows([]string{
		"id", "course_id", "title", "description", "max_points", "due_date",
		"late_penalty_percent", "max_late_days", "created_at", "updated_at",
	}).AddRow(
		id, "COURSE-001", "Assignment 1", "Test", 100.0, dueDate,
		10, 3, createdAt, updatedAt,
	)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, course_id, title, description, max_points, due_date`)).
		WithArgs(id).
		WillReturnRows(rows)

	assignment, err := repo.GetByID(ctx, id)

	assert.NoError(t, err)
	assert.NotNil(t, assignment)
	assert.Equal(t, id, assignment.ID)
	assert.Equal(t, "COURSE-001", assignment.CourseID)
	assert.Equal(t, "Assignment 1", assignment.Title)
	assert.Equal(t, 100.0, assignment.MaxPoints)
	assert.Equal(t, 10, assignment.LatePolicy.PenaltyPercentPerDay)
	assert.Equal(t, 3, assignment.LatePolicy.MaxLateDays)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestAssignmentRepository_GetByID_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	id := "non-existent-id"

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, course_id, title, description, max_points, due_date`)).
		WithArgs(id).
		WillReturnError(sql.ErrNoRows)

	assignment, err := repo.GetByID(ctx, id)

	assert.Error(t, err)
	assert.Nil(t, assignment)
	assert.Contains(t, err.Error(), "assignment not found")
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestAssignmentRepository_Update(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	assignment := &models.Assignment{
		ID:          "test-id",
		CourseID:    "COURSE-001",
		Title:       "Updated Assignment",
		Description: "Updated description",
		MaxPoints:   150.0,
		DueDate:     time.Now(),
		LatePolicy: models.LatePolicy{
			PenaltyPercentPerDay: 15,
			MaxLateDays:          5,
		},
	}

	mock.ExpectExec(regexp.QuoteMeta(`UPDATE assignments`)).
		WithArgs(
			assignment.ID,
			assignment.Title,
			assignment.Description,
			assignment.MaxPoints,
			assignment.DueDate,
			assignment.LatePolicy.PenaltyPercentPerDay,
			assignment.LatePolicy.MaxLateDays,
			sqlmock.AnyArg(), // updated_at
		).
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.Update(ctx, assignment)

	assert.NoError(t, err)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestAssignmentRepository_Update_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	assignment := &models.Assignment{
		ID:        "non-existent-id",
		Title:     "Test",
		MaxPoints: 100.0,
		DueDate:   time.Now(),
	}

	mock.ExpectExec(regexp.QuoteMeta(`UPDATE assignments`)).
		WillReturnResult(sqlmock.NewResult(0, 0)) // 0 rows affected

	err = repo.Update(ctx, assignment)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "assignment not found")
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestAssignmentRepository_Delete(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	id := "test-id"

	mock.ExpectExec(regexp.QuoteMeta(`DELETE FROM assignments WHERE id = $1`)).
		WithArgs(id).
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.Delete(ctx, id)

	assert.NoError(t, err)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestAssignmentRepository_Delete_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	id := "non-existent-id"

	mock.ExpectExec(regexp.QuoteMeta(`DELETE FROM assignments WHERE id = $1`)).
		WithArgs(id).
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.Delete(ctx, id)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "assignment not found")
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestAssignmentRepository_ListByCourse(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	courseID := "COURSE-001"
	page := 1
	pageSize := 10

	// Mock count query
	countRows := sqlmock.NewRows([]string{"count"}).AddRow(2)
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT COUNT(*) FROM assignments WHERE course_id = $1`)).
		WithArgs(courseID).
		WillReturnRows(countRows)

	// Mock list query
	dueDate := time.Now()
	rows := sqlmock.NewRows([]string{
		"id", "course_id", "title", "description", "max_points", "due_date",
		"late_penalty_percent", "max_late_days", "created_at", "updated_at",
	}).
		AddRow("id1", courseID, "Assignment 1", "Desc 1", 100.0, dueDate, 10, 3, time.Now(), time.Now()).
		AddRow("id2", courseID, "Assignment 2", "Desc 2", 150.0, dueDate, 15, 5, time.Now(), time.Now())

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT id, course_id, title, description, max_points, due_date`)).
		WithArgs(courseID, pageSize, 0).
		WillReturnRows(rows)

	assignments, total, err := repo.ListByCourse(ctx, courseID, page, pageSize)

	assert.NoError(t, err)
	assert.Equal(t, 2, total)
	assert.Len(t, assignments, 2)
	assert.Equal(t, "Assignment 1", assignments[0].Title)
	assert.Equal(t, "Assignment 2", assignments[1].Title)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestAssignmentRepository_HasSubmissions(t *testing.T) {
	db, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create mock: %v", err)
	}
	defer db.Close()

	repo := NewAssignmentRepository(db)
	ctx := context.Background()

	t.Run("has submissions", func(t *testing.T) {
		id := "test-id"
		rows := sqlmock.NewRows([]string{"exists"}).AddRow(true)

		mock.ExpectQuery(regexp.QuoteMeta(`SELECT EXISTS(SELECT 1 FROM submissions WHERE assignment_id = $1)`)).
			WithArgs(id).
			WillReturnRows(rows)

		exists, err := repo.HasSubmissions(ctx, id)

		assert.NoError(t, err)
		assert.True(t, exists)
		assert.NoError(t, mock.ExpectationsWereMet())
	})

	t.Run("no submissions", func(t *testing.T) {
		id := "test-id"
		rows := sqlmock.NewRows([]string{"exists"}).AddRow(false)

		mock.ExpectQuery(regexp.QuoteMeta(`SELECT EXISTS(SELECT 1 FROM submissions WHERE assignment_id = $1)`)).
			WithArgs(id).
			WillReturnRows(rows)

		exists, err := repo.HasSubmissions(ctx, id)

		assert.NoError(t, err)
		assert.False(t, exists)
		assert.NoError(t, mock.ExpectationsWereMet())
	})
}
