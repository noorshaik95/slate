package service

import (
	"testing"
	"time"

	"slate/services/assignment-grading-service/internal/models"
)

func TestLatePolicyCalculator_CalculatePenalty(t *testing.T) {
	calc := NewLatePolicyCalculator()
	baseTime := time.Date(2024, 1, 15, 23, 59, 59, 0, time.UTC)

	tests := []struct {
		name          string
		submittedAt   time.Time
		dueDate       time.Time
		policy        models.LatePolicy
		expectLate    bool
		expectedDays  int
	}{
		{
			name:        "on time submission - exact due date",
			submittedAt: baseTime,
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expectLate:   false,
			expectedDays: 0,
		},
		{
			name:        "on time submission - before due date",
			submittedAt: baseTime.Add(-2 * time.Hour),
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expectLate:   false,
			expectedDays: 0,
		},
		{
			name:        "1 day late",
			submittedAt: baseTime.Add(25 * time.Hour),
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expectLate:   true,
			expectedDays: 2, // Rounds up partial days
		},
		{
			name:        "exactly 1 day late",
			submittedAt: baseTime.Add(24 * time.Hour),
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expectLate:   true,
			expectedDays: 1,
		},
		{
			name:        "3 days late",
			submittedAt: baseTime.Add(72 * time.Hour),
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expectLate:   true,
			expectedDays: 3,
		},
		{
			name:        "exceeds max late days",
			submittedAt: baseTime.Add(96 * time.Hour),
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expectLate:   true,
			expectedDays: 4,
		},
		{
			name:        "no late submissions allowed - late submission",
			submittedAt: baseTime.Add(1 * time.Hour),
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 0,
				MaxLateDays:          0,
			},
			expectLate:   true,
			expectedDays: 0,
		},
		{
			name:        "no late submissions allowed - on time",
			submittedAt: baseTime,
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 0,
				MaxLateDays:          0,
			},
			expectLate:   false,
			expectedDays: 0,
		},
		{
			name:        "partial day late (1 hour)",
			submittedAt: baseTime.Add(1 * time.Hour),
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          5,
			},
			expectLate:   true,
			expectedDays: 1, // Rounds up to 1 day
		},
		{
			name:        "many days late",
			submittedAt: baseTime.Add(240 * time.Hour), // 10 days
			dueDate:     baseTime,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 5,
				MaxLateDays:          14,
			},
			expectLate:   true,
			expectedDays: 10,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			isLate, daysLate := calc.CalculatePenalty(tt.submittedAt, tt.dueDate, tt.policy)

			if isLate != tt.expectLate {
				t.Errorf("expected isLate=%v but got %v", tt.expectLate, isLate)
			}

			if daysLate != tt.expectedDays {
				t.Errorf("expected daysLate=%d but got %d", tt.expectedDays, daysLate)
			}
		})
	}
}

func TestLatePolicyCalculator_ApplyPenalty(t *testing.T) {
	calc := NewLatePolicyCalculator()

	tests := []struct {
		name          string
		originalScore float64
		daysLate      int
		policy        models.LatePolicy
		expected      float64
	}{
		{
			name:          "no penalty - on time",
			originalScore: 100.0,
			daysLate:      0,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expected: 100.0,
		},
		{
			name:          "10% penalty per day - 1 day late",
			originalScore: 100.0,
			daysLate:      1,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expected: 90.0,
		},
		{
			name:          "10% penalty per day - 2 days late",
			originalScore: 100.0,
			daysLate:      2,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expected: 80.0,
		},
		{
			name:          "10% penalty per day - 3 days late",
			originalScore: 100.0,
			daysLate:      3,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expected: 70.0,
		},
		{
			name:          "exceeds max late days - zero score",
			originalScore: 100.0,
			daysLate:      4,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expected: 0.0,
		},
		{
			name:          "no late submissions allowed - zero score",
			originalScore: 100.0,
			daysLate:      1,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 0,
				MaxLateDays:          0,
			},
			expected: 0.0,
		},
		{
			name:          "50% penalty per day - 1 day late",
			originalScore: 90.0,
			daysLate:      1,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 50,
				MaxLateDays:          2,
			},
			expected: 45.0,
		},
		{
			name:          "100% penalty per day - instant zero",
			originalScore: 85.0,
			daysLate:      1,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 100,
				MaxLateDays:          1,
			},
			expected: 0.0,
		},
		{
			name:          "penalty exceeds 100% - capped at zero",
			originalScore: 75.0,
			daysLate:      15,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          20,
			},
			expected: 0.0, // 150% penalty capped at 100%
		},
		{
			name:          "small penalty on lower score",
			originalScore: 50.0,
			daysLate:      1,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 5,
				MaxLateDays:          5,
			},
			expected: 47.5,
		},
		{
			name:          "fractional result",
			originalScore: 87.5,
			daysLate:      1,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 15,
				MaxLateDays:          3,
			},
			expected: 74.375,
		},
		{
			name:          "zero original score",
			originalScore: 0.0,
			daysLate:      2,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 10,
				MaxLateDays:          3,
			},
			expected: 0.0,
		},
		{
			name:          "exactly at max late days",
			originalScore: 100.0,
			daysLate:      3,
			policy: models.LatePolicy{
				PenaltyPercentPerDay: 20,
				MaxLateDays:          3,
			},
			expected: 40.0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := calc.ApplyPenalty(tt.originalScore, tt.daysLate, tt.policy)

			// Use a small epsilon for float comparison
			epsilon := 0.0001
			if result < tt.expected-epsilon || result > tt.expected+epsilon {
				t.Errorf("expected %.4f but got %.4f", tt.expected, result)
			}
		})
	}
}

func TestLatePolicyCalculator_CompleteWorkflow(t *testing.T) {
	calc := NewLatePolicyCalculator()
	dueDate := time.Date(2024, 1, 15, 23, 59, 59, 0, time.UTC)

	t.Run("complete workflow - 2 days late with 10% penalty", func(t *testing.T) {
		policy := models.LatePolicy{
			PenaltyPercentPerDay: 10,
			MaxLateDays:          5,
		}

		submittedAt := dueDate.Add(48 * time.Hour) // 2 days late
		originalScore := 95.0

		// Calculate if late and days late
		isLate, daysLate := calc.CalculatePenalty(submittedAt, dueDate, policy)

		if !isLate {
			t.Errorf("expected submission to be late")
		}

		if daysLate != 2 {
			t.Errorf("expected 2 days late but got %d", daysLate)
		}

		// Apply penalty
		adjustedScore := calc.ApplyPenalty(originalScore, daysLate, policy)
		expected := 76.0 // 95 - (95 * 0.20) = 76

		epsilon := 0.0001
		if adjustedScore < expected-epsilon || adjustedScore > expected+epsilon {
			t.Errorf("expected adjusted score %.2f but got %.2f", expected, adjustedScore)
		}
	})

	t.Run("complete workflow - on time submission", func(t *testing.T) {
		policy := models.LatePolicy{
			PenaltyPercentPerDay: 10,
			MaxLateDays:          3,
		}

		submittedAt := dueDate.Add(-1 * time.Hour) // 1 hour early
		originalScore := 88.0

		isLate, daysLate := calc.CalculatePenalty(submittedAt, dueDate, policy)

		if isLate {
			t.Errorf("expected submission to be on time")
		}

		if daysLate != 0 {
			t.Errorf("expected 0 days late but got %d", daysLate)
		}

		adjustedScore := calc.ApplyPenalty(originalScore, daysLate, policy)

		if adjustedScore != originalScore {
			t.Errorf("expected no penalty, score should remain %.2f but got %.2f", originalScore, adjustedScore)
		}
	})
}
