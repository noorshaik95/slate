package oauth

import (
	"context"
	"fmt"

	"slate/services/user-auth-service/internal/config"
	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel/trace"
)

// MockOAuthHandler implements a mock OAuth provider for testing
// It returns predefined responses without making actual HTTP calls
type MockOAuthHandler struct {
	tracer trace.Tracer
	logger *logger.Logger
}

// NewMockOAuthHandler creates a new mock OAuth handler
func NewMockOAuthHandler(tracer trace.Tracer, logger *logger.Logger) *MockOAuthHandler {
	return &MockOAuthHandler{
		tracer: tracer,
		logger: logger,
	}
}

// GetAuthURL returns a mock authorization URL
func (h *MockOAuthHandler) GetAuthURL(config *config.OAuthProviderConfig, state string) string {
	// Return a mock URL that includes the state
	return fmt.Sprintf("https://mock-oauth-provider.example.com/authorize?client_id=%s&redirect_uri=%s&state=%s&scope=%s",
		config.ClientID,
		config.RedirectURI,
		state,
		"openid profile email",
	)
}

// ExchangeToken returns mock OAuth tokens without making HTTP calls
func (h *MockOAuthHandler) ExchangeToken(ctx context.Context, code string, config *config.OAuthProviderConfig) (*OAuthTokenResponse, error) {
	h.logger.WithContext(ctx).
		Str("code", code).
		Str("provider", "mock").
		Msg("Mock OAuth token exchange")

	// Return mock tokens
	return &OAuthTokenResponse{
		AccessToken:  "mock_access_token_" + code,
		RefreshToken: "mock_refresh_token_" + code,
		TokenType:    "Bearer",
		ExpiresIn:    3600,
	}, nil
}

// GetUserInfo returns mock user information
func (h *MockOAuthHandler) GetUserInfo(ctx context.Context, accessToken string, config *config.OAuthProviderConfig) (*models.OAuthUserInfo, error) {
	h.logger.WithContext(ctx).
		Str("access_token", accessToken).
		Str("provider", "mock").
		Msg("Mock OAuth user info retrieval")

	// Return mock user info
	return &models.OAuthUserInfo{
		Provider:       "mock",
		ProviderUserID: "mock_user_12345",
		Email:          "mock.user@example.com",
		FirstName:      "Mock",
		LastName:       "User",
		AvatarURL:      "https://example.com/avatar.jpg",
	}, nil
}
