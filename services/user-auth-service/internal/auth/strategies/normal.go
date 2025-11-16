package strategies

import (
	"context"
	"fmt"

	"slate/services/user-auth-service/internal/auth"
	"slate/services/user-auth-service/internal/service"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
)

// NormalAuthStrategy wraps existing UserService.Login for strategy pattern compatibility.
// This strategy implements traditional username/password authentication by delegating
// to the existing, tested UserService.Login implementation.
type NormalAuthStrategy struct {
	userService service.UserServiceInterface
	tracer      trace.Tracer
	logger      *logger.Logger
}

// NewNormalAuthStrategy creates a new normal authentication strategy.
// It wraps the existing UserService to provide strategy pattern compatibility.
func NewNormalAuthStrategy(userService service.UserServiceInterface, tracer trace.Tracer, logger *logger.Logger) *NormalAuthStrategy {
	return &NormalAuthStrategy{
		userService: userService,
		tracer:      tracer,
		logger:      logger,
	}
}

// GetType returns the authentication type for this strategy.
func (s *NormalAuthStrategy) GetType() auth.AuthType {
	return auth.AuthTypeNormal
}

// ValidateConfig validates that all required dependencies are present.
func (s *NormalAuthStrategy) ValidateConfig() error {
	if s.userService == nil {
		return fmt.Errorf("userService is required for normal authentication")
	}
	return nil
}

// HandleCallback returns an error as normal authentication does not support callbacks.
// This method is required by the AuthenticationStrategy interface but is not used
// for normal authentication flows.
func (s *NormalAuthStrategy) HandleCallback(ctx context.Context, req *auth.CallbackRequest) (*auth.AuthResult, error) {
	return nil, auth.NewAuthError(auth.ErrConfigNotFound, "normal authentication does not support callbacks", nil)
}

// Authenticate performs username/password authentication by delegating to UserService.Login.
// This method validates the request, calls the existing Login implementation, and returns
// an AuthResult with the authenticated user and tokens.
func (s *NormalAuthStrategy) Authenticate(ctx context.Context, req *auth.AuthRequest) (*auth.AuthResult, error) {
	// Create OpenTelemetry span for tracing
	ctx, span := s.tracer.Start(ctx, "normal.Authenticate")
	defer span.End()

	// Add authentication type attribute
	span.SetAttributes(attribute.String("auth.type", "normal"))

	// Add span event: authentication started
	span.AddEvent("authentication.started", trace.WithAttributes(
		attribute.String("auth_type", "normal"),
		attribute.String("email", req.Email),
	))

	// Validate email is provided
	if req.Email == "" {
		err := auth.NewAuthError(auth.ErrInvalidCredentials, "email is required for normal authentication", nil)
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "missing_email"),
			attribute.String("error.type", string(auth.ErrInvalidCredentials)),
		))
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return nil, err
	}

	// Validate password is provided
	if req.Password == "" {
		err := auth.NewAuthError(auth.ErrInvalidCredentials, "password is required for normal authentication", nil)
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "missing_password"),
			attribute.String("error.type", string(auth.ErrInvalidCredentials)),
		))
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return nil, err
	}

	// Add span event: user lookup
	span.AddEvent("authentication.user_lookup", trace.WithAttributes(
		attribute.String("email", req.Email),
	))

	// Call existing UserService.Login - reuse existing implementation
	// This internally handles user lookup, password validation, and token generation
	user, tokens, err := s.userService.Login(ctx, req.Email, req.Password)
	if err != nil {
		// Wrap the error with AuthError type
		authErr := auth.NewAuthError(auth.ErrInvalidCredentials, "authentication failed", err)
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "login_failed"),
			attribute.String("error.type", string(auth.ErrInvalidCredentials)),
		))
		span.RecordError(authErr)
		span.SetStatus(codes.Error, authErr.Error())

		// Log authentication attempt using structured logging helper
		auth.LogAuthAttempt(s.logger, ctx, auth.AuthTypeNormal, req.Email, false, authErr)
		return nil, authErr
	}

	// Add user_id to span attributes
	span.SetAttributes(attribute.String("user_id", user.ID))

	// Add span event: validation (UserService.Login already validates user status)
	span.AddEvent("authentication.validation", trace.WithAttributes(
		attribute.String("user_id", user.ID),
		attribute.Bool("user_active", user.IsActive),
	))

	// Add span event: token generation (already done by UserService.Login)
	span.AddEvent("authentication.token_generation", trace.WithAttributes(
		attribute.String("user_id", user.ID),
	))

	// Add span event: authentication completed
	span.AddEvent("authentication.completed", trace.WithAttributes(
		attribute.String("user_id", user.ID),
		attribute.String("auth_type", "normal"),
	))

	// Log successful authentication using structured logging helper
	auth.LogAuthAttempt(s.logger, ctx, auth.AuthTypeNormal, req.Email, true, nil)

	// Create and return AuthResult
	return &auth.AuthResult{
		Success: true,
		User:    user,
		Tokens:  tokens,
	}, nil
}
