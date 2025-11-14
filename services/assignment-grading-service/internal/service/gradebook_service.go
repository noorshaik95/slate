package service

import (
	"bytes"
	"context"
	"encoding/csv"
	"fmt"

	"slate/services/assignment-grading-service/internal/models"
	"slate/services/assignment-grading-service/internal/repository"
)

type gradebookService struct {
	assignmentRepo repository.AssignmentRepository
	submissionRepo repository.SubmissionRepository
	gradeRepo      repository.GradeRepository
}

// NewGradebookService creates a new gradebook service
func NewGradebookService(
	assignmentRepo repository.AssignmentRepository,
	submissionRepo repository.SubmissionRepository,
	gradeRepo repository.GradeRepository,
) GradebookService {
	return &gradebookService{
		assignmentRepo: assignmentRepo,
		submissionRepo: submissionRepo,
		gradeRepo:      gradeRepo,
	}
}

// GetStudentGradebook gets the gradebook for a single student
func (s *gradebookService) GetStudentGradebook(ctx context.Context, studentID, courseID string) (*StudentGradebook, error) {
	// Get all assignments for the course
	assignments, _, err := s.assignmentRepo.ListByCourse(ctx, courseID, 1, 1000)
	if err != nil {
		return nil, fmt.Errorf("failed to list assignments: %w", err)
	}

	// Get all grades for the student
	grades, err := s.gradeRepo.ListByStudent(ctx, studentID, courseID)
	if err != nil {
		return nil, fmt.Errorf("failed to list grades: %w", err)
	}

	// Create grade map for quick lookup
	gradeMap := make(map[string]*models.Grade)
	for _, grade := range grades {
		gradeMap[grade.AssignmentID] = grade
	}

	// Get all submissions for the student
	submissions, err := s.submissionRepo.ListByStudent(ctx, studentID, courseID)
	if err != nil {
		return nil, fmt.Errorf("failed to list submissions: %w", err)
	}

	// Create submission map for quick lookup
	submissionMap := make(map[string]*models.Submission)
	for _, submission := range submissions {
		submissionMap[submission.AssignmentID] = submission
	}

	// Build gradebook entries
	var entries []GradebookEntry
	var totalPoints, earnedPoints float64

	for _, assignment := range assignments {
		entry := GradebookEntry{
			AssignmentID:    assignment.ID,
			AssignmentTitle: assignment.Title,
			MaxPoints:       assignment.MaxPoints,
			DueDate:         assignment.DueDate,
		}

		// Add submission info if exists
		if submission, ok := submissionMap[assignment.ID]; ok {
			entry.SubmittedAt = &submission.SubmittedAt
			entry.IsLate = submission.IsLate
		}

		// Add grade info if exists
		if grade, ok := gradeMap[assignment.ID]; ok {
			entry.Score = grade.Score
			entry.AdjustedScore = grade.AdjustedScore
			entry.Status = grade.Status
			earnedPoints += grade.AdjustedScore
		} else {
			entry.Status = "not_graded"
		}

		totalPoints += assignment.MaxPoints
		entries = append(entries, entry)
	}

	// Calculate percentage
	percentage := 0.0
	if totalPoints > 0 {
		percentage = (earnedPoints / totalPoints) * 100
	}

	// Calculate letter grade
	letterGrade := calculateLetterGrade(percentage)

	return &StudentGradebook{
		StudentID:    studentID,
		CourseID:     courseID,
		Entries:      entries,
		TotalPoints:  totalPoints,
		EarnedPoints: earnedPoints,
		Percentage:   percentage,
		LetterGrade:  letterGrade,
	}, nil
}

// GetCourseGradebook gets the gradebook for an entire course
func (s *gradebookService) GetCourseGradebook(ctx context.Context, courseID string) (*CourseGradebook, error) {
	// Get all assignments for the course
	assignments, _, err := s.assignmentRepo.ListByCourse(ctx, courseID, 1, 1000)
	if err != nil {
		return nil, fmt.Errorf("failed to list assignments: %w", err)
	}

	// Get all grades for the course
	grades, err := s.gradeRepo.ListByCourse(ctx, courseID)
	if err != nil {
		return nil, fmt.Errorf("failed to list grades: %w", err)
	}

	// Group grades by student
	studentGrades := make(map[string][]*models.Grade)
	for _, grade := range grades {
		studentGrades[grade.StudentID] = append(studentGrades[grade.StudentID], grade)
	}

	// Calculate total possible points
	var totalPoints float64
	for _, assignment := range assignments {
		totalPoints += assignment.MaxPoints
	}

	// Build student summaries
	var students []StudentSummary
	for studentID, studentGradeList := range studentGrades {
		var earnedPoints float64
		gradeMap := make(map[string]*models.Grade)
		for _, grade := range studentGradeList {
			gradeMap[grade.AssignmentID] = grade
			earnedPoints += grade.AdjustedScore
		}

		// Build entries for this student
		var entries []GradebookEntry
		for _, assignment := range assignments {
			entry := GradebookEntry{
				AssignmentID:    assignment.ID,
				AssignmentTitle: assignment.Title,
				MaxPoints:       assignment.MaxPoints,
				DueDate:         assignment.DueDate,
			}

			if grade, ok := gradeMap[assignment.ID]; ok {
				entry.Score = grade.Score
				entry.AdjustedScore = grade.AdjustedScore
				entry.Status = grade.Status
			} else {
				entry.Status = "not_graded"
			}

			entries = append(entries, entry)
		}

		percentage := 0.0
		if totalPoints > 0 {
			percentage = (earnedPoints / totalPoints) * 100
		}

		students = append(students, StudentSummary{
			StudentID:    studentID,
			TotalPoints:  totalPoints,
			EarnedPoints: earnedPoints,
			Percentage:   percentage,
			LetterGrade:  calculateLetterGrade(percentage),
			Entries:      entries,
		})
	}

	return &CourseGradebook{
		CourseID: courseID,
		Students: students,
	}, nil
}

// GetGradeStatistics gets statistics for an assignment
func (s *gradebookService) GetGradeStatistics(ctx context.Context, assignmentID string) (*repository.GradeStatistics, error) {
	stats, err := s.gradeRepo.GetStatistics(ctx, assignmentID)
	if err != nil {
		return nil, fmt.Errorf("failed to get statistics: %w", err)
	}

	return stats, nil
}

// ExportGrades exports grades to CSV format
func (s *gradebookService) ExportGrades(ctx context.Context, courseID, format string) ([]byte, error) {
	if format != "csv" {
		return nil, fmt.Errorf("unsupported format: %s", format)
	}

	// Get course gradebook
	gradebook, err := s.GetCourseGradebook(ctx, courseID)
	if err != nil {
		return nil, fmt.Errorf("failed to get gradebook: %w", err)
	}

	// Create CSV
	var buf bytes.Buffer
	writer := csv.NewWriter(&buf)

	// Write header
	header := []string{"Student ID", "Total Points", "Earned Points", "Percentage", "Letter Grade"}
	if err := writer.Write(header); err != nil {
		return nil, fmt.Errorf("failed to write CSV header: %w", err)
	}

	// Write data
	for _, student := range gradebook.Students {
		row := []string{
			student.StudentID,
			fmt.Sprintf("%.2f", student.TotalPoints),
			fmt.Sprintf("%.2f", student.EarnedPoints),
			fmt.Sprintf("%.2f", student.Percentage),
			student.LetterGrade,
		}
		if err := writer.Write(row); err != nil {
			return nil, fmt.Errorf("failed to write CSV row: %w", err)
		}
	}

	writer.Flush()
	if err := writer.Error(); err != nil {
		return nil, fmt.Errorf("CSV writer error: %w", err)
	}

	return buf.Bytes(), nil
}

// calculateLetterGrade converts a percentage to a letter grade
func calculateLetterGrade(percentage float64) string {
	switch {
	case percentage >= 90:
		return "A"
	case percentage >= 80:
		return "B"
	case percentage >= 70:
		return "C"
	case percentage >= 60:
		return "D"
	default:
		return "F"
	}
}
