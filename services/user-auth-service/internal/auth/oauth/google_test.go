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

func TestGoogleOAuthHandler_GetAuthURL(t *testing.T) {
	handler := NewGoogleOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:    "test-client-id",
		RedirectURI: "http://localhost:8080/callback",
		AuthURL:     "https://accounts.google.com/o/oauth2/v2/auth",
		Scopes:      []string{"openid", "profile", "email"},
	}

	state := "random-state-123"
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
	if !strings.Contains(authURL, "state=random-state-123") {
		t.Error("Auth URL missing state parameter")
	}
	if !strings.Contains(authURL, "access_type=offline") {
		t.Error("Auth URL missing access_type parameter")
	}
	if !strings.Contains(authURL, "prompt=consent") {
		t.Error("Auth URL missing prompt parameter")
	}
}

func TestGoogleOAuthHandler_ExchangeToken_Success(t *testing.T) {
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
			AccessToken:  "test-access-token",
			RefreshToken: "test-refresh-token",
			TokenType:    "Bearer",
			ExpiresIn:    3600,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewGoogleOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:     "test-client-id",
		ClientSecret: "test-client-secret",
		RedirectURI:  "http://localhost:8080/callback",
		TokenURL:     server.URL,
	}

	tokenResp, err := handler.ExchangeToken(context.Background(), "test-code", cfg)
	if err != nil {
		t.Fatalf("ExchangeToken failed: %v", err)
	}

	if tokenResp.AccessToken != "test-access-token" {
		t.Errorf("Expected access token 'test-access-token', got '%s'", tokenResp.AccessToken)
	}
	if tokenResp.RefreshToken != "test-refresh-token" {
		t.Errorf("Expected refresh token 'test-refresh-token', got '%s'", tokenResp.RefreshToken)
	}
	if tokenResp.TokenType != "Bearer" {
		t.Errorf("Expected token type 'Bearer', got '%s'", tokenResp.TokenType)
	}
	if tokenResp.ExpiresIn != 3600 {
		t.Errorf("Expected expires_in 3600, got %d", tokenResp.ExpiresIn)
	}
}

func TestGoogleOAuthHandler_ExchangeToken_InvalidResponse(t *testing.T) {
	// Create mock server returning malformed JSON
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte("invalid json"))
	}))
	defer server.Close()

	handler := NewGoogleOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:     "test-client-id",
		ClientSecret: "test-client-secret",
		RedirectURI:  "http://localhost:8080/callback",
		TokenURL:     server.URL,
	}

	_, err := handler.ExchangeToken(context.Background(), "test-code", cfg)
	if err == nil {
		t.Error("Expected error for invalid JSON response, got nil")
	}
	if !strings.Contains(err.Error(), "failed to parse token response") {
		t.Errorf("Expected parse error, got: %v", err)
	}
}

func TestGoogleOAuthHandler_ExchangeToken_HTTPError(t *testing.T) {
	// Create mock server returning 400 error
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusBadRequest)
		w.Write([]byte(`{"error": "invalid_grant"}`))
	}))
	defer server.Close()

	handler := NewGoogleOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:     "test-client-id",
		ClientSecret: "test-client-secret",
		RedirectURI:  "http://localhost:8080/callback",
		TokenURL:     server.URL,
	}

	_, err := handler.ExchangeToken(context.Background(), "test-code", cfg)
	if err == nil {
		t.Error("Expected error for HTTP 400 response, got nil")
	}
	if !strings.Contains(err.Error(), "token exchange failed with status 400") {
		t.Errorf("Expected HTTP error, got: %v", err)
	}
}

func TestGoogleOAuthHandler_GetUserInfo_Success(t *testing.T) {
	// Create mock server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("Expected GET request, got %s", r.Method)
		}
		authHeader := r.Header.Get("Authorization")
		if authHeader != "Bearer test-access-token" {
			t.Errorf("Expected Authorization header 'Bearer test-access-token', got '%s'", authHeader)
		}

		// Return valid user info response
		response := map[string]interface{}{
			"id":          "123456789",
			"email":       "test@example.com",
			"given_name":  "John",
			"family_name": "Doe",
			"picture":     "https://example.com/avatar.jpg",
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewGoogleOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		UserInfoURL: server.URL,
	}

	userInfo, err := handler.GetUserInfo(context.Background(), "test-access-token", cfg)
	if err != nil {
		t.Fatalf("GetUserInfo failed: %v", err)
	}

	if userInfo.Provider != "google" {
		t.Errorf("Expected provider 'google', got '%s'", userInfo.Provider)
	}
	if userInfo.ProviderUserID != "123456789" {
		t.Errorf("Expected provider user ID '123456789', got '%s'", userInfo.ProviderUserID)
	}
	if userInfo.Email != "test@example.com" {
		t.Errorf("Expected email 'test@example.com', got '%s'", userInfo.Email)
	}
	if userInfo.FirstName != "John" {
		t.Errorf("Expected first name 'John', got '%s'", userInfo.FirstName)
	}
	if userInfo.LastName != "Doe" {
		t.Errorf("Expected last name 'Doe', got '%s'", userInfo.LastName)
	}
	if userInfo.AvatarURL != "https://example.com/avatar.jpg" {
		t.Errorf("Expected avatar URL 'https://example.com/avatar.jpg', got '%s'", userInfo.AvatarURL)
	}
}

func TestGoogleOAuthHandler_GetUserInfo_InvalidToken(t *testing.T) {
	// Create mock server returning 401 error
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
		w.Write([]byte(`{"error": "invalid_token"}`))
	}))
	defer server.Close()

	handler := NewGoogleOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

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

func TestGoogleOAuthHandler_GetUserInfo_MissingFields(t *testing.T) {
	// Create mock server returning incomplete user info
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Return response with only email
		response := map[string]interface{}{
			"id":    "123456789",
			"email": "test@example.com",
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewGoogleOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		UserInfoURL: server.URL,
	}

	userInfo, err := handler.GetUserInfo(context.Background(), "test-access-token", cfg)
	if err != nil {
		t.Fatalf("GetUserInfo failed: %v", err)
	}

	// Should still succeed but with empty fields
	if userInfo.Email != "test@example.com" {
		t.Errorf("Expected email 'test@example.com', got '%s'", userInfo.Email)
	}
	if userInfo.FirstName != "" {
		t.Errorf("Expected empty first name, got '%s'", userInfo.FirstName)
	}
	if userInfo.LastName != "" {
		t.Errorf("Expected empty last name, got '%s'", userInfo.LastName)
	}
	if userInfo.AvatarURL != "" {
		t.Errorf("Expected empty avatar URL, got '%s'", userInfo.AvatarURL)
	}
}
