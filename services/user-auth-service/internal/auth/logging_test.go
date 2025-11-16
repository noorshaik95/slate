package auth

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"

	"slate/services/user-auth-service/pkg/logger"
)

func TestLogAuthAttempt_Success(t *testing.T) {
	// Create a buffer to capture log output
	var buf bytes.Buffer
	log := logger.NewLoggerWithWriter("info", &buf)
	ctx := context.Background()

	// Log successful authentication
	LogAuthAttempt(log, ctx, AuthTypeNormal, "user@example.com", true, nil)

	// Parse the log output
	output := buf.String()
	if output == "" {
		t.Fatal("expected log output, got empty string")
	}

	// Verify log contains expected fields
	if !strings.Contains(output, "authentication attempt successful") {
		t.Error("log should contain success message")
	}
	if !strings.Contains(output, "normal") {
		t.Error("log should contain auth_type")
	}
	if !strings.Contains(output, "user@example.com") {
		t.Error("log should contain identifier")
	}
	if !strings.Contains(output, "true") {
		t.Error("log should contain success=true")
	}
}

func TestLogAuthAttempt_Failure(t *testing.T) {
	// Create a buffer to capture log output
	var buf bytes.Buffer
	log := logger.NewLoggerWithWriter("info", &buf)
	ctx := context.Background()

	// Create an auth error
	authErr := NewAuthError(ErrInvalidCredentials, "invalid password", errors.New("bcrypt mismatch"))

	// Log failed authentication
	LogAuthAttempt(log, ctx, AuthTypeNormal, "user@example.com", false, authErr)

	// Parse the log output
	output := buf.String()
	if output == "" {
		t.Fatal("expected log output, got empty string")
	}

	// Verify log contains expected fields
	if !strings.Contains(output, "authentication attempt failed") {
		t.Error("log should contain failure message")
	}
	if !strings.Contains(output, "normal") {
		t.Error("log should contain auth_type")
	}
	if !strings.Contains(output, "INVALID_CREDENTIALS") {
		t.Error("log should contain error_type")
	}
	if !strings.Contains(output, "false") {
		t.Error("log should contain success=false")
	}
}

func TestLogAuthAttempt_NoSensitiveData(t *testing.T) {
	// Create a buffer to capture log output
	var buf bytes.Buffer
	log := logger.NewLoggerWithWriter("info", &buf)
	ctx := context.Background()

	// Log authentication with password in identifier (should be sanitized)
	LogAuthAttempt(log, ctx, AuthTypeNormal, "user@example.com", true, nil)

	output := buf.String()

	// Verify no sensitive data in logs
	if strings.Contains(output, "password") {
		t.Error("log should not contain password field")
	}
	if strings.Contains(output, "secret") {
		t.Error("log should not contain secret values")
	}
}

func TestLogAuthEvent_CustomFields(t *testing.T) {
	// Create a buffer to capture log output
	var buf bytes.Buffer
	log := logger.NewLoggerWithWriter("info", &buf)
	ctx := context.Background()

	// Log custom event with various field types
	fields := map[string]interface{}{
		"user_id":  "12345",
		"provider": "google",
		"success":  true,
		"count":    42,
		"password": "should_be_redacted",
	}

	LogAuthEvent(log, ctx, "OAuth token exchange completed", fields)

	output := buf.String()
	if output == "" {
		t.Fatal("expected log output, got empty string")
	}

	// Verify event message
	if !strings.Contains(output, "OAuth token exchange completed") {
		t.Error("log should contain event message")
	}

	// Verify fields are present
	if !strings.Contains(output, "12345") {
		t.Error("log should contain user_id")
	}
	if !strings.Contains(output, "google") {
		t.Error("log should contain provider")
	}

	// Verify password is redacted
	if !strings.Contains(output, "[REDACTED]") {
		t.Error("log should contain [REDACTED] for password")
	}
	if strings.Contains(output, "should_be_redacted") {
		t.Error("log should not contain actual password value")
	}
}

func TestLogAuthEvent_EmptyFields(t *testing.T) {
	// Create a buffer to capture log output
	var buf bytes.Buffer
	log := logger.NewLoggerWithWriter("info", &buf)
	ctx := context.Background()

	// Log event with empty fields
	fields := map[string]interface{}{}
	LogAuthEvent(log, ctx, "test event", fields)

	output := buf.String()
	if output == "" {
		t.Fatal("expected log output, got empty string")
	}

	// Verify event message is present
	if !strings.Contains(output, "test event") {
		t.Error("log should contain event message")
	}
}

func TestLogAuthEvent_NilFields(t *testing.T) {
	// Create a buffer to capture log output
	var buf bytes.Buffer
	log := logger.NewLoggerWithWriter("info", &buf)
	ctx := context.Background()

	// Log event with nil fields (should not crash)
	LogAuthEvent(log, ctx, "test event", nil)

	output := buf.String()
	if output == "" {
		t.Fatal("expected log output, got empty string")
	}

	// Verify event message is present
	if !strings.Contains(output, "test event") {
		t.Error("log should contain event message")
	}
}

func TestLogAuthEvent_SensitiveFieldTypes(t *testing.T) {
	// Create a buffer to capture log output
	var buf bytes.Buffer
	log := logger.NewLoggerWithWriter("info", &buf)
	ctx := context.Background()

	// Log event with all sensitive field types
	fields := map[string]interface{}{
		"password":      "secret123",
		"access_token":  "token_abc",
		"refresh_token": "refresh_xyz",
		"client_secret": "client_secret_value",
		"private_key":   "-----BEGIN PRIVATE KEY-----",
		"state":         "csrf_token",
		"safe_field":    "safe_value",
	}

	LogAuthEvent(log, ctx, "test sensitive fields", fields)

	output := buf.String()

	// Verify all sensitive fields are redacted
	sensitiveValues := []string{
		"secret123",
		"token_abc",
		"refresh_xyz",
		"client_secret_value",
		"-----BEGIN PRIVATE KEY-----",
		"csrf_token",
	}

	for _, value := range sensitiveValues {
		if strings.Contains(output, value) {
			t.Errorf("log should not contain sensitive value: %s", value)
		}
	}

	// Verify safe field is preserved
	if !strings.Contains(output, "safe_value") {
		t.Error("log should contain safe_value")
	}

	// Verify [REDACTED] appears for sensitive fields
	redactedCount := strings.Count(output, "[REDACTED]")
	if redactedCount < 6 {
		t.Errorf("expected at least 6 [REDACTED] markers, got %d", redactedCount)
	}
}

func TestLogAuthAttempt_WithTraceID(t *testing.T) {
	// Create a buffer to capture log output
	var buf bytes.Buffer
	log := logger.NewLoggerWithWriter("info", &buf)

	// Create context with trace ID
	ctx := context.WithValue(context.Background(), "trace_id", "test-trace-123")

	// Log authentication attempt
	LogAuthAttempt(log, ctx, AuthTypeOAuth, "user@example.com", true, nil)

	output := buf.String()

	// Parse JSON to verify trace_id is included
	var logEntry map[string]interface{}
	if err := json.Unmarshal([]byte(output), &logEntry); err != nil {
		// If not JSON, just check string contains trace_id
		if !strings.Contains(output, "test-trace-123") {
			t.Error("log should contain trace_id")
		}
	} else {
		// Verify trace_id in JSON
		if traceID, ok := logEntry["trace_id"].(string); !ok || traceID != "test-trace-123" {
			t.Errorf("expected trace_id=test-trace-123, got %v", logEntry["trace_id"])
		}
	}
}

func TestLogAuthAttempt_DifferentAuthTypes(t *testing.T) {
	tests := []struct {
		name     string
		authType AuthType
	}{
		{"normal auth", AuthTypeNormal},
		{"oauth auth", AuthTypeOAuth},
		{"saml auth", AuthTypeSAML},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer
			log := logger.NewLoggerWithWriter("info", &buf)
			ctx := context.Background()

			LogAuthAttempt(log, ctx, tt.authType, "user@example.com", true, nil)

			output := buf.String()
			if !strings.Contains(output, string(tt.authType)) {
				t.Errorf("log should contain auth_type=%s", tt.authType)
			}
		})
	}
}
