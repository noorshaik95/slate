package strategies

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"net/http"
	"sync"
	"time"

	"slate/services/user-auth-service/internal/auth"
	"slate/services/user-auth-service/internal/auth/oauth"
	"slate/services/user-auth-service/internal/auth/services"
	"slate/services/user-auth-service/internal/config"
	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/internal/service"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
)

// OAuthAuthStrategy implements OAuth 2.0 authentication with support for multiple providers.
// It handles the OAuth authorization flow including state management for CSRF protection,
// token exchange, user info retrieval, and user provisioning.
type OAuthAuthStrategy struct {
	config      *config.OAuthConfig
	userRepo    service.UserRepositoryInterface
	oauthRepo   services.OAuthRepositoryInterface
	userService *service.UserService
	tokenSvc    service.TokenServiceInterface
	sessionMgr  *services.SessionManager
	handlers    map[string]oauth.OAuthProviderHandler
	stateStore  map[string]time.Time
	stateMutex  sync.RWMutex
	tracer      trace.Tracer
	logger      *logger.Logger
}

// NewOAuthAuthStrategy creates a new OAuth authentication strategy with the provided dependencies.
// It initializes provider-specific handlers for Google, Microsoft, and custom OAuth providers.
//
// Parameters:
//   - config: OAuth configuration containing provider settings
//   - userRepo: Repository for user data access
//   - oauthRepo: Repository for OAuth provider data access
//   - userService: Service for user operations including token generation
//   - sessionMgr: Manager for storing OAuth tokens
//   - tracer: OpenTelemetry tracer for distributed tracing
//   - logger: Structured logger for authentication events
//
// Returns:
//   - Initialized OAuthAuthStrategy ready for use
func NewOAuthAuthStrategy(
	config *config.OAuthConfig,
	userRepo service.UserRepositoryInterface,
	oauthRepo services.OAuthRepositoryInterface,
	userService *service.UserService,
	tokenSvc service.TokenServiceInterface,
	sessionMgr *services.SessionManager,
	tracer trace.Tracer,
	logger *logger.Logger,
	environment string,
) *OAuthAuthStrategy {
	strategy := &OAuthAuthStrategy{
		config:      config,
		userRepo:    userRepo,
		oauthRepo:   oauthRepo,
		userService: userService,
		tokenSvc:    tokenSvc,
		sessionMgr:  sessionMgr,
		handlers:    make(map[string]oauth.OAuthProviderHandler),
		stateStore:  make(map[string]time.Time),
		tracer:      tracer,
		logger:      logger,
	}

	// Initialize provider handlers
	// Use mock handlers in development/test environments
	if environment == "development" || environment == "test" {
		logger.Info().Msg("Using mock OAuth handlers for development/test environment")
		mockHandler := oauth.NewMockOAuthHandler(tracer, logger)
		strategy.handlers["google"] = mockHandler
		strategy.handlers["microsoft"] = mockHandler
		strategy.handlers["custom"] = mockHandler
	} else {
		httpClient := &http.Client{Timeout: 30 * time.Second}
		strategy.handlers["google"] = oauth.NewGoogleOAuthHandler(httpClient, tracer, logger)
		strategy.handlers["microsoft"] = oauth.NewMicrosoftOAuthHandler(httpClient, tracer, logger)
		strategy.handlers["custom"] = oauth.NewCustomOAuthHandler(httpClient, tracer, logger)
	}

	return strategy
}

// generateState creates a cryptographically secure random state for CSRF protection.
// The state is stored with a timestamp and must be validated within 10 minutes.
//
// Returns:
//   - Base64 URL-safe encoded random state string
func (s *OAuthAuthStrategy) generateState() string {
	// Generate 32 random bytes
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		// This should never happen, but log if it does
		s.logger.Error().Err(err).Msg("Failed to generate random state")
		// Fallback to timestamp-based state (less secure but better than failing)
		return base64.URLEncoding.EncodeToString([]byte(fmt.Sprintf("%d", time.Now().UnixNano())))
	}

	state := base64.URLEncoding.EncodeToString(b)

	// Store state with current timestamp
	s.stateMutex.Lock()
	s.stateStore[state] = time.Now()
	s.stateMutex.Unlock()

	return state
}

// validateState verifies that the state parameter is valid and not expired.
// The state is removed from the store after validation (one-time use).
//
// Parameters:
//   - state: The state parameter to validate
//
// Returns:
//   - nil if state is valid
//   - Error if state is invalid, expired, or already used
func (s *OAuthAuthStrategy) validateState(state string) error {
	s.stateMutex.Lock()
	defer s.stateMutex.Unlock()

	// Check if state exists
	timestamp, exists := s.stateStore[state]
	if !exists {
		return fmt.Errorf("invalid state: state not found or already used")
	}

	// Check if state is expired (10 minutes)
	if time.Since(timestamp) > 10*time.Minute {
		delete(s.stateStore, state)
		return fmt.Errorf("invalid state: state expired")
	}

	// Remove state from store (one-time use)
	delete(s.stateStore, state)

	return nil
}

// getProviderHandler retrieves the OAuth provider handler for the specified provider.
//
// Parameters:
//   - provider: The provider name (google, microsoft, custom)
//
// Returns:
//   - The OAuth provider handler
//   - Error if provider is not found
func (s *OAuthAuthStrategy) getProviderHandler(provider string) (oauth.OAuthProviderHandler, error) {
	handler, exists := s.handlers[provider]
	if !exists {
		return nil, auth.NewAuthError(auth.ErrConfigNotFound, fmt.Sprintf("OAuth provider handler not found for: %s", provider), nil)
	}
	return handler, nil
}

// cleanupExpiredStates removes expired states from the state store.
// This should be called periodically to prevent memory leaks.
func (s *OAuthAuthStrategy) cleanupExpiredStates() {
	s.stateMutex.Lock()
	defer s.stateMutex.Unlock()

	now := time.Now()
	for state, timestamp := range s.stateStore {
		if now.Sub(timestamp) > 10*time.Minute {
			delete(s.stateStore, state)
		}
	}
}

// GetType returns the authentication type for this strategy.
//
// Returns:
//   - AuthTypeOAuth constant
func (s *OAuthAuthStrategy) GetType() auth.AuthType {
	return auth.AuthTypeOAuth
}

// ValidateConfig validates that the strategy has all required configuration and dependencies.
//
// Returns:
//   - nil if configuration is valid
//   - Error describing what is missing or invalid
func (s *OAuthAuthStrategy) ValidateConfig() error {
	if s.config == nil {
		return fmt.Errorf("OAuth config is nil")
	}

	if len(s.config.Providers) == 0 {
		return fmt.Errorf("no OAuth providers configured")
	}

	if s.userRepo == nil {
		return fmt.Errorf("user repository is nil")
	}

	if s.oauthRepo == nil {
		return fmt.Errorf("OAuth repository is nil")
	}

	// userService is optional - we only need tokenSvc for token generation
	// if s.userService == nil {
	// 	return fmt.Errorf("user service is nil")
	// }

	if s.tokenSvc == nil {
		return fmt.Errorf("token service is nil")
	}

	if s.sessionMgr == nil {
		return fmt.Errorf("session manager is nil")
	}

	if s.tracer == nil {
		return fmt.Errorf("tracer is nil")
	}

	if s.logger == nil {
		return fmt.Errorf("logger is nil")
	}

	return nil
}

// Authenticate initiates the OAuth authentication flow by generating an authorization URL.
// The user should be redirected to this URL to authorize with the OAuth provider.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//   - req: Authentication request containing the provider name
//
// Returns:
//   - AuthResult with Success=false and AuthorizationURL for user redirection
//   - Error if provider is missing or not configured
func (s *OAuthAuthStrategy) Authenticate(ctx context.Context, req *auth.AuthRequest) (*auth.AuthResult, error) {
	ctx, span := s.tracer.Start(ctx, "oauth.Authenticate",
		trace.WithAttributes(
			attribute.String("auth.type", "oauth"),
		),
	)
	defer span.End()

	// Add span event: authentication started
	span.AddEvent("authentication.started", trace.WithAttributes(
		attribute.String("auth_type", "oauth"),
		attribute.String("provider", req.Provider),
	))

	// Validate provider is specified
	if req.Provider == "" {
		err := fmt.Errorf("provider is required for OAuth authentication")
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "missing_provider"),
		))
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return nil, err
	}

	// Get provider config
	providerConfig, exists := s.config.Providers[req.Provider]
	if !exists {
		err := fmt.Errorf("OAuth provider not configured: %s", req.Provider)
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "provider_not_configured"),
			attribute.String("provider", req.Provider),
		))
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		s.logger.WarnWithContext(ctx).
			Str("provider", req.Provider).
			Msg("OAuth authentication attempted with unconfigured provider")
		return nil, err
	}

	// Get provider handler
	handler, err := s.getProviderHandler(req.Provider)
	if err != nil {
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "handler_not_found"),
		))
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return nil, err
	}

	// Generate state for CSRF protection
	state := s.generateState()

	// Get authorization URL from provider handler
	authURL := handler.GetAuthURL(&providerConfig, state)

	// Add span attributes
	span.SetAttributes(
		attribute.String("oauth.provider", req.Provider),
		attribute.String("oauth.state", state),
	)

	// Add span event: OAuth flow initiated (user will be redirected)
	span.AddEvent("authentication.oauth_initiated", trace.WithAttributes(
		attribute.String("provider", req.Provider),
		attribute.String("state", state),
	))

	// Log OAuth initiation
	s.logger.WithContext(ctx).
		Str("provider", req.Provider).
		Msg("OAuth authentication initiated")

	// Return auth result with authorization URL
	return &auth.AuthResult{
		Success:          false, // User needs to complete OAuth flow
		AuthorizationURL: authURL,
		State:            state,
	}, nil
}

// HandleCallback processes the OAuth callback after the user has authorized with the provider.
// This method exchanges the authorization code for tokens, retrieves user information,
// creates or updates the user account, and generates JWT tokens.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//   - req: Callback request containing authorization code and state
//
// Returns:
//   - AuthResult with Success=true, user information, and JWT tokens
//   - Error if callback processing fails
func (s *OAuthAuthStrategy) HandleCallback(ctx context.Context, req *auth.CallbackRequest) (*auth.AuthResult, error) {
	ctx, span := s.tracer.Start(ctx, "oauth.HandleCallback",
		trace.WithAttributes(
			attribute.String("auth.type", "oauth"),
		),
	)
	defer span.End()

	// Add span event: callback started
	span.AddEvent("authentication.started", trace.WithAttributes(
		attribute.String("auth_type", "oauth"),
		attribute.String("phase", "callback"),
	))

	// Part 1: Validation
	// Validate code is present
	if req.Code == "" {
		err := fmt.Errorf("authorization code is required")
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "missing_code"),
		))
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return nil, err
	}

	// Validate state is present
	if req.State == "" {
		err := fmt.Errorf("state parameter is required")
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "missing_state"),
		))
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return nil, err
	}

	// Validate state for CSRF protection
	if err := s.validateState(req.State); err != nil {
		authErr := auth.NewAuthError(auth.ErrInvalidState, "invalid OAuth state parameter", err)
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "invalid_state"),
			attribute.String("error.type", string(auth.ErrInvalidState)),
		))
		span.RecordError(authErr)
		span.SetStatus(codes.Error, "invalid state")
		s.logger.WarnWithContext(ctx).
			Err(authErr).
			Msg("OAuth callback with invalid state")
		return nil, authErr
	}

	// Extract provider name from request metadata or use first configured provider
	// In a real implementation, the provider would be encoded in the state or passed separately
	var provider string
	var providerConfig config.OAuthProviderConfig
	for p, cfg := range s.config.Providers {
		provider = p
		providerConfig = cfg
		break // Use first provider for now
	}

	if provider == "" {
		err := auth.NewAuthError(auth.ErrConfigNotFound, "no OAuth provider configured", nil)
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return nil, err
	}

	// Get provider handler
	handler, err := s.getProviderHandler(provider)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return nil, err
	}

	span.SetAttributes(attribute.String("oauth.provider", provider))

	// Part 2: Token Exchange
	s.logger.WithContext(ctx).
		Str("provider", provider).
		Msg("Exchanging authorization code for tokens")

	tokenResponse, err := handler.ExchangeToken(ctx, req.Code, &providerConfig)
	if err != nil {
		authErr := auth.NewAuthError(auth.ErrOAuthFailed, "failed to exchange authorization code for tokens", err)
		span.RecordError(authErr)
		span.SetStatus(codes.Error, "token exchange failed")
		span.SetAttributes(attribute.String("error.type", string(auth.ErrOAuthFailed)))
		s.logger.ErrorWithContext(ctx).
			Str("provider", provider).
			Err(authErr).
			Msg("Failed to exchange authorization code for tokens")
		return nil, authErr
	}

	// Part 3: User Info
	span.AddEvent("authentication.user_info_retrieval", trace.WithAttributes(
		attribute.String("provider", provider),
	))

	s.logger.WithContext(ctx).
		Str("provider", provider).
		Msg("Retrieving user information from provider")

	userInfo, err := handler.GetUserInfo(ctx, tokenResponse.AccessToken, &providerConfig)
	if err != nil {
		authErr := auth.NewAuthError(auth.ErrOAuthFailed, "failed to retrieve user information from provider", err)
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "user_info_failed"),
			attribute.String("error.type", string(auth.ErrOAuthFailed)),
		))
		span.RecordError(authErr)
		span.SetStatus(codes.Error, "user info retrieval failed")
		s.logger.ErrorWithContext(ctx).
			Str("provider", provider).
			Err(authErr).
			Msg("Failed to retrieve user information")
		return nil, authErr
	}

	// Part 4: User Creation/Update
	span.AddEvent("authentication.user_lookup", trace.WithAttributes(
		attribute.String("email", userInfo.Email),
		attribute.String("provider", provider),
	))

	var user *models.User

	// Check if OAuth provider record exists
	existingOAuthProvider, err := s.oauthRepo.GetByProviderAndUserID(ctx, userInfo.Provider, userInfo.ProviderUserID)
	if err == nil && existingOAuthProvider != nil {
		// OAuth provider exists, get the user
		user, err = s.userRepo.GetByID(ctx, existingOAuthProvider.UserID)
		if err != nil {
			authErr := auth.NewAuthError(auth.ErrOAuthFailed, "failed to retrieve user account", err)
			span.RecordError(authErr)
			span.SetStatus(codes.Error, "failed to get user")
			span.SetAttributes(attribute.String("error.type", string(auth.ErrOAuthFailed)))
			return nil, authErr
		}

		// Update user information if changed
		updated := false
		if userInfo.FirstName != "" && user.FirstName != userInfo.FirstName {
			user.FirstName = userInfo.FirstName
			updated = true
		}
		if userInfo.LastName != "" && user.LastName != userInfo.LastName {
			user.LastName = userInfo.LastName
			updated = true
		}
		if userInfo.AvatarURL != "" && user.AvatarURL != userInfo.AvatarURL {
			user.AvatarURL = userInfo.AvatarURL
			updated = true
		}

		if updated {
			user.UpdatedAt = time.Now()
			if err := s.userRepo.Update(ctx, user); err != nil {
				s.logger.WarnWithContext(ctx).
					Str("user_id", user.ID).
					Err(err).
					Msg("Failed to update user information")
				// Don't fail authentication if update fails
			}
		}
	} else {
		// OAuth provider doesn't exist, check if user exists by email
		user, err = s.userRepo.GetByEmail(ctx, userInfo.Email)
		if err == nil && user != nil {
			// User exists, link OAuth provider to existing user
			s.logger.WithContext(ctx).
				Str("user_id", user.ID).
				Str("provider", provider).
				Msg("Linking OAuth provider to existing user")
		} else {
			// User doesn't exist, create new user
			s.logger.WithContext(ctx).
				Str("email", userInfo.Email).
				Str("provider", provider).
				Msg("Creating new user from OAuth")

			user = models.NewUser(
				userInfo.Email,
				"", // No password for OAuth users
				userInfo.FirstName,
				userInfo.LastName,
				"", // No phone from OAuth
			)
			user.AvatarURL = userInfo.AvatarURL
			// Note: auth_method field would be set here if it existed in the model

			if err := s.userRepo.Create(ctx, user); err != nil {
				authErr := auth.NewAuthError(auth.ErrOAuthFailed, "failed to create user account", err)
				span.RecordError(authErr)
				span.SetStatus(codes.Error, "failed to create user")
				span.SetAttributes(attribute.String("error.type", string(auth.ErrOAuthFailed)))
				s.logger.ErrorWithContext(ctx).
					Str("email", userInfo.Email).
					Err(authErr).
					Msg("Failed to create user from OAuth")
				return nil, authErr
			}
		}
	}

	// Create or update OAuth provider record
	tokenExpiry := time.Now().Add(time.Duration(tokenResponse.ExpiresIn) * time.Second)
	oauthProvider := models.NewOAuthProvider(
		user.ID,
		userInfo.Provider,
		userInfo.ProviderUserID,
		tokenResponse.AccessToken,
		tokenResponse.RefreshToken,
		tokenExpiry,
	)

	if err := s.sessionMgr.StoreOAuthTokens(ctx, user.ID, oauthProvider); err != nil {
		s.logger.WarnWithContext(ctx).
			Str("user_id", user.ID).
			Err(err).
			Msg("Failed to store OAuth tokens")
		// Don't fail authentication if token storage fails
	}

	// Part 5: Validation and Session
	span.AddEvent("authentication.validation", trace.WithAttributes(
		attribute.String("user_id", user.ID),
		attribute.Bool("user_active", user.IsActive),
	))

	// Check if user is active
	if !user.IsActive {
		err := auth.NewAuthError(auth.ErrUserInactive, "user account is disabled", nil)
		span.AddEvent("authentication.failed", trace.WithAttributes(
			attribute.String("reason", "user_inactive"),
			attribute.String("error.type", string(auth.ErrUserInactive)),
		))
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		s.logger.WarnWithContext(ctx).
			Str("user_id", user.ID).
			Msg("OAuth authentication attempted for inactive user")
		return nil, err
	}

	// Reload user to get roles
	user, err = s.userRepo.GetByID(ctx, user.ID)
	if err != nil {
		authErr := auth.NewAuthError(auth.ErrOAuthFailed, "failed to reload user data", err)
		span.RecordError(authErr)
		span.SetStatus(codes.Error, "failed to reload user")
		span.SetAttributes(attribute.String("error.type", string(auth.ErrOAuthFailed)))
		return nil, authErr
	}

	// Generate JWT tokens directly using token service
	span.AddEvent("authentication.token_generation", trace.WithAttributes(
		attribute.String("user_id", user.ID),
	))

	accessToken, expiresIn, err := s.tokenSvc.GenerateAccessToken(user.ID, user.Email, user.Roles)
	if err != nil {
		authErr := auth.NewAuthError(auth.ErrOAuthFailed, "failed to generate access token", err)
		span.RecordError(authErr)
		span.SetStatus(codes.Error, "failed to generate access token")
		span.SetAttributes(attribute.String("error.type", string(auth.ErrOAuthFailed)))
		s.logger.ErrorWithContext(ctx).
			Str("user_id", user.ID).
			Err(authErr).
			Msg("Failed to generate access token")
		return nil, authErr
	}

	refreshToken, err := s.tokenSvc.GenerateRefreshToken(user.ID, user.Email, user.Roles)
	if err != nil {
		authErr := auth.NewAuthError(auth.ErrOAuthFailed, "failed to generate refresh token", err)
		span.RecordError(authErr)
		span.SetStatus(codes.Error, "failed to generate refresh token")
		span.SetAttributes(attribute.String("error.type", string(auth.ErrOAuthFailed)))
		s.logger.ErrorWithContext(ctx).
			Str("user_id", user.ID).
			Err(authErr).
			Msg("Failed to generate refresh token")
		return nil, authErr
	}

	tokens := &models.TokenPair{
		AccessToken:  accessToken,
		RefreshToken: refreshToken,
		ExpiresIn:    expiresIn,
	}

	// Add user_id to span
	span.SetAttributes(attribute.String("user_id", user.ID))

	// Add span event: authentication completed
	span.AddEvent("authentication.completed", trace.WithAttributes(
		attribute.String("user_id", user.ID),
		attribute.String("auth_type", "oauth"),
		attribute.String("provider", provider),
	))

	// Log successful authentication
	s.logger.WithContext(ctx).
		Str("user_id", user.ID).
		Str("provider", provider).
		Msg("OAuth authentication successful")

	// Return successful auth result
	return &auth.AuthResult{
		Success: true,
		User:    user,
		Tokens:  tokens,
	}, nil
}
