// +build integration

package internal

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"testing"
	"time"

	"slate/services/assignment-grading-service/internal/models"
	"slate/services/assignment-grading-service/internal/repository"
	"slate/services/assignment-grading-service/internal/service"
	"slate/services/assignment-grading-service/pkg/kafka"
	"slate/services/assignment-grading-service/pkg/storage"

	_ "github.com/lib/pq"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Integration tests require a PostgreSQL database
// Run with: go test -tags=integration ./internal/...

func setupTestDB(t *testing.T) *sql.DB {
	dsn := os.Getenv("TEST_DATABASE_URL")
	if dsn == "" {
		dsn = "host=localhost port=5432 user=postgres password=postgres dbname=assignment_grading_test sslmode=disable"
	}

	db, err := sql.Open("postgres", dsn)
	require.NoError(t, err)

	err = db.Ping()
	require.NoError(t, err)

	// Clean up tables
	cleanupTables(t, db)

	// Create tables
	createTables(t, db)

	return db
}

func cleanupTables(t *testing.T, db *sql.DB) {
	_, err := db.Exec("DROP TABLE IF EXISTS grades CASCADE")
	require.NoError(t, err)
	_, err = db.Exec("DROP TABLE IF EXISTS submissions CASCADE")
	require.NoError(t, err)
	_, err = db.Exec("DROP TABLE IF EXISTS assignments CASCADE")
	require.NoError(t, err)
}

func createTables(t *testing.T, db *sql.DB) {
	schema := `
		CREATE TABLE IF NOT EXISTS assignments (
			id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
			course_id VARCHAR(255) NOT NULL,
			title VARCHAR(500) NOT NULL,
			description TEXT,
			max_points DECIMAL(10, 2) NOT NULL CHECK (max_points > 0),
			due_date TIMESTAMP NOT NULL,
			late_penalty_percent INT DEFAULT 0 CHECK (late_penalty_percent >= 0 AND late_penalty_percent <= 100),
			max_late_days INT DEFAULT 0 CHECK (max_late_days >= 0),
			created_at TIMESTAMP NOT NULL DEFAULT NOW(),
			updated_at TIMESTAMP NOT NULL DEFAULT NOW()
		);

		CREATE TABLE IF NOT EXISTS submissions (
			id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
			assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
			student_id VARCHAR(255) NOT NULL,
			file_path VARCHAR(1000) NOT NULL,
			submitted_at TIMESTAMP NOT NULL DEFAULT NOW(),
			status VARCHAR(50) NOT NULL DEFAULT 'submitted' CHECK (status IN ('submitted', 'graded', 'returned')),
			is_late BOOLEAN NOT NULL DEFAULT FALSE,
			days_late INT NOT NULL DEFAULT 0,
			created_at TIMESTAMP NOT NULL DEFAULT NOW(),
			updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
			UNIQUE(assignment_id, student_id)
		);

		CREATE TABLE IF NOT EXISTS grades (
			id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
			submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
			student_id VARCHAR(255) NOT NULL,
			assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
			score DECIMAL(10, 2) NOT NULL CHECK (score >= 0),
			adjusted_score DECIMAL(10, 2) NOT NULL CHECK (adjusted_score >= 0),
			feedback TEXT,
			status VARCHAR(50) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published')),
			graded_at TIMESTAMP,
			published_at TIMESTAMP,
			graded_by VARCHAR(255) NOT NULL,
			created_at TIMESTAMP NOT NULL DEFAULT NOW(),
			updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
			UNIQUE(submission_id)
		);
	`

	_, err := db.Exec(schema)
	require.NoError(t, err)
}

func teardownTestDB(t *testing.T, db *sql.DB) {
	cleanupTables(t, db)
	db.Close()
}

func TestIntegration_CompleteAssignmentWorkflow(t *testing.T) {
	db := setupTestDB(t)
	defer teardownTestDB(t, db)

	ctx := context.Background()

	// Setup repositories
	assignmentRepo := repository.NewAssignmentRepository(db)
	submissionRepo := repository.NewSubmissionRepository(db)
	gradeRepo := repository.NewGradeRepository(db)

	// Setup services
	kafkaProducer := kafka.NewProducer([]string{}, "", false) // Disabled for tests
	defer kafkaProducer.Close()

	tempDir := t.TempDir()
	fileStorage, err := storage.NewLocalFileStorage(tempDir, 10*1024*1024)
	require.NoError(t, err)

	assignmentService := service.NewAssignmentService(assignmentRepo, kafkaProducer)
	submissionService := service.NewSubmissionService(assignmentRepo, submissionRepo, fileStorage, kafkaProducer)
	gradingService := service.NewGradingService(assignmentRepo, submissionRepo, gradeRepo, kafkaProducer)
	gradebookService := service.NewGradebookService(assignmentRepo, submissionRepo, gradeRepo)

	// Step 1: Create an assignment
	dueDate := time.Now().Add(-24 * time.Hour) // Due yesterday (to test late submissions)
	latePolicy := models.LatePolicy{
		PenaltyPercentPerDay: 10,
		MaxLateDays:          3,
	}

	assignment, err := assignmentService.CreateAssignment(
		ctx,
		"COURSE-001",
		"Homework 1",
		"First homework assignment",
		100.0,
		dueDate,
		latePolicy,
	)
	require.NoError(t, err)
	assert.NotEmpty(t, assignment.ID)
	assert.Equal(t, "Homework 1", assignment.Title)

	// Step 2: Submit assignment (late)
	fileContent := []byte("Student's homework submission")
	submission, err := submissionService.SubmitAssignment(
		ctx,
		assignment.ID,
		"STUDENT-001",
		fileContent,
		"homework.pdf",
		"application/pdf",
	)
	require.NoError(t, err)
	assert.NotEmpty(t, submission.ID)
	assert.True(t, submission.IsLate)
	assert.Equal(t, 1, submission.DaysLate)

	// Verify file was saved
	exists, err := fileStorage.Exists(submission.FilePath)
	require.NoError(t, err)
	assert.True(t, exists)

	// Step 3: Grade the submission
	grade, err := gradingService.CreateGrade(
		ctx,
		submission.ID,
		90.0,
		"Good work, but submitted late",
		"INSTRUCTOR-001",
	)
	require.NoError(t, err)
	assert.NotEmpty(t, grade.ID)
	assert.Equal(t, 90.0, grade.Score)
	assert.Equal(t, 81.0, grade.AdjustedScore) // 90 - 10% penalty = 81
	assert.Equal(t, models.GradeStatusDraft, grade.Status)

	// Step 4: Publish the grade
	publishedGrade, err := gradingService.PublishGrade(ctx, grade.ID)
	require.NoError(t, err)
	assert.Equal(t, models.GradeStatusPublished, publishedGrade.Status)
	assert.NotNil(t, publishedGrade.PublishedAt)

	// Step 5: Get student gradebook
	gradebook, err := gradebookService.GetStudentGradebook(ctx, "STUDENT-001", "COURSE-001")
	require.NoError(t, err)
	assert.Equal(t, "STUDENT-001", gradebook.StudentID)
	assert.Equal(t, 100.0, gradebook.TotalPoints)
	assert.Equal(t, 81.0, gradebook.EarnedPoints)
	assert.Equal(t, 81.0, gradebook.Percentage)
	assert.Equal(t, "B", gradebook.LetterGrade)

	// Step 6: Get grade statistics
	stats, err := gradebookService.GetGradeStatistics(ctx, assignment.ID)
	require.NoError(t, err)
	assert.Equal(t, 1, stats.TotalSubmissions)
	assert.Equal(t, 1, stats.GradedCount)
	assert.Equal(t, 81.0, stats.Mean)
}

func TestIntegration_MultipleStudentsWorkflow(t *testing.T) {
	db := setupTestDB(t)
	defer teardownTestDB(t, db)

	ctx := context.Background()

	// Setup
	assignmentRepo := repository.NewAssignmentRepository(db)
	submissionRepo := repository.NewSubmissionRepository(db)
	gradeRepo := repository.NewGradeRepository(db)

	kafkaProducer := kafka.NewProducer([]string{}, "", false)
	defer kafkaProducer.Close()

	tempDir := t.TempDir()
	fileStorage, err := storage.NewLocalFileStorage(tempDir, 10*1024*1024)
	require.NoError(t, err)

	assignmentService := service.NewAssignmentService(assignmentRepo, kafkaProducer)
	submissionService := service.NewSubmissionService(assignmentRepo, submissionRepo, fileStorage, kafkaProducer)
	gradingService := service.NewGradingService(assignmentRepo, submissionRepo, gradeRepo, kafkaProducer)
	gradebookService := service.NewGradebookService(assignmentRepo, submissionRepo, gradeRepo)

	// Create assignment
	dueDate := time.Now().Add(24 * time.Hour) // Due tomorrow
	assignment, err := assignmentService.CreateAssignment(
		ctx,
		"COURSE-001",
		"Quiz 1",
		"First quiz",
		50.0,
		dueDate,
		models.LatePolicy{PenaltyPercentPerDay: 10, MaxLateDays: 2},
	)
	require.NoError(t, err)

	// Multiple students submit
	students := []struct {
		id    string
		score float64
	}{
		{"STUDENT-001", 45.0},
		{"STUDENT-002", 40.0},
		{"STUDENT-003", 48.0},
	}

	for _, student := range students {
		// Submit
		submission, err := submissionService.SubmitAssignment(
			ctx,
			assignment.ID,
			student.id,
			[]byte(fmt.Sprintf("Submission from %s", student.id)),
			"quiz.pdf",
			"application/pdf",
		)
		require.NoError(t, err)

		// Grade
		grade, err := gradingService.CreateGrade(
			ctx,
			submission.ID,
			student.score,
			"Graded",
			"INSTRUCTOR-001",
		)
		require.NoError(t, err)

		// Publish
		_, err = gradingService.PublishGrade(ctx, grade.ID)
		require.NoError(t, err)
	}

	// Get course gradebook
	courseGradebook, err := gradebookService.GetCourseGradebook(ctx, "COURSE-001")
	require.NoError(t, err)
	assert.Len(t, courseGradebook.Students, 3)

	// Get statistics
	stats, err := gradebookService.GetGradeStatistics(ctx, assignment.ID)
	require.NoError(t, err)
	assert.Equal(t, 3, stats.TotalSubmissions)
	assert.Equal(t, 3, stats.GradedCount)
	assert.InDelta(t, 44.33, stats.Mean, 0.1)
	assert.Equal(t, 40.0, stats.MinScore)
	assert.Equal(t, 48.0, stats.MaxScore)
}

func TestIntegration_LatePenaltyScenarios(t *testing.T) {
	db := setupTestDB(t)
	defer teardownTestDB(t, db)

	ctx := context.Background()

	assignmentRepo := repository.NewAssignmentRepository(db)
	submissionRepo := repository.NewSubmissionRepository(db)
	gradeRepo := repository.NewGradeRepository(db)

	kafkaProducer := kafka.NewProducer([]string{}, "", false)
	defer kafkaProducer.Close()

	tempDir := t.TempDir()
	fileStorage, err := storage.NewLocalFileStorage(tempDir, 10*1024*1024)
	require.NoError(t, err)

	assignmentService := service.NewAssignmentService(assignmentRepo, kafkaProducer)
	submissionService := service.NewSubmissionService(assignmentRepo, submissionRepo, fileStorage, kafkaProducer)
	gradingService := service.NewGradingService(assignmentRepo, submissionRepo, gradeRepo, kafkaProducer)

	tests := []struct {
		name               string
		daysBeforeDue      int
		score              float64
		expectedAdjusted   float64
		penaltyPercent     int
		maxLateDays        int
	}{
		{
			name:             "On time submission",
			daysBeforeDue:    1,
			score:            100.0,
			expectedAdjusted: 100.0,
			penaltyPercent:   10,
			maxLateDays:      3,
		},
		{
			name:             "1 day late with 10% penalty",
			daysBeforeDue:    -1,
			score:            100.0,
			expectedAdjusted: 90.0,
			penaltyPercent:   10,
			maxLateDays:      3,
		},
		{
			name:             "2 days late with 10% penalty",
			daysBeforeDue:    -2,
			score:            100.0,
			expectedAdjusted: 80.0,
			penaltyPercent:   10,
			maxLateDays:      3,
		},
	}

	for i, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create assignment with specific due date
			dueDate := time.Now().Add(time.Duration(tt.daysBeforeDue) * 24 * time.Hour)
			assignment, err := assignmentService.CreateAssignment(
				ctx,
				"COURSE-001",
				fmt.Sprintf("Assignment %d", i),
				"Test",
				100.0,
				dueDate,
				models.LatePolicy{
					PenaltyPercentPerDay: tt.penaltyPercent,
					MaxLateDays:          tt.maxLateDays,
				},
			)
			require.NoError(t, err)

			// Submit
			submission, err := submissionService.SubmitAssignment(
				ctx,
				assignment.ID,
				fmt.Sprintf("STUDENT-%d", i),
				[]byte("test"),
				"test.pdf",
				"application/pdf",
			)
			require.NoError(t, err)

			// Grade
			grade, err := gradingService.CreateGrade(
				ctx,
				submission.ID,
				tt.score,
				"Test",
				"INSTRUCTOR-001",
			)
			require.NoError(t, err)

			assert.InDelta(t, tt.expectedAdjusted, grade.AdjustedScore, 0.1)
		})
	}
}

func TestIntegration_ResubmissionReplacement(t *testing.T) {
	db := setupTestDB(t)
	defer teardownTestDB(t, db)

	ctx := context.Background()

	assignmentRepo := repository.NewAssignmentRepository(db)
	submissionRepo := repository.NewSubmissionRepository(db)

	kafkaProducer := kafka.NewProducer([]string{}, "", false)
	defer kafkaProducer.Close()

	tempDir := t.TempDir()
	fileStorage, err := storage.NewLocalFileStorage(tempDir, 10*1024*1024)
	require.NoError(t, err)

	assignmentService := service.NewAssignmentService(assignmentRepo, kafkaProducer)
	submissionService := service.NewSubmissionService(assignmentRepo, submissionRepo, fileStorage, kafkaProducer)

	// Create assignment
	assignment, err := assignmentService.CreateAssignment(
		ctx,
		"COURSE-001",
		"Test Assignment",
		"Test",
		100.0,
		time.Now().Add(24*time.Hour),
		models.LatePolicy{PenaltyPercentPerDay: 10, MaxLateDays: 3},
	)
	require.NoError(t, err)

	// First submission
	firstSubmission, err := submissionService.SubmitAssignment(
		ctx,
		assignment.ID,
		"STUDENT-001",
		[]byte("First attempt"),
		"first.pdf",
		"application/pdf",
	)
	require.NoError(t, err)
	firstID := firstSubmission.ID

	// Second submission (should replace)
	secondSubmission, err := submissionService.SubmitAssignment(
		ctx,
		assignment.ID,
		"STUDENT-001",
		[]byte("Second attempt"),
		"second.pdf",
		"application/pdf",
	)
	require.NoError(t, err)

	// Should have same ID (replaced, not new)
	assert.Equal(t, firstID, secondSubmission.ID)

	// Verify only one submission exists
	submissions, err := submissionService.ListSubmissions(ctx, assignment.ID, "submitted_at", "DESC")
	require.NoError(t, err)
	assert.Len(t, submissions, 1)
}
