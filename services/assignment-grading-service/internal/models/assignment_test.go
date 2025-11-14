package models

import (
	"strings"
	"testing"
	"time"
)

func TestAssignment_Validate(t *testing.T) {
	validDueDate := time.Now().Add(24 * time.Hour)

	tests := []struct {
		name        string
		assignment  Assignment
		expectError bool
		errorMsg    string
	}{
		{
			name: "valid assignment",
			assignment: Assignment{
				CourseID:    "COURSE-001",
				Title:       "Assignment 1",
				Description: "Test assignment",
				MaxPoints:   100.0,
				DueDate:     validDueDate,
				LatePolicy: LatePolicy{
					PenaltyPercentPerDay: 10,
					MaxLateDays:          3,
				},
			},
			expectError: false,
		},
		{
			name: "missing course_id",
			assignment: Assignment{
				Title:     "Assignment 1",
				MaxPoints: 100.0,
				DueDate:   validDueDate,
			},
			expectError: true,
			errorMsg:    "course_id is required",
		},
		{
			name: "missing title",
			assignment: Assignment{
				CourseID:  "COURSE-001",
				MaxPoints: 100.0,
				DueDate:   validDueDate,
			},
			expectError: true,
			errorMsg:    "title is required",
		},
		{
			name: "title too long",
			assignment: Assignment{
				CourseID:  "COURSE-001",
				Title:     strings.Repeat("A", 501),
				MaxPoints: 100.0,
				DueDate:   validDueDate,
			},
			expectError: true,
			errorMsg:    "title must be 500 characters or less",
		},
		{
			name: "zero max_points",
			assignment: Assignment{
				CourseID: "COURSE-001",
				Title:    "Assignment 1",
				DueDate:  validDueDate,
			},
			expectError: true,
			errorMsg:    "max_points must be greater than 0",
		},
		{
			name: "negative max_points",
			assignment: Assignment{
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: -10.0,
				DueDate:   validDueDate,
			},
			expectError: true,
			errorMsg:    "max_points must be greater than 0",
		},
		{
			name: "zero due_date",
			assignment: Assignment{
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: 100.0,
			},
			expectError: true,
			errorMsg:    "due_date is required",
		},
		{
			name: "invalid late policy - negative penalty",
			assignment: Assignment{
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: 100.0,
				DueDate:   validDueDate,
				LatePolicy: LatePolicy{
					PenaltyPercentPerDay: -5,
					MaxLateDays:          3,
				},
			},
			expectError: true,
			errorMsg:    "penalty_percent_per_day must be between 0 and 100",
		},
		{
			name: "invalid late policy - penalty over 100",
			assignment: Assignment{
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: 100.0,
				DueDate:   validDueDate,
				LatePolicy: LatePolicy{
					PenaltyPercentPerDay: 150,
					MaxLateDays:          3,
				},
			},
			expectError: true,
			errorMsg:    "penalty_percent_per_day must be between 0 and 100",
		},
		{
			name: "invalid late policy - negative max late days",
			assignment: Assignment{
				CourseID:  "COURSE-001",
				Title:     "Assignment 1",
				MaxPoints: 100.0,
				DueDate:   validDueDate,
				LatePolicy: LatePolicy{
					PenaltyPercentPerDay: 10,
					MaxLateDays:          -1,
				},
			},
			expectError: true,
			errorMsg:    "max_late_days must be non-negative",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.assignment.Validate()
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

func TestLatePolicy_Validate(t *testing.T) {
	tests := []struct {
		name        string
		policy      LatePolicy
		expectError bool
		errorMsg    string
	}{
		{
			name: "valid policy",
			policy: LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          5,
			},
			expectError: false,
		},
		{
			name: "no late submissions allowed",
			policy: LatePolicy{
				PenaltyPercentPerDay: 0,
				MaxLateDays:          0,
			},
			expectError: false,
		},
		{
			name: "100% penalty per day",
			policy: LatePolicy{
				PenaltyPercentPerDay: 100,
				MaxLateDays:          1,
			},
			expectError: false,
		},
		{
			name: "negative penalty",
			policy: LatePolicy{
				PenaltyPercentPerDay: -10,
				MaxLateDays:          3,
			},
			expectError: true,
			errorMsg:    "penalty_percent_per_day must be between 0 and 100",
		},
		{
			name: "penalty over 100",
			policy: LatePolicy{
				PenaltyPercentPerDay: 101,
				MaxLateDays:          3,
			},
			expectError: true,
			errorMsg:    "penalty_percent_per_day must be between 0 and 100",
		},
		{
			name: "negative max late days",
			policy: LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          -1,
			},
			expectError: true,
			errorMsg:    "max_late_days must be non-negative",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.policy.Validate()
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

func TestLatePolicy_AllowsLateSubmissions(t *testing.T) {
	tests := []struct {
		name     string
		policy   LatePolicy
		expected bool
	}{
		{
			name: "allows late submissions",
			policy: LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expected: true,
		},
		{
			name: "no late submissions allowed",
			policy: LatePolicy{
				PenaltyPercentPerDay: 0,
				MaxLateDays:          0,
			},
			expected: false,
		},
		{
			name: "one day late allowed",
			policy: LatePolicy{
				PenaltyPercentPerDay: 50,
				MaxLateDays:          1,
			},
			expected: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.policy.AllowsLateSubmissions()
			if result != tt.expected {
				t.Errorf("expected %v but got %v", tt.expected, result)
			}
		})
	}
}
