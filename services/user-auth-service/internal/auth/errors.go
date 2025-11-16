package auth

import (
	"errors"
	"fmt"

	"google.golang.org/grpc/codes"
)

// AuthErrorType represents the type of authentication error
type AuthErrorType string

const (
	// ErrInvalidCredentials indicates invalid username or password
	ErrInvalidCredentials AuthErrorType = "INVALID_CREDENTIALS"

	// ErrUserInactive indicates the user account is disabled
	ErrUserInactive AuthErrorType = "USER_INACTIVE"

	// ErrOrganizationInactive indicates the organization is disabled
	ErrOrganizationInactive AuthErrorType = "ORGANIZATION_INACTIVE"

	// ErrOAuthFailed indicates OAuth authentication failed
	ErrOAuthFailed AuthErrorType = "OAUTH_FAILED"

	// ErrSAMLFailed indicates SAML authentication failed
	ErrSAMLFailed AuthErrorType = "SAML_FAILED"

	// ErrInvalidState indicates invalid OAuth state parameter (CSRF protection)
	ErrInvalidState AuthErrorType = "INVALID_STATE"

	// ErrConfigNotFound indicates authentication configuration not found
	ErrConfigNotFound AuthErrorType = "CONFIG_NOT_FOUND"

	// ErrInvalidSignature indicates invalid SAML signature
	ErrInvalidSignature AuthErrorType = "INVALID_SIGNATURE"

	// ErrAssertionExpired indicates SAML assertion has expired
	ErrAssertionExpired AuthErrorType = "ASSERTION_EXPIRED"

	// ErrJITProvisioningDisabled indicates JIT provisioning is disabled for new users
	ErrJITProvisioningDisabled AuthErrorType = "JIT_PROVISIONING_DISABLED"
)

// AuthError represents an authentication error with type, message, and underlying cause
type AuthError struct {
	// Type is the error type for categorization
	Type AuthErrorType

	// Message is the human-readable error message
	Message string

	// Cause is the underlying error that caused this authentication error
	Cause error
}

// Error implements the error interface, returning a formatted error message
func (e *AuthError) Error() string {
	if e.Cause != nil {
		return fmt.Sprintf("[%s] %s: %v", e.Type, e.Message, e.Cause)
	}
	return fmt.Sprintf("[%s] %s", e.Type, e.Message)
}

// Unwrap returns the underlying cause error, enabling error unwrapping
func (e *AuthError) Unwrap() error {
	return e.Cause
}

// NewAuthError creates a new AuthError with the specified type, message, and cause
//
// Example:
//
//	err := NewAuthError(ErrInvalidCredentials, "invalid email or password", nil)
//	err := NewAuthError(ErrOAuthFailed, "token exchange failed", originalErr)
func NewAuthError(errType AuthErrorType, message string, cause error) *AuthError {
	return &AuthError{
		Type:    errType,
		Message: message,
		Cause:   cause,
	}
}

// MapAuthErrorToGRPCCode maps an AuthError to the appropriate gRPC status code
//
// This function checks if the error is an AuthError and maps its type to the
// corresponding gRPC code. If the error is not an AuthError, it returns codes.Internal.
//
// Mapping:
//   - ErrInvalidCredentials → codes.Unauthenticated
//   - ErrUserInactive → codes.PermissionDenied
//   - ErrOrganizationInactive → codes.PermissionDenied
//   - ErrOAuthFailed → codes.Internal
//   - ErrSAMLFailed → codes.Internal
//   - ErrInvalidState → codes.InvalidArgument
//   - ErrConfigNotFound → codes.NotFound
//   - ErrInvalidSignature → codes.Unauthenticated
//   - ErrAssertionExpired → codes.Unauthenticated
//   - ErrJITProvisioningDisabled → codes.NotFound
//   - Other errors → codes.Internal
func MapAuthErrorToGRPCCode(err error) codes.Code {
	var authErr *AuthError
	if errors.As(err, &authErr) {
		switch authErr.Type {
		case ErrInvalidCredentials:
			return codes.Unauthenticated
		case ErrUserInactive:
			return codes.PermissionDenied
		case ErrOrganizationInactive:
			return codes.PermissionDenied
		case ErrOAuthFailed:
			return codes.Internal
		case ErrSAMLFailed:
			return codes.Internal
		case ErrInvalidState:
			return codes.InvalidArgument
		case ErrConfigNotFound:
			return codes.NotFound
		case ErrInvalidSignature:
			return codes.Unauthenticated
		case ErrAssertionExpired:
			return codes.Unauthenticated
		case ErrJITProvisioningDisabled:
			return codes.NotFound
		default:
			return codes.Internal
		}
	}
	return codes.Internal
}

// SanitizeForLogging sanitizes sensitive data from a map before logging
//
// WARNING: This function should be used for all logging operations that may contain
// sensitive authentication data. It redacts passwords, tokens, secrets, and truncates
// long values to prevent log pollution.
//
// Sensitive fields that are redacted:
//   - password
//   - access_token
//   - refresh_token
//   - client_secret
//   - private_key
//   - state (OAuth state parameter)
//
// Long values (> 100 characters) are truncated to first 20 characters + "..."
func SanitizeForLogging(data map[string]interface{}) map[string]interface{} {
	if data == nil {
		return nil
	}

	// Create a copy to avoid modifying the original
	sanitized := make(map[string]interface{}, len(data))

	// List of sensitive field names to redact
	sensitiveFields := map[string]bool{
		"password":      true,
		"access_token":  true,
		"refresh_token": true,
		"client_secret": true,
		"private_key":   true,
		"state":         true,
	}

	for key, value := range data {
		// Redact sensitive fields
		if sensitiveFields[key] {
			sanitized[key] = "[REDACTED]"
			continue
		}

		// Truncate long string values
		if strValue, ok := value.(string); ok {
			if len(strValue) > 100 {
				sanitized[key] = strValue[:20] + "..."
			} else {
				sanitized[key] = strValue
			}
		} else {
			sanitized[key] = value
		}
	}

	return sanitized
}
