package jwt

import (
	"math"
	"regexp"
	"slate/services/user-auth-service/pkg/logger"
	"strings"
	"unicode"
)

// SecretValidator validates JWT secret strength
type SecretValidator struct {
	minLength        int
	requireMixedCase bool
	requireNumbers   bool
	requireSpecial   bool
	devMode          bool
	log              *logger.Logger
}

// NewSecretValidator creates a new secret validator
// devMode allows weaker secrets with warnings instead of errors
func NewSecretValidator(devMode bool) *SecretValidator {
	return &SecretValidator{
		minLength:        32,
		requireMixedCase: true,
		requireNumbers:   true,
		requireSpecial:   true,
		devMode:          devMode,
		log:              logger.NewLogger("info"),
	}
}

// ValidateSecret validates the JWT secret meets security requirements
// Returns error if validation fails in production mode
// Logs warnings in development mode
func (v *SecretValidator) ValidateSecret(secret string) error {
	// Check minimum length
	if len(secret) < v.minLength {
		if v.devMode {
			v.log.Warn().
				Int("min_length", v.minLength).
				Int("current_length", len(secret)).
				Msg("JWT secret is shorter than recommended")
			return nil
		}
		v.log.Error().
			Int("min_length", v.minLength).
			Int("current_length", len(secret)).
			Msg("JWT secret must meet minimum length requirement")
		return &ValidationError{Field: "jwt_secret", Message: "secret too short"}
	}

	// Check mixed case
	if v.requireMixedCase && !hasMixedCase(secret) {
		msg := "JWT secret must contain both uppercase and lowercase letters"
		if v.devMode {
			v.log.Warn().Msg(msg)
			return nil
		}
		v.log.Error().Msg(msg)
		return &ValidationError{Field: "jwt_secret", Message: "missing mixed case"}
	}

	// Check numbers
	if v.requireNumbers && !hasNumbers(secret) {
		msg := "JWT secret must contain numbers"
		if v.devMode {
			v.log.Warn().Msg(msg)
			return nil
		}
		v.log.Error().Msg(msg)
		return &ValidationError{Field: "jwt_secret", Message: "missing numbers"}
	}

	// Check special characters
	if v.requireSpecial && !hasSpecialChars(secret) {
		msg := "JWT secret must contain special characters"
		if v.devMode {
			v.log.Warn().Msg(msg)
			return nil
		}
		v.log.Error().Msg(msg)
		return &ValidationError{Field: "jwt_secret", Message: "missing special characters"}
	}

	// Calculate and log entropy (without logging the secret itself)
	entropy := calculateEntropy(secret)
	v.log.Info().Float64("entropy_bits", entropy).Msg("JWT secret entropy calculated")

	return nil
}

// ValidationError represents a validation error
type ValidationError struct {
	Field   string
	Message string
}

func (e *ValidationError) Error() string {
	return e.Field + ": " + e.Message
}

// hasMixedCase checks if string contains both uppercase and lowercase letters
func hasMixedCase(s string) bool {
	hasUpper := false
	hasLower := false

	for _, r := range s {
		if unicode.IsUpper(r) {
			hasUpper = true
		}
		if unicode.IsLower(r) {
			hasLower = true
		}
		if hasUpper && hasLower {
			return true
		}
	}

	return false
}

// hasNumbers checks if string contains numeric digits
func hasNumbers(s string) bool {
	return regexp.MustCompile(`\d`).MatchString(s)
}

// hasSpecialChars checks if string contains special characters
func hasSpecialChars(s string) bool {
	// Check for common special characters
	specialChars := "!@#$%^&*()_+-=[]{}|;:,.<>?/~`"
	return strings.ContainsAny(s, specialChars)
}

// calculateEntropy calculates Shannon entropy of the string
// Higher entropy indicates more randomness and better security
func calculateEntropy(s string) float64 {
	if len(s) == 0 {
		return 0
	}

	// Count frequency of each character
	freq := make(map[rune]int)
	for _, c := range s {
		freq[c]++
	}

	// Calculate Shannon entropy
	var entropy float64
	length := float64(len(s))

	for _, count := range freq {
		p := float64(count) / length
		if p > 0 {
			entropy -= p * math.Log2(p)
		}
	}

	// Return total entropy (entropy per character * length)
	return entropy * length
}
