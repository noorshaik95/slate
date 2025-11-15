package service

import (
	"context"
	"fmt"

	"slate/services/assignment-grading-service/internal/models"
	"slate/services/assignment-grading-service/internal/repository"
	"slate/services/assignment-grading-service/pkg/kafka"
)

type gradingService struct {
	assignmentRepo repository.AssignmentRepository
	submissionRepo repository.SubmissionRepository
	gradeRepo      repository.GradeRepository
	producer       kafka.EventPublisher
	latePolicyCalc *LatePolicyCalculator
}

// NewGradingService creates a new grading service
func NewGradingService(
	assignmentRepo repository.AssignmentRepository,
	submissionRepo repository.SubmissionRepository,
	gradeRepo repository.GradeRepository,
	producer kafka.EventPublisher,
) GradingService {
	return &gradingService{
		assignmentRepo: assignmentRepo,
		submissionRepo: submissionRepo,
		gradeRepo:      gradeRepo,
		producer:       producer,
		latePolicyCalc: NewLatePolicyCalculator(),
	}
}

// CreateGrade creates a new grade for a submission
func (s *gradingService) CreateGrade(ctx context.Context, submissionID string, score float64, feedback, gradedBy string) (*models.Grade, error) {
	// Get submission
	submission, err := s.submissionRepo.GetByID(ctx, submissionID)
	if err != nil {
		return nil, fmt.Errorf("failed to get submission: %w", err)
	}

	// Get assignment to validate score and apply late penalty
	assignment, err := s.assignmentRepo.GetByID(ctx, submission.AssignmentID)
	if err != nil {
		return nil, fmt.Errorf("failed to get assignment: %w", err)
	}

	// Validate score
	if score < 0 || score > assignment.MaxPoints {
		return nil, fmt.Errorf("score must be between 0 and %f", assignment.MaxPoints)
	}

	// Apply late penalty
	adjustedScore := s.latePolicyCalc.ApplyPenalty(score, submission.DaysLate, assignment.LatePolicy)

	// Create grade model
	grade := &models.Grade{
		SubmissionID:  submissionID,
		StudentID:     submission.StudentID,
		AssignmentID:  assignment.ID,
		Score:         score,
		AdjustedScore: adjustedScore,
		Feedback:      feedback,
		Status:        models.GradeStatusDraft,
		GradedBy:      gradedBy,
	}

	// Validate
	if err := grade.Validate(); err != nil {
		return nil, fmt.Errorf("validation failed: %w", err)
	}

	if err := grade.ValidateScore(assignment.MaxPoints); err != nil {
		return nil, fmt.Errorf("score validation failed: %w", err)
	}

	// Create in repository
	if err := s.gradeRepo.Create(ctx, grade); err != nil {
		return nil, fmt.Errorf("failed to create grade: %w", err)
	}

	// Update submission status
	submission.Status = models.StatusGraded
	if err := s.submissionRepo.Update(ctx, submission); err != nil {
		fmt.Printf("Failed to update submission status: %v\n", err)
	}

	return grade, nil
}

// UpdateGrade updates an existing grade (only draft grades can be updated)
func (s *gradingService) UpdateGrade(ctx context.Context, id string, score float64, feedback string) (*models.Grade, error) {
	// Get existing grade
	grade, err := s.gradeRepo.GetByID(ctx, id)
	if err != nil {
		return nil, fmt.Errorf("failed to get grade: %w", err)
	}

	// Only draft grades can be updated
	if !grade.IsDraft() {
		return nil, fmt.Errorf("only draft grades can be updated")
	}

	// Get submission and assignment to validate and apply late penalty
	submission, err := s.submissionRepo.GetByID(ctx, grade.SubmissionID)
	if err != nil {
		return nil, fmt.Errorf("failed to get submission: %w", err)
	}

	assignment, err := s.assignmentRepo.GetByID(ctx, submission.AssignmentID)
	if err != nil {
		return nil, fmt.Errorf("failed to get assignment: %w", err)
	}

	// Validate score
	if score < 0 || score > assignment.MaxPoints {
		return nil, fmt.Errorf("score must be between 0 and %f", assignment.MaxPoints)
	}

	// Apply late penalty
	adjustedScore := s.latePolicyCalc.ApplyPenalty(score, submission.DaysLate, assignment.LatePolicy)

	// Update fields
	grade.Score = score
	grade.AdjustedScore = adjustedScore
	grade.Feedback = feedback

	// Update in repository
	if err := s.gradeRepo.Update(ctx, grade); err != nil {
		return nil, fmt.Errorf("failed to update grade: %w", err)
	}

	return grade, nil
}

// PublishGrade publishes a grade (makes it visible to student)
func (s *gradingService) PublishGrade(ctx context.Context, id string) (*models.Grade, error) {
	// Get grade
	grade, err := s.gradeRepo.GetByID(ctx, id)
	if err != nil {
		return nil, fmt.Errorf("failed to get grade: %w", err)
	}

	// Publish grade
	if publishErr := grade.Publish(); publishErr != nil {
		return nil, fmt.Errorf("failed to publish grade: %w", publishErr)
	}

	// Update in repository
	if updateErr := s.gradeRepo.Update(ctx, grade); updateErr != nil {
		return nil, fmt.Errorf("failed to update grade: %w", updateErr)
	}

	// Update submission status
	submission, err := s.submissionRepo.GetByID(ctx, grade.SubmissionID)
	if err == nil {
		submission.Status = models.StatusReturned
		if err := s.submissionRepo.Update(ctx, submission); err != nil {
			fmt.Printf("Failed to update submission status: %v\n", err)
		}
	}

	// Publish event
	event := kafka.NewGradePublishedEvent(grade.ID, grade.AssignmentID, grade.StudentID, grade.Score, grade.AdjustedScore)
	if err := s.producer.PublishEvent(ctx, event); err != nil {
		fmt.Printf("Failed to publish grade.published event: %v\n", err)
	}

	return grade, nil
}

// GetGrade retrieves a grade by ID
func (s *gradingService) GetGrade(ctx context.Context, id string) (*models.Grade, error) {
	grade, err := s.gradeRepo.GetByID(ctx, id)
	if err != nil {
		return nil, fmt.Errorf("failed to get grade: %w", err)
	}

	return grade, nil
}
