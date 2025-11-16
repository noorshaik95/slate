package auth

import (
	"context"
	"errors"

	"slate/services/user-auth-service/pkg/logger"

	"github.com/rs/zerolog"
)

// LogAuthAttempt logs an authentication attempt with structured fields
//
// This function logs authentication attempts with consistent formatting, including
// trace ID, auth type, identifier, success status, and error details if applicable.
//
// Parameters:
//   - logger: The logger instance
//   - ctx: Context containing trace information
//   - authType: The authentication type (normal, oauth, saml)
//   - identifier: User identifier (email or organization_id), will be sanitized
//   - success: Whether the authentication was successful
//   - err: Error if authentication failed, nil otherwise
func LogAuthAttempt(log *logger.Logger, ctx context.Context, authType AuthType, identifier string, success bool, err error) {
	// Create base fields
	fields := map[string]interface{}{
		"auth_type":  string(authType),
		"identifier": identifier,
		"success":    success,
	}

	// Add error information if present
	if err != nil {
		fields["error"] = err.Error()

		// Add error type if it's an AuthError
		var authErr *AuthError
		if errors.As(err, &authErr) {
			fields["error_type"] = string(authErr.Type)
		}
	}

	// Sanitize fields before logging
	sanitized := SanitizeForLogging(fields)

	// Log with appropriate level
	if success {
		event := log.WithContext(ctx)
		for key, value := range sanitized {
			addFieldToZerologEvent(event, key, value)
		}
		event.Msg("authentication attempt successful")
	} else {
		event := log.WarnWithContext(ctx)
		for key, value := range sanitized {
			addFieldToZerologEvent(event, key, value)
		}
		event.Msg("authentication attempt failed")
	}
}

// LogAuthEvent logs a custom authentication event with structured fields
//
// This function provides a flexible way to log authentication-related events
// with custom fields. All fields are sanitized before logging to prevent
// exposure of sensitive data.
//
// Parameters:
//   - logger: The logger instance
//   - ctx: Context containing trace information
//   - event: Event description
//   - fields: Map of field names to values
func LogAuthEvent(log *logger.Logger, ctx context.Context, event string, fields map[string]interface{}) {
	// Sanitize fields before logging
	sanitized := SanitizeForLogging(fields)

	// Create log event
	logEvent := log.WithContext(ctx)
	for key, value := range sanitized {
		addFieldToZerologEvent(logEvent, key, value)
	}
	logEvent.Msg(event)
}

// addFieldToZerologEvent is a helper function to add a field to a zerolog event
// It handles different value types appropriately
func addFieldToZerologEvent(event *zerolog.Event, key string, value interface{}) {
	switch v := value.(type) {
	case string:
		event.Str(key, v)
	case int:
		event.Int(key, v)
	case int64:
		event.Int64(key, v)
	case bool:
		event.Bool(key, v)
	case error:
		event.AnErr(key, v)
	default:
		event.Interface(key, v)
	}
}
