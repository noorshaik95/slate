package models

import (
	"testing"
	"time"
)

func TestGrade_Validate(t *testing.T) {
	now := time.Now()

	tests := []struct {
		name        string
		grade       Grade
		expectError bool
		errorMsg    string
	}{
		{
			name: "valid grade",
			grade: Grade{
				SubmissionID:  "SUB-001",
				StudentID:     "STUDENT-001",
				AssignmentID:  "ASSIGN-001",
				Score:         85.0,
				AdjustedScore: 85.0,
				Status:        GradeStatusDraft,
				GradedBy:      "INSTRUCTOR-001",
				GradedAt:      &now,
			},
			expectError: false,
		},
		{
			name: "missing submission_id",
			grade: Grade{
				StudentID:     "STUDENT-001",
				AssignmentID:  "ASSIGN-001",
				Score:         85.0,
				AdjustedScore: 85.0,
				Status:        GradeStatusDraft,
				GradedBy:      "INSTRUCTOR-001",
			},
			expectError: true,
			errorMsg:    "submission_id is required",
		},
		{
			name: "missing student_id",
			grade: Grade{
				SubmissionID:  "SUB-001",
				AssignmentID:  "ASSIGN-001",
				Score:         85.0,
				AdjustedScore: 85.0,
				Status:        GradeStatusDraft,
				GradedBy:      "INSTRUCTOR-001",
			},
			expectError: true,
			errorMsg:    "student_id is required",
		},
		{
			name: "missing assignment_id",
			grade: Grade{
				SubmissionID:  "SUB-001",
				StudentID:     "STUDENT-001",
				Score:         85.0,
				AdjustedScore: 85.0,
				Status:        GradeStatusDraft,
				GradedBy:      "INSTRUCTOR-001",
			},
			expectError: true,
			errorMsg:    "assignment_id is required",
		},
		{
			name: "negative score",
			grade: Grade{
				SubmissionID:  "SUB-001",
				StudentID:     "STUDENT-001",
				AssignmentID:  "ASSIGN-001",
				Score:         -10.0,
				AdjustedScore: -10.0,
				Status:        GradeStatusDraft,
				GradedBy:      "INSTRUCTOR-001",
			},
			expectError: true,
			errorMsg:    "score must be non-negative",
		},
		{
			name: "negative adjusted_score",
			grade: Grade{
				SubmissionID:  "SUB-001",
				StudentID:     "STUDENT-001",
				AssignmentID:  "ASSIGN-001",
				Score:         85.0,
				AdjustedScore: -5.0,
				Status:        GradeStatusDraft,
				GradedBy:      "INSTRUCTOR-001",
			},
			expectError: true,
			errorMsg:    "adjusted_score must be non-negative",
		},
		{
			name: "missing graded_by",
			grade: Grade{
				SubmissionID:  "SUB-001",
				StudentID:     "STUDENT-001",
				AssignmentID:  "ASSIGN-001",
				Score:         85.0,
				AdjustedScore: 85.0,
				Status:        GradeStatusDraft,
			},
			expectError: true,
			errorMsg:    "graded_by is required",
		},
		{
			name: "invalid status",
			grade: Grade{
				SubmissionID:  "SUB-001",
				StudentID:     "STUDENT-001",
				AssignmentID:  "ASSIGN-001",
				Score:         85.0,
				AdjustedScore: 85.0,
				Status:        "invalid_status",
				GradedBy:      "INSTRUCTOR-001",
			},
			expectError: true,
			errorMsg:    "invalid status",
		},
		{
			name: "published grade",
			grade: Grade{
				SubmissionID:  "SUB-001",
				StudentID:     "STUDENT-001",
				AssignmentID:  "ASSIGN-001",
				Score:         90.0,
				AdjustedScore: 90.0,
				Status:        GradeStatusPublished,
				GradedBy:      "INSTRUCTOR-001",
				GradedAt:      &now,
				PublishedAt:   &now,
			},
			expectError: false,
		},
		{
			name: "zero score is valid",
			grade: Grade{
				SubmissionID:  "SUB-001",
				StudentID:     "STUDENT-001",
				AssignmentID:  "ASSIGN-001",
				Score:         0.0,
				AdjustedScore: 0.0,
				Status:        GradeStatusDraft,
				GradedBy:      "INSTRUCTOR-001",
			},
			expectError: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.grade.Validate()
			if tt.expectError {
				if err == nil {
					t.Errorf("expected error but got nil")
				} else if err.Error() != tt.errorMsg {
					t.Errorf("expected error %q but got %q", tt.errorMsg, err.Error())
				}
			} else {
				if err != nil {
					t.Errorf("expected no error but got: %v", err)
				}
			}
		})
	}
}

func TestGrade_ValidateScore(t *testing.T) {
	tests := []struct {
		name        string
		grade       Grade
		maxPoints   float64
		expectError bool
		errorMsg    string
	}{
		{
			name: "score within range",
			grade: Grade{
				Score:         85.0,
				AdjustedScore: 85.0,
			},
			maxPoints:   100.0,
			expectError: false,
		},
		{
			name: "score equals max points",
			grade: Grade{
				Score:         100.0,
				AdjustedScore: 100.0,
			},
			maxPoints:   100.0,
			expectError: false,
		},
		{
			name: "score exceeds max points",
			grade: Grade{
				Score:         110.0,
				AdjustedScore: 110.0,
			},
			maxPoints:   100.0,
			expectError: true,
			errorMsg:    "score exceeds max_points",
		},
		{
			name: "adjusted score exceeds max points",
			grade: Grade{
				Score:         90.0,
				AdjustedScore: 110.0,
			},
			maxPoints:   100.0,
			expectError: true,
			errorMsg:    "adjusted_score exceeds max_points",
		},
		{
			name: "zero score with max points",
			grade: Grade{
				Score:         0.0,
				AdjustedScore: 0.0,
			},
			maxPoints:   100.0,
			expectError: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.grade.ValidateScore(tt.maxPoints)
			if tt.expectError {
				if err == nil {
					t.Errorf("expected error but got nil")
				} else if err.Error() != tt.errorMsg {
					t.Errorf("expected error %q but got %q", tt.errorMsg, err.Error())
				}
			} else {
				if err != nil {
					t.Errorf("expected no error but got: %v", err)
				}
			}
		})
	}
}

func TestGrade_Publish(t *testing.T) {
	now := time.Now()

	t.Run("publish draft grade", func(t *testing.T) {
		grade := Grade{
			Status:    GradeStatusDraft,
			UpdatedAt: now,
		}

		err := grade.Publish()
		if err != nil {
			t.Errorf("expected no error but got: %v", err)
		}
		if grade.Status != GradeStatusPublished {
			t.Errorf("expected status %q but got %q", GradeStatusPublished, grade.Status)
		}
		if grade.PublishedAt == nil {
			t.Errorf("expected PublishedAt to be set")
		}
		if !grade.UpdatedAt.After(now) {
			t.Errorf("expected UpdatedAt to be updated")
		}
	})

	t.Run("publish already published grade", func(t *testing.T) {
		grade := Grade{
			Status:      GradeStatusPublished,
			PublishedAt: &now,
		}

		err := grade.Publish()
		if err == nil {
			t.Errorf("expected error but got nil")
		}
		if err.Error() != "grade is already published" {
			t.Errorf("unexpected error message: %v", err)
		}
	})
}

func TestGrade_IsDraft(t *testing.T) {
	tests := []struct {
		name     string
		status   string
		expected bool
	}{
		{"draft status", GradeStatusDraft, true},
		{"published status", GradeStatusPublished, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			grade := Grade{Status: tt.status}
			result := grade.IsDraft()
			if result != tt.expected {
				t.Errorf("IsDraft() = %v, want %v", result, tt.expected)
			}
		})
	}
}

func TestGrade_IsPublished(t *testing.T) {
	tests := []struct {
		name     string
		status   string
		expected bool
	}{
		{"draft status", GradeStatusDraft, false},
		{"published status", GradeStatusPublished, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			grade := Grade{Status: tt.status}
			result := grade.IsPublished()
			if result != tt.expected {
				t.Errorf("IsPublished() = %v, want %v", result, tt.expected)
			}
		})
	}
}

func TestIsValidGradeStatus(t *testing.T) {
	tests := []struct {
		status   string
		expected bool
	}{
		{GradeStatusDraft, true},
		{GradeStatusPublished, true},
		{"invalid", false},
		{"", false},
		{"DRAFT", false}, // Case sensitive
	}

	for _, tt := range tests {
		t.Run(tt.status, func(t *testing.T) {
			result := isValidGradeStatus(tt.status)
			if result != tt.expected {
				t.Errorf("isValidGradeStatus(%q) = %v, want %v", tt.status, result, tt.expected)
			}
		})
	}
}
