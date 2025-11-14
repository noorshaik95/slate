package service

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"slate/services/assignment-grading-service/internal/models"
	"slate/services/assignment-grading-service/internal/repository"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestGradebookService_GetStudentGradebook(t *testing.T) {
	ctx := context.Background()

	t.Run("get gradebook for student with grades", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		dueDate := time.Now().Add(24 * time.Hour)
		assignments := []*models.Assignment{
			{
				ID:        "assignment1",
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: 100.0,
				DueDate:   dueDate,
			},
			{
				ID:        "assignment2",
				CourseID:  "COURSE-001",
				Title:     "Assignment 2",
				MaxPoints: 50.0,
				DueDate:   dueDate,
			},
		}

		grades := []*models.Grade{
			{
				ID:            "grade1",
				AssignmentID:  "assignment1",
				StudentID:     "STUDENT-001",
				Score:         90.0,
				AdjustedScore: 90.0,
				Status:        models.GradeStatusPublished,
			},
			{
				ID:            "grade2",
				AssignmentID:  "assignment2",
				StudentID:     "STUDENT-001",
				Score:         45.0,
				AdjustedScore: 45.0,
				Status:        models.GradeStatusPublished,
			},
		}

		submissions := []*models.Submission{
			{
				ID:           "sub1",
				AssignmentID: "assignment1",
				StudentID:    "STUDENT-001",
				SubmittedAt:  time.Now(),
				IsLate:       false,
			},
			{
				ID:           "sub2",
				AssignmentID: "assignment2",
				StudentID:    "STUDENT-001",
				SubmittedAt:  time.Now(),
				IsLate:       false,
			},
		}

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(assignments, 2, nil)
		mockGradeRepo.On("ListByStudent", ctx, "STUDENT-001", "COURSE-001").Return(grades, nil)
		mockSubmissionRepo.On("ListByStudent", ctx, "STUDENT-001", "COURSE-001").Return(submissions, nil)

		gradebook, err := service.GetStudentGradebook(ctx, "STUDENT-001", "COURSE-001")

		assert.NoError(t, err)
		assert.NotNil(t, gradebook)
		assert.Equal(t, "STUDENT-001", gradebook.StudentID)
		assert.Equal(t, "COURSE-001", gradebook.CourseID)
		assert.Equal(t, 150.0, gradebook.TotalPoints)    // 100 + 50
		assert.Equal(t, 135.0, gradebook.EarnedPoints)   // 90 + 45
		assert.Equal(t, 90.0, gradebook.Percentage)      // 135/150 * 100 = 90%
		assert.Equal(t, "A", gradebook.LetterGrade)      // 90% = A
		assert.Len(t, gradebook.Entries, 2)
		mockAssignmentRepo.AssertExpectations(t)
		mockGradeRepo.AssertExpectations(t)
		mockSubmissionRepo.AssertExpectations(t)
	})

	t.Run("get gradebook for student with partial grades", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		dueDate := time.Now().Add(24 * time.Hour)
		assignments := []*models.Assignment{
			{
				ID:        "assignment1",
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: 100.0,
				DueDate:   dueDate,
			},
			{
				ID:        "assignment2",
				CourseID:  "COURSE-001",
				Title:     "Assignment 2",
				MaxPoints: 50.0,
				DueDate:   dueDate,
			},
		}

		// Only grade for assignment 1
		grades := []*models.Grade{
			{
				ID:            "grade1",
				AssignmentID:  "assignment1",
				StudentID:     "STUDENT-001",
				Score:         80.0,
				AdjustedScore: 80.0,
				Status:        models.GradeStatusPublished,
			},
		}

		submissions := []*models.Submission{
			{
				ID:           "sub1",
				AssignmentID: "assignment1",
				StudentID:    "STUDENT-001",
				SubmittedAt:  time.Now(),
				IsLate:       false,
			},
		}

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(assignments, 2, nil)
		mockGradeRepo.On("ListByStudent", ctx, "STUDENT-001", "COURSE-001").Return(grades, nil)
		mockSubmissionRepo.On("ListByStudent", ctx, "STUDENT-001", "COURSE-001").Return(submissions, nil)

		gradebook, err := service.GetStudentGradebook(ctx, "STUDENT-001", "COURSE-001")

		assert.NoError(t, err)
		assert.NotNil(t, gradebook)
		assert.Equal(t, 150.0, gradebook.TotalPoints)    // 100 + 50
		assert.Equal(t, 80.0, gradebook.EarnedPoints)    // Only assignment 1
		assert.InDelta(t, 53.33, gradebook.Percentage, 0.1) // 80/150 * 100
		assert.Equal(t, "F", gradebook.LetterGrade)      // 53.33% = F
		assert.Len(t, gradebook.Entries, 2)

		// Check that ungraded assignment has "not_graded" status
		assert.Equal(t, "not_graded", gradebook.Entries[1].Status)
	})

	t.Run("assignments list error", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(nil, 0, errors.New("database error"))

		gradebook, err := service.GetStudentGradebook(ctx, "STUDENT-001", "COURSE-001")

		assert.Error(t, err)
		assert.Nil(t, gradebook)
		assert.Contains(t, err.Error(), "failed to list assignments")
	})

	t.Run("zero total points", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		// Empty course with no assignments
		assignments := []*models.Assignment{}
		grades := []*models.Grade{}
		submissions := []*models.Submission{}

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(assignments, 0, nil)
		mockGradeRepo.On("ListByStudent", ctx, "STUDENT-001", "COURSE-001").Return(grades, nil)
		mockSubmissionRepo.On("ListByStudent", ctx, "STUDENT-001", "COURSE-001").Return(submissions, nil)

		gradebook, err := service.GetStudentGradebook(ctx, "STUDENT-001", "COURSE-001")

		assert.NoError(t, err)
		assert.NotNil(t, gradebook)
		assert.Equal(t, 0.0, gradebook.TotalPoints)
		assert.Equal(t, 0.0, gradebook.EarnedPoints)
		assert.Equal(t, 0.0, gradebook.Percentage)
		assert.Len(t, gradebook.Entries, 0)
	})
}

func TestGradebookService_GetCourseGradebook(t *testing.T) {
	ctx := context.Background()

	t.Run("get course gradebook with multiple students", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		dueDate := time.Now().Add(24 * time.Hour)
		assignments := []*models.Assignment{
			{
				ID:        "assignment1",
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: 100.0,
				DueDate:   dueDate,
			},
		}

		grades := []*models.Grade{
			{
				ID:            "grade1",
				AssignmentID:  "assignment1",
				StudentID:     "STUDENT-001",
				Score:         90.0,
				AdjustedScore: 90.0,
				Status:        models.GradeStatusPublished,
			},
			{
				ID:            "grade2",
				AssignmentID:  "assignment1",
				StudentID:     "STUDENT-002",
				Score:         80.0,
				AdjustedScore: 80.0,
				Status:        models.GradeStatusPublished,
			},
		}

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(assignments, 1, nil)
		mockGradeRepo.On("ListByCourse", ctx, "COURSE-001").Return(grades, nil)

		gradebook, err := service.GetCourseGradebook(ctx, "COURSE-001")

		assert.NoError(t, err)
		assert.NotNil(t, gradebook)
		assert.Equal(t, "COURSE-001", gradebook.CourseID)
		assert.Len(t, gradebook.Students, 2)

		// Find student 1
		var student1 *StudentSummary
		for i := range gradebook.Students {
			if gradebook.Students[i].StudentID == "STUDENT-001" {
				student1 = &gradebook.Students[i]
				break
			}
		}
		assert.NotNil(t, student1)
		assert.Equal(t, 90.0, student1.EarnedPoints)
		assert.Equal(t, 90.0, student1.Percentage)
		assert.Equal(t, "A", student1.LetterGrade)
	})

	t.Run("assignments list error", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(nil, 0, errors.New("database error"))

		gradebook, err := service.GetCourseGradebook(ctx, "COURSE-001")

		assert.Error(t, err)
		assert.Nil(t, gradebook)
		assert.Contains(t, err.Error(), "failed to list assignments")
	})

	t.Run("grades list error", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		assignments := []*models.Assignment{
			{ID: "assignment1", CourseID: "COURSE-001", MaxPoints: 100.0},
		}

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(assignments, 1, nil)
		mockGradeRepo.On("ListByCourse", ctx, "COURSE-001").Return(nil, errors.New("database error"))

		gradebook, err := service.GetCourseGradebook(ctx, "COURSE-001")

		assert.Error(t, err)
		assert.Nil(t, gradebook)
		assert.Contains(t, err.Error(), "failed to list grades")
	})
}

func TestGradebookService_GetGradeStatistics(t *testing.T) {
	ctx := context.Background()

	t.Run("get statistics successfully", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		expectedStats := &repository.GradeStatistics{
			TotalSubmissions: 10,
			GradedCount:      8,
			Mean:             82.5,
			Median:           84.0,
			StdDeviation:     7.2,
			MinScore:         65.0,
			MaxScore:         98.0,
		}

		mockGradeRepo.On("GetStatistics", ctx, "assignment-id").Return(expectedStats, nil)

		stats, err := service.GetGradeStatistics(ctx, "assignment-id")

		assert.NoError(t, err)
		assert.Equal(t, expectedStats, stats)
		assert.Equal(t, 10, stats.TotalSubmissions)
		assert.Equal(t, 8, stats.GradedCount)
		assert.Equal(t, 82.5, stats.Mean)
		mockGradeRepo.AssertExpectations(t)
	})

	t.Run("repository error", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		mockGradeRepo.On("GetStatistics", ctx, "assignment-id").Return(nil, errors.New("database error"))

		stats, err := service.GetGradeStatistics(ctx, "assignment-id")

		assert.Error(t, err)
		assert.Nil(t, stats)
		assert.Contains(t, err.Error(), "failed to get statistics")
		mockGradeRepo.AssertExpectations(t)
	})
}

func TestGradebookService_ExportGrades(t *testing.T) {
	ctx := context.Background()

	t.Run("export grades to CSV successfully", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		dueDate := time.Now().Add(24 * time.Hour)
		assignments := []*models.Assignment{
			{
				ID:        "assignment1",
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: 100.0,
				DueDate:   dueDate,
			},
		}

		grades := []*models.Grade{
			{
				ID:            "grade1",
				AssignmentID:  "assignment1",
				StudentID:     "STUDENT-001",
				Score:         90.0,
				AdjustedScore: 90.0,
				Status:        models.GradeStatusPublished,
			},
			{
				ID:            "grade2",
				AssignmentID:  "assignment1",
				StudentID:     "STUDENT-002",
				Score:         85.0,
				AdjustedScore: 85.0,
				Status:        models.GradeStatusPublished,
			},
		}

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(assignments, 1, nil)
		mockGradeRepo.On("ListByCourse", ctx, "COURSE-001").Return(grades, nil)

		csvData, err := service.ExportGrades(ctx, "COURSE-001", "csv")

		assert.NoError(t, err)
		assert.NotNil(t, csvData)

		// Verify CSV format
		csvString := string(csvData)
		assert.Contains(t, csvString, "Student ID")
		assert.Contains(t, csvString, "Total Points")
		assert.Contains(t, csvString, "Earned Points")
		assert.Contains(t, csvString, "Percentage")
		assert.Contains(t, csvString, "Letter Grade")
		assert.Contains(t, csvString, "STUDENT-001")
		assert.Contains(t, csvString, "STUDENT-002")
		assert.Contains(t, csvString, "90.00")
		assert.Contains(t, csvString, "85.00")

		// Verify CSV has correct number of rows (header + 2 students)
		lines := strings.Split(strings.TrimSpace(csvString), "\n")
		assert.Len(t, lines, 3) // Header + 2 students
	})

	t.Run("unsupported export format", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		csvData, err := service.ExportGrades(ctx, "COURSE-001", "json")

		assert.Error(t, err)
		assert.Nil(t, csvData)
		assert.Contains(t, err.Error(), "unsupported format")
	})

	t.Run("gradebook retrieval error", func(t *testing.T) {
		mockAssignmentRepo := new(MockAssignmentRepository)
		mockSubmissionRepo := new(MockSubmissionRepository)
		mockGradeRepo := new(MockGradeRepository)
		service := NewGradebookService(mockAssignmentRepo, mockSubmissionRepo, mockGradeRepo)

		mockAssignmentRepo.On("ListByCourse", ctx, "COURSE-001", 1, 1000).Return(nil, 0, errors.New("database error"))

		csvData, err := service.ExportGrades(ctx, "COURSE-001", "csv")

		assert.Error(t, err)
		assert.Nil(t, csvData)
		assert.Contains(t, err.Error(), "failed to get gradebook")
	})
}

func TestCalculateLetterGrade(t *testing.T) {
	tests := []struct {
		percentage  float64
		expected    string
	}{
		{95.0, "A"},
		{90.0, "A"},
		{89.9, "B"},
		{85.0, "B"},
		{80.0, "B"},
		{79.9, "C"},
		{75.0, "C"},
		{70.0, "C"},
		{69.9, "D"},
		{65.0, "D"},
		{60.0, "D"},
		{59.9, "F"},
		{50.0, "F"},
		{0.0, "F"},
	}

	for _, tt := range tests {
		t.Run(tt.expected, func(t *testing.T) {
			result := calculateLetterGrade(tt.percentage)
			assert.Equal(t, tt.expected, result, "For percentage %.2f", tt.percentage)
		})
	}
}
