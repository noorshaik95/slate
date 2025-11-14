package models

import (
	"errors"
	"time"
)

// Submission status constants
const (
	StatusSubmitted = "submitted"
	StatusGraded    = "graded"
	StatusReturned  = "returned"
)

// Submission represents a student's assignment submission
type Submission struct {
	ID           string    `json:"id"`
	AssignmentID string    `json:"assignment_id"`
	StudentID    string    `json:"student_id"`
	FilePath     string    `json:"file_path"`
	SubmittedAt  time.Time `json:"submitted_at"`
	Status       string    `json:"status"`
	IsLate       bool      `json:"is_late"`
	DaysLate     int       `json:"days_late"`
	CreatedAt    time.Time `json:"created_at"`
	UpdatedAt    time.Time `json:"updated_at"`
}

// Validate checks if the submission has valid data
func (s *Submission) Validate() error {
	if s.AssignmentID == "" {
		return errors.New("assignment_id is required")
	}

	if s.StudentID == "" {
		return errors.New("student_id is required")
	}

	if s.FilePath == "" {
		return errors.New("file_path is required")
	}

	if s.SubmittedAt.IsZero() {
		return errors.New("submitted_at is required")
	}

	if !isValidStatus(s.Status) {
		return errors.New("invalid status")
	}

	if s.DaysLate < 0 {
		return errors.New("days_late must be non-negative")
	}

	return nil
}

// isValidStatus checks if the status is valid
func isValidStatus(status string) bool {
	switch status {
	case StatusSubmitted, StatusGraded, StatusReturned:
		return true
	default:
		return false
	}
}

// UpdateStatus updates the submission status
func (s *Submission) UpdateStatus(newStatus string) error {
	if !isValidStatus(newStatus) {
		return errors.New("invalid status")
	}
	s.Status = newStatus
	s.UpdatedAt = time.Now()
	return nil
}
