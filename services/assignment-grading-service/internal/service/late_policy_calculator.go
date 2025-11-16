package service

import (
	"time"

	"slate/services/assignment-grading-service/internal/models"
)

// LatePolicyCalculator handles late submission penalty calculations
type LatePolicyCalculator struct{}

// NewLatePolicyCalculator creates a new late policy calculator
func NewLatePolicyCalculator() *LatePolicyCalculator {
	return &LatePolicyCalculator{}
}

// CalculatePenalty determines if a submission is late and calculates days late
func (c *LatePolicyCalculator) CalculatePenalty(submittedAt, dueDate time.Time, policy models.LatePolicy) (isLate bool, daysLate int) {
	// If submission is on or before due date, not late
	if submittedAt.Before(dueDate) || submittedAt.Equal(dueDate) {
		return false, 0
	}

	// No late submissions allowed
	if !policy.AllowsLateSubmissions() {
		return true, 0
	}

	// Calculate days late (using calendar days)
	duration := submittedAt.Sub(dueDate)
	daysLate = int(duration.Hours() / 24)

	// Round up if there's any partial day
	if duration.Hours() > float64(daysLate*24) {
		daysLate++
	}

	// Check if exceeds max late days
	if daysLate > policy.MaxLateDays {
		return true, daysLate
	}

	return true, daysLate
}

// ApplyPenalty calculates the adjusted score after applying late penalty
func (c *LatePolicyCalculator) ApplyPenalty(originalScore float64, daysLate int, policy models.LatePolicy) float64 {
	// No penalty if not late
	if daysLate == 0 {
		return originalScore
	}

	// No late submissions allowed or exceeds max late days
	if !policy.AllowsLateSubmissions() || daysLate > policy.MaxLateDays {
		return 0
	}

	// Calculate penalty
	totalPenaltyPercent := float64(daysLate * policy.PenaltyPercentPerDay)
	if totalPenaltyPercent > 100 {
		totalPenaltyPercent = 100
	}

	penaltyAmount := originalScore * (totalPenaltyPercent / 100.0)
	adjustedScore := originalScore - penaltyAmount

	// Ensure score doesn't go below 0
	if adjustedScore < 0 {
		adjustedScore = 0
	}

	return adjustedScore
}
