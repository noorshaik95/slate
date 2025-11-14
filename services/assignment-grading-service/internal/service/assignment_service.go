package service

import (
	"context"
	"fmt"
	"time"

	"slate/services/assignment-grading-service/internal/models"
	"slate/services/assignment-grading-service/internal/repository"
	"slate/services/assignment-grading-service/pkg/kafka"
)

type assignmentService struct {
	repo     repository.AssignmentRepository
	producer *kafka.Producer
}

// NewAssignmentService creates a new assignment service
func NewAssignmentService(repo repository.AssignmentRepository, producer *kafka.Producer) AssignmentService {
	return &assignmentService{
		repo:     repo,
		producer: producer,
	}
}

// CreateAssignment creates a new assignment
func (s *assignmentService) CreateAssignment(ctx context.Context, courseID, title, description string, maxPoints float64, dueDate time.Time, latePolicy models.LatePolicy) (*models.Assignment, error) {
	// Create assignment model
	assignment := &models.Assignment{
		CourseID:    courseID,
		Title:       title,
		Description: description,
		MaxPoints:   maxPoints,
		DueDate:     dueDate,
		LatePolicy:  latePolicy,
	}

	// Validate
	if err := assignment.Validate(); err != nil {
		return nil, fmt.Errorf("validation failed: %w", err)
	}

	// Create in repository
	if err := s.repo.Create(ctx, assignment); err != nil {
		return nil, fmt.Errorf("failed to create assignment: %w", err)
	}

	// Publish event
	event := kafka.NewAssignmentCreatedEvent(assignment.ID, assignment.CourseID, assignment.Title)
	if err := s.producer.PublishEvent(ctx, event); err != nil {
		// Log error but don't fail the operation
		fmt.Printf("Failed to publish assignment.created event: %v\n", err)
	}

	return assignment, nil
}

// GetAssignment retrieves an assignment by ID
func (s *assignmentService) GetAssignment(ctx context.Context, id string) (*models.Assignment, error) {
	assignment, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, fmt.Errorf("failed to get assignment: %w", err)
	}

	return assignment, nil
}

// UpdateAssignment updates an existing assignment
func (s *assignmentService) UpdateAssignment(ctx context.Context, id, title, description string, maxPoints float64, dueDate time.Time, latePolicy models.LatePolicy) (*models.Assignment, error) {
	// Check if assignment has submissions
	hasSubmissions, err := s.repo.HasSubmissions(ctx, id)
	if err != nil {
		return nil, fmt.Errorf("failed to check submissions: %w", err)
	}

	if hasSubmissions {
		return nil, fmt.Errorf("cannot update assignment with existing submissions")
	}

	// Get existing assignment
	assignment, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, fmt.Errorf("failed to get assignment: %w", err)
	}

	// Update fields
	assignment.Title = title
	assignment.Description = description
	assignment.MaxPoints = maxPoints
	assignment.DueDate = dueDate
	assignment.LatePolicy = latePolicy

	// Validate
	if err := assignment.Validate(); err != nil {
		return nil, fmt.Errorf("validation failed: %w", err)
	}

	// Update in repository
	if err := s.repo.Update(ctx, assignment); err != nil {
		return nil, fmt.Errorf("failed to update assignment: %w", err)
	}

	// Publish event
	event := kafka.NewAssignmentUpdatedEvent(assignment.ID, assignment.CourseID)
	if err := s.producer.PublishEvent(ctx, event); err != nil {
		fmt.Printf("Failed to publish assignment.updated event: %v\n", err)
	}

	return assignment, nil
}

// DeleteAssignment deletes an assignment
func (s *assignmentService) DeleteAssignment(ctx context.Context, id string) error {
	// Get assignment to get course ID for event
	assignment, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return fmt.Errorf("failed to get assignment: %w", err)
	}

	// Delete from repository (will cascade delete submissions and grades)
	if err := s.repo.Delete(ctx, id); err != nil {
		return fmt.Errorf("failed to delete assignment: %w", err)
	}

	// Publish event
	event := kafka.NewAssignmentDeletedEvent(assignment.ID, assignment.CourseID)
	if err := s.producer.PublishEvent(ctx, event); err != nil {
		fmt.Printf("Failed to publish assignment.deleted event: %v\n", err)
	}

	return nil
}

// ListAssignments lists assignments for a course with pagination
func (s *assignmentService) ListAssignments(ctx context.Context, courseID string, page, pageSize int) ([]*models.Assignment, int, error) {
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	assignments, total, err := s.repo.ListByCourse(ctx, courseID, page, pageSize)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to list assignments: %w", err)
	}

	return assignments, total, nil
}
