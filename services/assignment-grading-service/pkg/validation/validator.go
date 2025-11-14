package validation

import (
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

// Validator provides input validation for user data
type Validator struct {
	emailRegex *regexp.Regexp
	phoneRegex *regexp.Regexp
}

// ValidationError represents a validation error for a specific field
type ValidationError struct {
	Field   string
	Message string
	Code    string
}

// Error implements the error interface
func (e *ValidationError) Error() string {
	return fmt.Sprintf("%s: %s", e.Field, e.Message)
}

// NewValidator creates a new Validator instance with compiled regex patterns
func NewValidator() *Validator {
	// RFC 5322 compliant email regex (simplified but practical)
	emailRegex := regexp.MustCompile(`^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$`)

	// E.164 international phone format: +[country code][number]
	// Allows 7-15 digits after the + sign
	phoneRegex := regexp.MustCompile(`^\+[1-9]\d{6,14}$`)

	return &Validator{
		emailRegex: emailRegex,
		phoneRegex: phoneRegex,
	}
}

// ValidatePassword validates password complexity requirements
// Requirements:
// - Minimum 8 characters
// - At least one uppercase letter
// - At least one lowercase letter
// - At least one digit
// - At least one special character
func (v *Validator) ValidatePassword(password string) error {
	if len(password) < 8 {
		return &ValidationError{
			Field:   "password",
			Message: "password must be at least 8 characters long",
			Code:    "PASSWORD_TOO_SHORT",
		}
	}

	var (
		hasUpper   bool
		hasLower   bool
		hasDigit   bool
		hasSpecial bool
	)

	for _, char := range password {
		switch {
		case unicode.IsUpper(char):
			hasUpper = true
		case unicode.IsLower(char):
			hasLower = true
		case unicode.IsDigit(char):
			hasDigit = true
		case unicode.IsPunct(char) || unicode.IsSymbol(char):
			hasSpecial = true
		}
	}

	if !hasUpper {
		return &ValidationError{
			Field:   "password",
			Message: "password must contain at least one uppercase letter",
			Code:    "PASSWORD_NO_UPPERCASE",
		}
	}

	if !hasLower {
		return &ValidationError{
			Field:   "password",
			Message: "password must contain at least one lowercase letter",
			Code:    "PASSWORD_NO_LOWERCASE",
		}
	}

	if !hasDigit {
		return &ValidationError{
			Field:   "password",
			Message: "password must contain at least one digit",
			Code:    "PASSWORD_NO_DIGIT",
		}
	}

	if !hasSpecial {
		return &ValidationError{
			Field:   "password",
			Message: "password must contain at least one special character",
			Code:    "PASSWORD_NO_SPECIAL",
		}
	}

	return nil
}

// ValidateEmail validates email format using RFC 5322 compliant regex
func (v *Validator) ValidateEmail(email string) error {
	if email == "" {
		return &ValidationError{
			Field:   "email",
			Message: "email is required",
			Code:    "EMAIL_REQUIRED",
		}
	}

	if len(email) > 254 {
		return &ValidationError{
			Field:   "email",
			Message: "email is too long (maximum 254 characters)",
			Code:    "EMAIL_TOO_LONG",
		}
	}

	if !v.emailRegex.MatchString(email) {
		return &ValidationError{
			Field:   "email",
			Message: "email format is invalid",
			Code:    "EMAIL_INVALID_FORMAT",
		}
	}

	return nil
}

// ValidatePhone validates phone number in E.164 international format
// Format: +[country code][number] (e.g., +14155552671)
func (v *Validator) ValidatePhone(phone string) error {
	if phone == "" {
		// Phone is optional, so empty is valid
		return nil
	}

	if !v.phoneRegex.MatchString(phone) {
		return &ValidationError{
			Field:   "phone",
			Message: "phone number must be in E.164 format (e.g., +14155552671)",
			Code:    "PHONE_INVALID_FORMAT",
		}
	}

	return nil
}

// SanitizeName sanitizes first name or last name by removing HTML tags and special characters
// Allows: letters, spaces, hyphens, apostrophes
// Returns sanitized name and error if name is invalid
func (v *Validator) SanitizeName(name string) (string, error) {
	if name == "" {
		return "", &ValidationError{
			Field:   "name",
			Message: "name is required",
			Code:    "NAME_REQUIRED",
		}
	}

	// Remove leading/trailing whitespace
	name = strings.TrimSpace(name)

	// Remove HTML tags using regex
	htmlTagRegex := regexp.MustCompile(`<[^>]*>`)
	name = htmlTagRegex.ReplaceAllString(name, "")

	// Build sanitized name allowing only letters, spaces, hyphens, and apostrophes
	var sanitized strings.Builder
	for _, char := range name {
		if unicode.IsLetter(char) || char == ' ' || char == '-' || char == '\'' {
			sanitized.WriteRune(char)
		}
	}

	result := strings.TrimSpace(sanitized.String())

	if result == "" {
		return "", &ValidationError{
			Field:   "name",
			Message: "name contains no valid characters",
			Code:    "NAME_INVALID",
		}
	}

	if len(result) > 100 {
		return "", &ValidationError{
			Field:   "name",
			Message: "name is too long (maximum 100 characters)",
			Code:    "NAME_TOO_LONG",
		}
	}

	return result, nil
}
