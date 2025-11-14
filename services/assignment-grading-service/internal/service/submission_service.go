package service

import (
	"bytes"
	"context"
	"fmt"
	"time"

	"slate/services/assignment-grading-service/internal/models"
	"slate/services/assignment-grading-service/internal/repository"
	"slate/services/assignment-grading-service/pkg/kafka"
	"slate/services/assignment-grading-service/pkg/storage"
)

type submissionService struct {
	assignmentRepo repository.AssignmentRepository
	submissionRepo repository.SubmissionRepository
	storage        storage.FileStorage
	producer       *kafka.Producer
	latePolicyCalc *LatePolicyCalculator
}

// NewSubmissionService creates a new submission service
func NewSubmissionService(
	assignmentRepo repository.AssignmentRepository,
	submissionRepo repository.SubmissionRepository,
	storage storage.FileStorage,
	producer *kafka.Producer,
) SubmissionService {
	return &submissionService{
		assignmentRepo: assignmentRepo,
		submissionRepo: submissionRepo,
		storage:        storage,
		producer:       producer,
		latePolicyCalc: NewLatePolicyCalculator(),
	}
}

// SubmitAssignment creates a new submission
func (s *submissionService) SubmitAssignment(ctx context.Context, assignmentID, studentID string, fileContent []byte, fileName, contentType string) (*models.Submission, error) {
	// Get assignment to check late policy and due date
	assignment, err := s.assignmentRepo.GetByID(ctx, assignmentID)
	if err != nil {
		return nil, fmt.Errorf("failed to get assignment: %w", err)
	}

	// Save file
	reader := bytes.NewReader(fileContent)
	filePath, err := s.storage.Save(fileName, reader, contentType)
	if err != nil {
		return nil, fmt.Errorf("failed to save file: %w", err)
	}

	// Calculate if late
	submittedAt := time.Now()
	isLate, daysLate := s.latePolicyCalc.CalculatePenalty(submittedAt, assignment.DueDate, assignment.LatePolicy)

	// Create submission model
	submission := &models.Submission{
		AssignmentID: assignmentID,
		StudentID:    studentID,
		FilePath:     filePath,
		SubmittedAt:  submittedAt,
		Status:       models.StatusSubmitted,
		IsLate:       isLate,
		DaysLate:     daysLate,
	}

	// Validate
	if err := submission.Validate(); err != nil {
		// Clean up file on validation error
		s.storage.Delete(filePath)
		return nil, fmt.Errorf("validation failed: %w", err)
	}

	// Create in repository (will replace existing submission if any)
	if err := s.submissionRepo.Create(ctx, submission); err != nil {
		// Clean up file on error
		s.storage.Delete(filePath)
		return nil, fmt.Errorf("failed to create submission: %w", err)
	}

	// Publish event
	event := kafka.NewSubmissionCreatedEvent(submission.ID, assignmentID, studentID, isLate)
	if err := s.producer.PublishEvent(ctx, event); err != nil {
		fmt.Printf("Failed to publish submission.created event: %v\n", err)
	}

	return submission, nil
}

// GetSubmission retrieves a submission by ID
func (s *submissionService) GetSubmission(ctx context.Context, id string) (*models.Submission, error) {
	submission, err := s.submissionRepo.GetByID(ctx, id)
	if err != nil {
		return nil, fmt.Errorf("failed to get submission: %w", err)
	}

	return submission, nil
}

// ListSubmissions lists submissions for an assignment
func (s *submissionService) ListSubmissions(ctx context.Context, assignmentID, sortBy, order string) ([]*models.Submission, error) {
	submissions, err := s.submissionRepo.ListByAssignment(ctx, assignmentID, sortBy, order)
	if err != nil {
		return nil, fmt.Errorf("failed to list submissions: %w", err)
	}

	return submissions, nil
}

// ListStudentSubmissions lists submissions for a student in a course
func (s *submissionService) ListStudentSubmissions(ctx context.Context, studentID, courseID string) ([]*models.Submission, error) {
	submissions, err := s.submissionRepo.ListByStudent(ctx, studentID, courseID)
	if err != nil {
		return nil, fmt.Errorf("failed to list student submissions: %w", err)
	}

	return submissions, nil
}
