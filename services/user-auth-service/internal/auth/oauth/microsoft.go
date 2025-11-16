package oauth

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	"slate/services/user-auth-service/internal/config"
	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
)

// MicrosoftOAuthHandler implements OAuth 2.0 authentication for Microsoft.
type MicrosoftOAuthHandler struct {
	httpClient *http.Client
	tracer     trace.Tracer
	logger     *logger.Logger
}

// NewMicrosoftOAuthHandler creates a new Microsoft OAuth handler.
func NewMicrosoftOAuthHandler(httpClient *http.Client, tracer trace.Tracer, logger *logger.Logger) *MicrosoftOAuthHandler {
	return &MicrosoftOAuthHandler{
		httpClient: httpClient,
		tracer:     tracer,
		logger:     logger,
	}
}

// GetAuthURL generates the Microsoft OAuth authorization URL.
// The user should be redirected to this URL to begin the OAuth flow.
func (h *MicrosoftOAuthHandler) GetAuthURL(config *config.OAuthProviderConfig, state string) string {
	params := url.Values{}
	params.Add("client_id", config.ClientID)
	params.Add("redirect_uri", config.RedirectURI)
	params.Add("response_type", "code")
	params.Add("scope", strings.Join(config.Scopes, " "))
	params.Add("state", state)
	params.Add("response_mode", "query")

	return fmt.Sprintf("%s?%s", config.AuthURL, params.Encode())
}

// ExchangeToken exchanges an authorization code for access and refresh tokens.
func (h *MicrosoftOAuthHandler) ExchangeToken(ctx context.Context, code string, config *config.OAuthProviderConfig) (*OAuthTokenResponse, error) {
	ctx, span := h.tracer.Start(ctx, "oauth.microsoft.ExchangeToken")
	defer span.End()

	span.SetAttributes(
		attribute.String("oauth.provider", "microsoft"),
	)

	// Build form data
	formData := url.Values{}
	formData.Set("code", code)
	formData.Set("client_id", config.ClientID)
	formData.Set("client_secret", config.ClientSecret)
	formData.Set("redirect_uri", config.RedirectURI)
	formData.Set("grant_type", "authorization_code")
	formData.Set("scope", strings.Join(config.Scopes, " "))

	// Create HTTP request
	req, err := http.NewRequestWithContext(ctx, "POST", config.TokenURL, strings.NewReader(formData.Encode()))
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to create request")
		h.logger.Error().Err(err).Msg("Failed to create token exchange request")
		return nil, fmt.Errorf("failed to create token exchange request: %w", err)
	}

	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	// Inject trace context into HTTP headers for distributed tracing
	otel.GetTextMapPropagator().Inject(ctx, &httpHeaderCarrier{header: req.Header})

	// Execute request
	resp, err := h.httpClient.Do(req)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "token exchange request failed")
		h.logger.Error().Err(err).Msg("Token exchange request failed")
		return nil, fmt.Errorf("token exchange request failed: %w", err)
	}
	defer resp.Body.Close()

	// Read response body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to read response body")
		h.logger.Error().Err(err).Msg("Failed to read token response body")
		return nil, fmt.Errorf("failed to read token response body: %w", err)
	}

	// Check for HTTP errors
	if resp.StatusCode != http.StatusOK {
		span.SetStatus(codes.Error, fmt.Sprintf("token exchange failed with status %d", resp.StatusCode))
		h.logger.Error().Int("status", resp.StatusCode).Str("body", string(body)).Msg("Token exchange failed")
		return nil, fmt.Errorf("token exchange failed with status %d: %s", resp.StatusCode, string(body))
	}

	// Parse JSON response
	var tokenResp OAuthTokenResponse
	if err := json.Unmarshal(body, &tokenResp); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to parse token response")
		h.logger.Error().Err(err).Str("body", string(body)).Msg("Failed to parse token response")
		return nil, fmt.Errorf("failed to parse token response: %w", err)
	}

	h.logger.Info().Str("provider", "microsoft").Msg("Successfully exchanged authorization code for tokens")
	return &tokenResp, nil
}

// GetUserInfo retrieves user information from Microsoft Graph API using an access token.
func (h *MicrosoftOAuthHandler) GetUserInfo(ctx context.Context, accessToken string, config *config.OAuthProviderConfig) (*models.OAuthUserInfo, error) {
	ctx, span := h.tracer.Start(ctx, "oauth.microsoft.GetUserInfo")
	defer span.End()

	span.SetAttributes(
		attribute.String("oauth.provider", "microsoft"),
	)

	// Create HTTP request
	req, err := http.NewRequestWithContext(ctx, "GET", config.UserInfoURL, nil)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to create request")
		h.logger.Error().Err(err).Msg("Failed to create user info request")
		return nil, fmt.Errorf("failed to create user info request: %w", err)
	}

	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", accessToken))

	// Inject trace context into HTTP headers for distributed tracing
	otel.GetTextMapPropagator().Inject(ctx, &httpHeaderCarrier{header: req.Header})

	// Execute request
	resp, err := h.httpClient.Do(req)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "user info request failed")
		h.logger.Error().Err(err).Msg("User info request failed")
		return nil, fmt.Errorf("user info request failed: %w", err)
	}
	defer resp.Body.Close()

	// Read response body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to read response body")
		h.logger.Error().Err(err).Msg("Failed to read user info response body")
		return nil, fmt.Errorf("failed to read user info response body: %w", err)
	}

	// Check for HTTP errors
	if resp.StatusCode != http.StatusOK {
		span.SetStatus(codes.Error, fmt.Sprintf("user info request failed with status %d", resp.StatusCode))
		h.logger.Error().Int("status", resp.StatusCode).Str("body", string(body)).Msg("User info request failed")
		return nil, fmt.Errorf("user info request failed with status %d: %s", resp.StatusCode, string(body))
	}

	// Parse JSON response
	var msUser struct {
		ID                string `json:"id"`
		Mail              string `json:"mail"`
		UserPrincipalName string `json:"userPrincipalName"`
		GivenName         string `json:"givenName"`
		Surname           string `json:"surname"`
	}
	if err := json.Unmarshal(body, &msUser); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to parse user info response")
		h.logger.Error().Err(err).Str("body", string(body)).Msg("Failed to parse user info response")
		return nil, fmt.Errorf("failed to parse user info response: %w", err)
	}

	// Use mail if available, otherwise use userPrincipalName
	email := msUser.Mail
	if email == "" {
		email = msUser.UserPrincipalName
	}

	// Map to OAuthUserInfo
	userInfo := &models.OAuthUserInfo{
		Provider:       "microsoft",
		ProviderUserID: msUser.ID,
		Email:          email,
		FirstName:      msUser.GivenName,
		LastName:       msUser.Surname,
		AvatarURL:      "", // Microsoft Graph doesn't provide avatar URL in basic profile
		AccessToken:    accessToken,
		TokenExpiry:    time.Now().Add(time.Hour), // Default 1 hour, should be updated with actual expiry
	}

	span.SetAttributes(
		attribute.String("user.email", userInfo.Email),
		attribute.String("user.provider_id", userInfo.ProviderUserID),
	)

	h.logger.Info().Str("email", userInfo.Email).Msg("Successfully retrieved user info from Microsoft")
	return userInfo, nil
}
