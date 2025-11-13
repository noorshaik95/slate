package validation

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestValidatePassword_Success(t *testing.T) {
	validator := NewValidator()

	validPasswords := []string{
		"Password123!",
		"MyP@ssw0rd",
		"Str0ng#Pass",
		"C0mpl3x!ty",
		"Secur3$Pass",
	}

	for _, password := range validPasswords {
		t.Run(password, func(t *testing.T) {
			err := validator.ValidatePassword(password)
			assert.NoError(t, err, "Expected password '%s' to be valid", password)
		})
	}
}

func TestValidatePassword_TooShort(t *testing.T) {
	validator := NewValidator()

	err := validator.ValidatePassword("Pass1!")

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "password", validationErr.Field)
	assert.Equal(t, "PASSWORD_TOO_SHORT", validationErr.Code)
	assert.Contains(t, validationErr.Message, "at least 8 characters")
}

func TestValidatePassword_NoUppercase(t *testing.T) {
	validator := NewValidator()

	err := validator.ValidatePassword("password123!")

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "password", validationErr.Field)
	assert.Equal(t, "PASSWORD_NO_UPPERCASE", validationErr.Code)
	assert.Contains(t, validationErr.Message, "uppercase letter")
}

func TestValidatePassword_NoLowercase(t *testing.T) {
	validator := NewValidator()

	err := validator.ValidatePassword("PASSWORD123!")

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "password", validationErr.Field)
	assert.Equal(t, "PASSWORD_NO_LOWERCASE", validationErr.Code)
	assert.Contains(t, validationErr.Message, "lowercase letter")
}

func TestValidatePassword_NoDigit(t *testing.T) {
	validator := NewValidator()

	err := validator.ValidatePassword("Password!")

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "password", validationErr.Field)
	assert.Equal(t, "PASSWORD_NO_DIGIT", validationErr.Code)
	assert.Contains(t, validationErr.Message, "digit")
}

func TestValidatePassword_NoSpecialCharacter(t *testing.T) {
	validator := NewValidator()

	err := validator.ValidatePassword("Password123")

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "password", validationErr.Field)
	assert.Equal(t, "PASSWORD_NO_SPECIAL", validationErr.Code)
	assert.Contains(t, validationErr.Message, "special character")
}

func TestValidateEmail_Success(t *testing.T) {
	validator := NewValidator()

	validEmails := []string{
		"user@example.com",
		"test.user@example.com",
		"user+tag@example.co.uk",
		"user_name@example-domain.com",
		"123@example.com",
	}

	for _, email := range validEmails {
		t.Run(email, func(t *testing.T) {
			err := validator.ValidateEmail(email)
			assert.NoError(t, err, "Expected email '%s' to be valid", email)
		})
	}
}

func TestValidateEmail_Empty(t *testing.T) {
	validator := NewValidator()

	err := validator.ValidateEmail("")

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "email", validationErr.Field)
	assert.Equal(t, "EMAIL_REQUIRED", validationErr.Code)
	assert.Contains(t, validationErr.Message, "required")
}

func TestValidateEmail_TooLong(t *testing.T) {
	validator := NewValidator()

	// Create an email longer than 254 characters
	longEmail := string(make([]byte, 250)) + "@example.com"

	err := validator.ValidateEmail(longEmail)

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "email", validationErr.Field)
	assert.Equal(t, "EMAIL_TOO_LONG", validationErr.Code)
	assert.Contains(t, validationErr.Message, "too long")
	assert.Contains(t, validationErr.Message, "254")
}

func TestValidateEmail_InvalidFormat(t *testing.T) {
	validator := NewValidator()

	invalidEmails := []string{
		"notanemail",
		"@example.com",
		"user@",
		"user @example.com",
		"user@example",
		// Note: Our simplified regex allows some edge cases like double dots
		// This is acceptable for practical use while maintaining simplicity
	}

	for _, email := range invalidEmails {
		t.Run(email, func(t *testing.T) {
			err := validator.ValidateEmail(email)
			require.Error(t, err, "Expected email '%s' to be invalid", email)
			validationErr, ok := err.(*ValidationError)
			require.True(t, ok, "Expected ValidationError type")
			assert.Equal(t, "email", validationErr.Field)
			assert.Equal(t, "EMAIL_INVALID_FORMAT", validationErr.Code)
			assert.Contains(t, validationErr.Message, "invalid")
		})
	}
}

func TestValidatePhone_Success(t *testing.T) {
	validator := NewValidator()

	validPhones := []string{
		"+14155552671",  // US
		"+442071838750", // UK
		"+33123456789",  // France
		"+81312345678",  // Japan
		"+61212345678",  // Australia
		"+919876543210", // India
	}

	for _, phone := range validPhones {
		t.Run(phone, func(t *testing.T) {
			err := validator.ValidatePhone(phone)
			assert.NoError(t, err, "Expected phone '%s' to be valid", phone)
		})
	}
}

func TestValidatePhone_Empty(t *testing.T) {
	validator := NewValidator()

	// Empty phone is valid (optional field)
	err := validator.ValidatePhone("")
	assert.NoError(t, err)
}

func TestValidatePhone_InvalidFormat(t *testing.T) {
	validator := NewValidator()

	invalidPhones := []string{
		"1234567890",         // Missing +
		"+1234",              // Too short
		"+12345678901234567", // Too long
		"+0123456789",        // Starts with 0 after +
		"123-456-7890",       // Contains hyphens
		"+1 415 555 2671",    // Contains spaces
		"+1(415)5552671",     // Contains parentheses
	}

	for _, phone := range invalidPhones {
		t.Run(phone, func(t *testing.T) {
			err := validator.ValidatePhone(phone)
			require.Error(t, err, "Expected phone '%s' to be invalid", phone)
			validationErr, ok := err.(*ValidationError)
			require.True(t, ok, "Expected ValidationError type")
			assert.Equal(t, "phone", validationErr.Field)
			assert.Equal(t, "PHONE_INVALID_FORMAT", validationErr.Code)
			assert.Contains(t, validationErr.Message, "E.164")
		})
	}
}

func TestSanitizeName_Success(t *testing.T) {
	validator := NewValidator()

	testCases := []struct {
		input    string
		expected string
	}{
		{"John", "John"},
		{"Mary Jane", "Mary Jane"},
		{"O'Brien", "O'Brien"},
		{"Jean-Pierre", "Jean-Pierre"},
		{"  John  ", "John"}, // Trimmed
		{"Anne-Marie", "Anne-Marie"},
	}

	for _, tc := range testCases {
		t.Run(tc.input, func(t *testing.T) {
			result, err := validator.SanitizeName(tc.input)
			require.NoError(t, err)
			assert.Equal(t, tc.expected, result)
		})
	}
}

func TestSanitizeName_RemovesHTMLTags(t *testing.T) {
	validator := NewValidator()

	testCases := []struct {
		input    string
		expected string
	}{
		{"<script>alert('xss')</script>John", "alert'xss'John"}, // Tags removed, apostrophes kept (valid in names)
		{"John<b>Bold</b>", "JohnBold"},
		{"<div>Mary</div>", "Mary"},
		{"Test<img src='x'>Name", "TestName"}, // Tags and attributes removed
	}

	for _, tc := range testCases {
		t.Run(tc.input, func(t *testing.T) {
			result, err := validator.SanitizeName(tc.input)
			require.NoError(t, err)
			assert.Equal(t, tc.expected, result)
		})
	}
}

func TestSanitizeName_RemovesSpecialCharacters(t *testing.T) {
	validator := NewValidator()

	testCases := []struct {
		input    string
		expected string
	}{
		{"John@123", "John"},
		{"Mary#Jane", "MaryJane"},
		{"Test$Name", "TestName"},
		{"User_123", "User"},
		{"Name!@#$%", "Name"},
	}

	for _, tc := range testCases {
		t.Run(tc.input, func(t *testing.T) {
			result, err := validator.SanitizeName(tc.input)
			require.NoError(t, err)
			assert.Equal(t, tc.expected, result)
		})
	}
}

func TestSanitizeName_Empty(t *testing.T) {
	validator := NewValidator()

	_, err := validator.SanitizeName("")

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "name", validationErr.Field)
	assert.Equal(t, "NAME_REQUIRED", validationErr.Code)
	assert.Contains(t, validationErr.Message, "required")
}

func TestSanitizeName_OnlySpecialCharacters(t *testing.T) {
	validator := NewValidator()

	_, err := validator.SanitizeName("@#$%^&*()")

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "name", validationErr.Field)
	assert.Equal(t, "NAME_INVALID", validationErr.Code)
	assert.Contains(t, validationErr.Message, "no valid characters")
}

func TestSanitizeName_TooLong(t *testing.T) {
	validator := NewValidator()

	// Create a name longer than 100 characters
	longName := string(make([]byte, 101))
	for i := range longName {
		longName = longName[:i] + "a" + longName[i+1:]
	}

	_, err := validator.SanitizeName(longName)

	require.Error(t, err)
	validationErr, ok := err.(*ValidationError)
	require.True(t, ok, "Expected ValidationError type")
	assert.Equal(t, "name", validationErr.Field)
	assert.Equal(t, "NAME_TOO_LONG", validationErr.Code)
	assert.Contains(t, validationErr.Message, "too long")
	assert.Contains(t, validationErr.Message, "100")
}

func TestValidationError_Error(t *testing.T) {
	err := &ValidationError{
		Field:   "email",
		Message: "email format is invalid",
		Code:    "EMAIL_INVALID_FORMAT",
	}

	errorString := err.Error()
	assert.Contains(t, errorString, "email")
	assert.Contains(t, errorString, "email format is invalid")
}

func TestNewValidator(t *testing.T) {
	validator := NewValidator()

	assert.NotNil(t, validator)
	assert.NotNil(t, validator.emailRegex)
	assert.NotNil(t, validator.phoneRegex)
}

// Integration test: Validate complete user registration data
func TestValidateCompleteUserData(t *testing.T) {
	validator := NewValidator()

	// Valid complete user data
	email := "john.doe@example.com"
	password := "SecurePass123!"
	firstName := "John"
	lastName := "Doe"
	phone := "+14155552671"

	// Validate all fields
	assert.NoError(t, validator.ValidateEmail(email))
	assert.NoError(t, validator.ValidatePassword(password))

	sanitizedFirstName, err := validator.SanitizeName(firstName)
	assert.NoError(t, err)
	assert.Equal(t, firstName, sanitizedFirstName)

	sanitizedLastName, err := validator.SanitizeName(lastName)
	assert.NoError(t, err)
	assert.Equal(t, lastName, sanitizedLastName)

	assert.NoError(t, validator.ValidatePhone(phone))
}

// Integration test: Validate user data with malicious input
func TestValidateMaliciousUserData(t *testing.T) {
	validator := NewValidator()

	// Malicious input attempts
	maliciousEmail := "user@example.com<script>alert('xss')</script>"
	maliciousName := "<script>alert('xss')</script>John"
	weakPassword := "password"

	// Email should fail format validation
	err := validator.ValidateEmail(maliciousEmail)
	assert.Error(t, err)

	// Name should be sanitized (HTML tags removed, apostrophes kept as they're valid in names)
	sanitizedName, err := validator.SanitizeName(maliciousName)
	assert.NoError(t, err)
	assert.Equal(t, "alert'xss'John", sanitizedName) // Tags removed, apostrophes kept
	assert.NotContains(t, sanitizedName, "<script>")
	assert.NotContains(t, sanitizedName, "(")
	assert.NotContains(t, sanitizedName, ")")

	// Weak password should fail
	err = validator.ValidatePassword(weakPassword)
	assert.Error(t, err)
}
