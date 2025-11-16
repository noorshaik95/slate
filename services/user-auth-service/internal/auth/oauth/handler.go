package oauth

import (
	"context"

	"slate/services/user-auth-service/internal/config"
	"slate/services/user-auth-service/internal/models"
)

// OAuthProviderHandler defines the interface for OAuth provider-specific operations.
// Each OAuth provider (Google, Microsoft, Custom) implements this interface to handle
// authorization URL generation, token exchange, and user info retrieval.
//
// Example usage:
//
//	handler := NewGoogleOAuthHandler(httpClient, tracer, logger)
//	authURL := handler.GetAuthURL(config, "random-state-string")
//	// User is redirected to authURL
//	// After authorization, exchange code for tokens:
//	tokens, err := handler.ExchangeToken(ctx, code, config)
//	// Fetch user information:
//	userInfo, err := handler.GetUserInfo(ctx, tokens.AccessToken, config)
type OAuthProviderHandler interface {
	// GetAuthURL generates the OAuth authorization URL for the provider.
	// The state parameter is used for CSRF protection.
	// Returns the complete authorization URL where the user should be redirected.
	GetAuthURL(config *config.OAuthProviderConfig, state string) string

	// ExchangeToken exchanges an authorization code for access and refresh tokens.
	// This is called after the user authorizes the application and is redirected back
	// with an authorization code.
	ExchangeToken(ctx context.Context, code string, config *config.OAuthProviderConfig) (*OAuthTokenResponse, error)

	// GetUserInfo retrieves user information from the OAuth provider using an access token.
	// Returns standardized user information that can be used to create or update a user account.
	GetUserInfo(ctx context.Context, accessToken string, config *config.OAuthProviderConfig) (*models.OAuthUserInfo, error)
}

// OAuthTokenResponse represents the response from an OAuth token exchange.
// This structure is returned by the OAuth provider's token endpoint after
// successfully exchanging an authorization code for tokens.
type OAuthTokenResponse struct {
	// AccessToken is the token used to access protected resources
	AccessToken string `json:"access_token"`

	// RefreshToken is used to obtain new access tokens when they expire
	RefreshToken string `json:"refresh_token,omitempty"`

	// TokenType is typically "Bearer"
	TokenType string `json:"token_type"`

	// ExpiresIn is the number of seconds until the access token expires
	ExpiresIn int `json:"expires_in"`
}
