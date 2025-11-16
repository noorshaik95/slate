package oauth

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"slate/services/user-auth-service/internal/config"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel"
)

func TestMicrosoftOAuthHandler_GetAuthURL(t *testing.T) {
	handler := NewMicrosoftOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:    "test-client-id",
		RedirectURI: "http://localhost:8080/callback",
		AuthURL:     "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
		Scopes:      []string{"openid", "profile", "email"},
	}

	state := "random-state-456"
	authURL := handler.GetAuthURL(cfg, state)

	// Verify URL contains required parameters
	if !strings.Contains(authURL, "client_id=test-client-id") {
		t.Error("Auth URL missing client_id parameter")
	}
	if !strings.Contains(authURL, "redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback") {
		t.Error("Auth URL missing redirect_uri parameter")
	}
	if !strings.Contains(authURL, "response_type=code") {
		t.Error("Auth URL missing response_type parameter")
	}
	if !strings.Contains(authURL, "scope=openid+profile+email") {
		t.Error("Auth URL missing scope parameter")
	}
	if !strings.Contains(authURL, "state=random-state-456") {
		t.Error("Auth URL missing state parameter")
	}
	if !strings.Contains(authURL, "response_mode=query") {
		t.Error("Auth URL missing response_mode parameter")
	}
}

func TestMicrosoftOAuthHandler_ExchangeToken_Success(t *testing.T) {
	// Create mock server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("Expected POST request, got %s", r.Method)
		}
		if r.Header.Get("Content-Type") != "application/x-www-form-urlencoded" {
			t.Error("Expected Content-Type: application/x-www-form-urlencoded")
		}

		// Return valid token response
		response := OAuthTokenResponse{
			AccessToken:  "ms-access-token",
			RefreshToken: "ms-refresh-token",
			TokenType:    "Bearer",
			ExpiresIn:    3600,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewMicrosoftOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:     "test-client-id",
		ClientSecret: "test-client-secret",
		RedirectURI:  "http://localhost:8080/callback",
		TokenURL:     server.URL,
		Scopes:       []string{"openid", "profile", "email"},
	}

	tokenResp, err := handler.ExchangeToken(context.Background(), "test-code", cfg)
	if err != nil {
		t.Fatalf("ExchangeToken failed: %v", err)
	}

	if tokenResp.AccessToken != "ms-access-token" {
		t.Errorf("Expected access token 'ms-access-token', got '%s'", tokenResp.AccessToken)
	}
	if tokenResp.RefreshToken != "ms-refresh-token" {
		t.Errorf("Expected refresh token 'ms-refresh-token', got '%s'", tokenResp.RefreshToken)
	}
}

func TestMicrosoftOAuthHandler_ExchangeToken_Error(t *testing.T) {
	// Create mock server returning error
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusBadRequest)
		w.Write([]byte(`{"error": "invalid_request", "error_description": "Invalid authorization code"}`))
	}))
	defer server.Close()

	handler := NewMicrosoftOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:     "test-client-id",
		ClientSecret: "test-client-secret",
		RedirectURI:  "http://localhost:8080/callback",
		TokenURL:     server.URL,
		Scopes:       []string{"openid", "profile", "email"},
	}

	_, err := handler.ExchangeToken(context.Background(), "invalid-code", cfg)
	if err == nil {
		t.Error("Expected error for invalid code, got nil")
	}
	if !strings.Contains(err.Error(), "token exchange failed with status 400") {
		t.Errorf("Expected HTTP 400 error, got: %v", err)
	}
}

func TestMicrosoftOAuthHandler_GetUserInfo_Success(t *testing.T) {
	// Create mock server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("Expected GET request, got %s", r.Method)
		}
		authHeader := r.Header.Get("Authorization")
		if authHeader != "Bearer ms-access-token" {
			t.Errorf("Expected Authorization header 'Bearer ms-access-token', got '%s'", authHeader)
		}

		// Return valid Microsoft Graph user response
		response := map[string]interface{}{
			"id":        "ms-user-123",
			"mail":      "user@contoso.com",
			"givenName": "Jane",
			"surname":   "Smith",
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewMicrosoftOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		UserInfoURL: server.URL,
	}

	userInfo, err := handler.GetUserInfo(context.Background(), "ms-access-token", cfg)
	if err != nil {
		t.Fatalf("GetUserInfo failed: %v", err)
	}

	if userInfo.Provider != "microsoft" {
		t.Errorf("Expected provider 'microsoft', got '%s'", userInfo.Provider)
	}
	if userInfo.ProviderUserID != "ms-user-123" {
		t.Errorf("Expected provider user ID 'ms-user-123', got '%s'", userInfo.ProviderUserID)
	}
	if userInfo.Email != "user@contoso.com" {
		t.Errorf("Expected email 'user@contoso.com', got '%s'", userInfo.Email)
	}
	if userInfo.FirstName != "Jane" {
		t.Errorf("Expected first name 'Jane', got '%s'", userInfo.FirstName)
	}
	if userInfo.LastName != "Smith" {
		t.Errorf("Expected last name 'Smith', got '%s'", userInfo.LastName)
	}
}

func TestMicrosoftOAuthHandler_GetUserInfo_UseUserPrincipalName(t *testing.T) {
	// Create mock server returning user without mail field
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Return response with userPrincipalName instead of mail
		response := map[string]interface{}{
			"id":                "ms-user-456",
			"userPrincipalName": "user@contoso.onmicrosoft.com",
			"givenName":         "Bob",
			"surname":           "Johnson",
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewMicrosoftOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		UserInfoURL: server.URL,
	}

	userInfo, err := handler.GetUserInfo(context.Background(), "ms-access-token", cfg)
	if err != nil {
		t.Fatalf("GetUserInfo failed: %v", err)
	}

	// Should use userPrincipalName when mail is not available
	if userInfo.Email != "user@contoso.onmicrosoft.com" {
		t.Errorf("Expected email 'user@contoso.onmicrosoft.com', got '%s'", userInfo.Email)
	}
	if userInfo.FirstName != "Bob" {
		t.Errorf("Expected first name 'Bob', got '%s'", userInfo.FirstName)
	}
	if userInfo.LastName != "Johnson" {
		t.Errorf("Expected last name 'Johnson', got '%s'", userInfo.LastName)
	}
}

func TestMicrosoftOAuthHandler_GetUserInfo_Error(t *testing.T) {
	// Create mock server returning error
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
		w.Write([]byte(`{"error": {"code": "InvalidAuthenticationToken", "message": "Access token is invalid"}}`))
	}))
	defer server.Close()

	handler := NewMicrosoftOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		UserInfoURL: server.URL,
	}

	_, err := handler.GetUserInfo(context.Background(), "invalid-token", cfg)
	if err == nil {
		t.Error("Expected error for invalid token, got nil")
	}
	if !strings.Contains(err.Error(), "user info request failed with status 401") {
		t.Errorf("Expected HTTP 401 error, got: %v", err)
	}
}
