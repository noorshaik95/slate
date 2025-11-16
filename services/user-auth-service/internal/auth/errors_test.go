package auth

import (
	"errors"
	"testing"

	"google.golang.org/grpc/codes"
)

func TestAuthError_Error(t *testing.T) {
	tests := []struct {
		name     string
		authErr  *AuthError
		expected string
	}{
		{
			name: "error with cause",
			authErr: &AuthError{
				Type:    ErrInvalidCredentials,
				Message: "invalid email or password",
				Cause:   errors.New("user not found"),
			},
			expected: "[INVALID_CREDENTIALS] invalid email or password: user not found",
		},
		{
			name: "error without cause",
			authErr: &AuthError{
				Type:    ErrUserInactive,
				Message: "user account is disabled",
				Cause:   nil,
			},
			expected: "[USER_INACTIVE] user account is disabled",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.authErr.Error()
			if result != tt.expected {
				t.Errorf("Error() = %v, want %v", result, tt.expected)
			}
		})
	}
}

func TestAuthError_Unwrap(t *testing.T) {
	cause := errors.New("underlying error")
	authErr := &AuthError{
		Type:    ErrOAuthFailed,
		Message: "OAuth failed",
		Cause:   cause,
	}

	unwrapped := authErr.Unwrap()
	if unwrapped != cause {
		t.Errorf("Unwrap() = %v, want %v", unwrapped, cause)
	}
}

func TestNewAuthError(t *testing.T) {
	cause := errors.New("test cause")
	authErr := NewAuthError(ErrSAMLFailed, "SAML authentication failed", cause)

	if authErr.Type != ErrSAMLFailed {
		t.Errorf("Type = %v, want %v", authErr.Type, ErrSAMLFailed)
	}
	if authErr.Message != "SAML authentication failed" {
		t.Errorf("Message = %v, want %v", authErr.Message, "SAML authentication failed")
	}
	if authErr.Cause != cause {
		t.Errorf("Cause = %v, want %v", authErr.Cause, cause)
	}
}

func TestMapAuthErrorToGRPCCode_AllTypes(t *testing.T) {
	tests := []struct {
		name         string
		err          error
		expectedCode codes.Code
	}{
		{
			name:         "ErrInvalidCredentials",
			err:          NewAuthError(ErrInvalidCredentials, "invalid credentials", nil),
			expectedCode: codes.Unauthenticated,
		},
		{
			name:         "ErrUserInactive",
			err:          NewAuthError(ErrUserInactive, "user inactive", nil),
			expectedCode: codes.PermissionDenied,
		},
		{
			name:         "ErrOrganizationInactive",
			err:          NewAuthError(ErrOrganizationInactive, "org inactive", nil),
			expectedCode: codes.PermissionDenied,
		},
		{
			name:         "ErrOAuthFailed",
			err:          NewAuthError(ErrOAuthFailed, "oauth failed", nil),
			expectedCode: codes.Internal,
		},
		{
			name:         "ErrSAMLFailed",
			err:          NewAuthError(ErrSAMLFailed, "saml failed", nil),
			expectedCode: codes.Internal,
		},
		{
			name:         "ErrInvalidState",
			err:          NewAuthError(ErrInvalidState, "invalid state", nil),
			expectedCode: codes.InvalidArgument,
		},
		{
			name:         "ErrConfigNotFound",
			err:          NewAuthError(ErrConfigNotFound, "config not found", nil),
			expectedCode: codes.NotFound,
		},
		{
			name:         "ErrInvalidSignature",
			err:          NewAuthError(ErrInvalidSignature, "invalid signature", nil),
			expectedCode: codes.Unauthenticated,
		},
		{
			name:         "ErrAssertionExpired",
			err:          NewAuthError(ErrAssertionExpired, "assertion expired", nil),
			expectedCode: codes.Unauthenticated,
		},
		{
			name:         "ErrJITProvisioningDisabled",
			err:          NewAuthError(ErrJITProvisioningDisabled, "jit disabled", nil),
			expectedCode: codes.NotFound,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			code := MapAuthErrorToGRPCCode(tt.err)
			if code != tt.expectedCode {
				t.Errorf("MapAuthErrorToGRPCCode() = %v, want %v", code, tt.expectedCode)
			}
		})
	}
}

func TestMapAuthErrorToGRPCCode_NonAuthError(t *testing.T) {
	// Test with a generic error (not an AuthError)
	genericErr := errors.New("generic error")
	code := MapAuthErrorToGRPCCode(genericErr)

	if code != codes.Internal {
		t.Errorf("MapAuthErrorToGRPCCode() = %v, want %v", code, codes.Internal)
	}
}

func TestSanitizeForLogging_Password(t *testing.T) {
	data := map[string]interface{}{
		"email":    "user@example.com",
		"password": "secretpassword123",
	}

	sanitized := SanitizeForLogging(data)

	if sanitized["password"] != "[REDACTED]" {
		t.Errorf("password should be redacted, got %v", sanitized["password"])
	}
	if sanitized["email"] != "user@example.com" {
		t.Errorf("email should not be redacted, got %v", sanitized["email"])
	}
}

func TestSanitizeForLogging_Tokens(t *testing.T) {
	data := map[string]interface{}{
		"access_token":  "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
		"refresh_token": "refresh_token_value",
		"client_secret": "client_secret_value",
		"user_id":       "12345",
	}

	sanitized := SanitizeForLogging(data)

	if sanitized["access_token"] != "[REDACTED]" {
		t.Errorf("access_token should be redacted, got %v", sanitized["access_token"])
	}
	if sanitized["refresh_token"] != "[REDACTED]" {
		t.Errorf("refresh_token should be redacted, got %v", sanitized["refresh_token"])
	}
	if sanitized["client_secret"] != "[REDACTED]" {
		t.Errorf("client_secret should be redacted, got %v", sanitized["client_secret"])
	}
	if sanitized["user_id"] != "12345" {
		t.Errorf("user_id should not be redacted, got %v", sanitized["user_id"])
	}
}

func TestSanitizeForLogging_LongValues(t *testing.T) {
	longValue := "this is a very long string that exceeds one hundred characters and should be truncated to only show the first twenty characters"
	data := map[string]interface{}{
		"long_field":  longValue,
		"short_field": "short",
	}

	sanitized := SanitizeForLogging(data)

	if sanitized["long_field"] != "this is a very long ..." {
		t.Errorf("long_field should be truncated, got %v", sanitized["long_field"])
	}
	if sanitized["short_field"] != "short" {
		t.Errorf("short_field should not be truncated, got %v", sanitized["short_field"])
	}
}

func TestSanitizeForLogging_SafeFields(t *testing.T) {
	data := map[string]interface{}{
		"user_id":   "12345",
		"email":     "user@example.com",
		"auth_type": "oauth",
		"provider":  "google",
		"success":   true,
	}

	sanitized := SanitizeForLogging(data)

	// All these fields should remain unchanged
	if sanitized["user_id"] != "12345" {
		t.Errorf("user_id should not be modified, got %v", sanitized["user_id"])
	}
	if sanitized["email"] != "user@example.com" {
		t.Errorf("email should not be modified, got %v", sanitized["email"])
	}
	if sanitized["auth_type"] != "oauth" {
		t.Errorf("auth_type should not be modified, got %v", sanitized["auth_type"])
	}
	if sanitized["provider"] != "google" {
		t.Errorf("provider should not be modified, got %v", sanitized["provider"])
	}
	if sanitized["success"] != true {
		t.Errorf("success should not be modified, got %v", sanitized["success"])
	}
}

func TestSanitizeForLogging_NestedObjects(t *testing.T) {
	// Note: Current implementation doesn't handle nested objects,
	// but we test to ensure it doesn't crash
	data := map[string]interface{}{
		"user": map[string]interface{}{
			"email":    "user@example.com",
			"password": "secret",
		},
		"password": "topsecret",
	}

	sanitized := SanitizeForLogging(data)

	// Top-level password should be redacted
	if sanitized["password"] != "[REDACTED]" {
		t.Errorf("password should be redacted, got %v", sanitized["password"])
	}

	// Nested object should be preserved as-is (not sanitized in current implementation)
	if sanitized["user"] == nil {
		t.Error("nested user object should be preserved")
	}
}

func TestSanitizeForLogging_NilInput(t *testing.T) {
	sanitized := SanitizeForLogging(nil)
	if sanitized != nil {
		t.Errorf("SanitizeForLogging(nil) should return nil, got %v", sanitized)
	}
}

func TestSanitizeForLogging_EmptyMap(t *testing.T) {
	data := map[string]interface{}{}
	sanitized := SanitizeForLogging(data)

	if len(sanitized) != 0 {
		t.Errorf("sanitized map should be empty, got %v", sanitized)
	}
}

func TestSanitizeForLogging_StateParameter(t *testing.T) {
	data := map[string]interface{}{
		"state":    "csrf_token_12345",
		"provider": "google",
	}

	sanitized := SanitizeForLogging(data)

	if sanitized["state"] != "[REDACTED]" {
		t.Errorf("state should be redacted, got %v", sanitized["state"])
	}
	if sanitized["provider"] != "google" {
		t.Errorf("provider should not be redacted, got %v", sanitized["provider"])
	}
}

func TestSanitizeForLogging_PrivateKey(t *testing.T) {
	data := map[string]interface{}{
		"private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBg...",
		"public_key":  "-----BEGIN PUBLIC KEY-----\nMIIBIjANBg...",
	}

	sanitized := SanitizeForLogging(data)

	if sanitized["private_key"] != "[REDACTED]" {
		t.Errorf("private_key should be redacted, got %v", sanitized["private_key"])
	}
	// public_key is not in the sensitive list, so it should be preserved
	if sanitized["public_key"] == "[REDACTED]" {
		t.Error("public_key should not be redacted")
	}
}
