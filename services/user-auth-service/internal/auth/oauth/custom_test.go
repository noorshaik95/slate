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

func TestCustomOAuthHandler_GetAuthURL(t *testing.T) {
	handler := NewCustomOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:    "custom-client-id",
		RedirectURI: "http://localhost:8080/callback",
		AuthURL:     "https://custom-provider.com/oauth/authorize",
		Scopes:      []string{"read", "write"},
	}

	state := "custom-state-789"
	authURL := handler.GetAuthURL(cfg, state)

	// Verify URL contains required parameters
	if !strings.Contains(authURL, "client_id=custom-client-id") {
		t.Error("Auth URL missing client_id parameter")
	}
	if !strings.Contains(authURL, "redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback") {
		t.Error("Auth URL missing redirect_uri parameter")
	}
	if !strings.Contains(authURL, "response_type=code") {
		t.Error("Auth URL missing response_type parameter")
	}
	if !strings.Contains(authURL, "scope=read+write") {
		t.Error("Auth URL missing scope parameter")
	}
	if !strings.Contains(authURL, "state=custom-state-789") {
		t.Error("Auth URL missing state parameter")
	}
}

func TestCustomOAuthHandler_ExchangeToken_Success(t *testing.T) {
	// Create mock server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("Expected POST request, got %s", r.Method)
		}

		// Return valid token response
		response := OAuthTokenResponse{
			AccessToken:  "custom-access-token",
			RefreshToken: "custom-refresh-token",
			TokenType:    "Bearer",
			ExpiresIn:    7200,
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewCustomOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		ClientID:     "custom-client-id",
		ClientSecret: "custom-client-secret",
		RedirectURI:  "http://localhost:8080/callback",
		TokenURL:     server.URL,
	}

	tokenResp, err := handler.ExchangeToken(context.Background(), "test-code", cfg)
	if err != nil {
		t.Fatalf("ExchangeToken failed: %v", err)
	}

	if tokenResp.AccessToken != "custom-access-token" {
		t.Errorf("Expected access token 'custom-access-token', got '%s'", tokenResp.AccessToken)
	}
	if tokenResp.RefreshToken != "custom-refresh-token" {
		t.Errorf("Expected refresh token 'custom-refresh-token', got '%s'", tokenResp.RefreshToken)
	}
	if tokenResp.ExpiresIn != 7200 {
		t.Errorf("Expected expires_in 7200, got %d", tokenResp.ExpiresIn)
	}
}

func TestCustomOAuthHandler_GetUserInfo_WithAttributeMapping(t *testing.T) {
	// Create mock server with non-standard field names
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Return response with custom field names
		response := map[string]interface{}{
			"user_id":    "custom-123",
			"user_email": "custom@example.com",
			"fname":      "Alice",
			"lname":      "Brown",
			"avatar":     "https://example.com/custom-avatar.jpg",
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewCustomOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		UserInfoURL: server.URL,
		AttributeMapping: map[string]string{
			"id":         "user_id",
			"email":      "user_email",
			"first_name": "fname",
			"last_name":  "lname",
			"avatar_url": "avatar",
		},
	}

	userInfo, err := handler.GetUserInfo(context.Background(), "custom-access-token", cfg)
	if err != nil {
		t.Fatalf("GetUserInfo failed: %v", err)
	}

	if userInfo.Provider != "custom" {
		t.Errorf("Expected provider 'custom', got '%s'", userInfo.Provider)
	}
	if userInfo.ProviderUserID != "custom-123" {
		t.Errorf("Expected provider user ID 'custom-123', got '%s'", userInfo.ProviderUserID)
	}
	if userInfo.Email != "custom@example.com" {
		t.Errorf("Expected email 'custom@example.com', got '%s'", userInfo.Email)
	}
	if userInfo.FirstName != "Alice" {
		t.Errorf("Expected first name 'Alice', got '%s'", userInfo.FirstName)
	}
	if userInfo.LastName != "Brown" {
		t.Errorf("Expected last name 'Brown', got '%s'", userInfo.LastName)
	}
	if userInfo.AvatarURL != "https://example.com/custom-avatar.jpg" {
		t.Errorf("Expected avatar URL 'https://example.com/custom-avatar.jpg', got '%s'", userInfo.AvatarURL)
	}
}

func TestCustomOAuthHandler_GetUserInfo_DefaultMapping(t *testing.T) {
	// Create mock server with standard field names
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Return response with standard OAuth field names
		response := map[string]interface{}{
			"id":          "standard-456",
			"email":       "standard@example.com",
			"given_name":  "Charlie",
			"family_name": "Davis",
			"picture":     "https://example.com/standard-avatar.jpg",
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewCustomOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		UserInfoURL:      server.URL,
		AttributeMapping: nil, // No custom mapping, use defaults
	}

	userInfo, err := handler.GetUserInfo(context.Background(), "custom-access-token", cfg)
	if err != nil {
		t.Fatalf("GetUserInfo failed: %v", err)
	}

	// Should use default field names
	if userInfo.ProviderUserID != "standard-456" {
		t.Errorf("Expected provider user ID 'standard-456', got '%s'", userInfo.ProviderUserID)
	}
	if userInfo.Email != "standard@example.com" {
		t.Errorf("Expected email 'standard@example.com', got '%s'", userInfo.Email)
	}
	if userInfo.FirstName != "Charlie" {
		t.Errorf("Expected first name 'Charlie', got '%s'", userInfo.FirstName)
	}
	if userInfo.LastName != "Davis" {
		t.Errorf("Expected last name 'Davis', got '%s'", userInfo.LastName)
	}
	if userInfo.AvatarURL != "https://example.com/standard-avatar.jpg" {
		t.Errorf("Expected avatar URL 'https://example.com/standard-avatar.jpg', got '%s'", userInfo.AvatarURL)
	}
}

func TestCustomOAuthHandler_GetUserInfo_MissingMappedField(t *testing.T) {
	// Create mock server with incomplete data
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Return response missing some mapped fields
		response := map[string]interface{}{
			"user_id":    "partial-789",
			"user_email": "partial@example.com",
			// fname and lname are missing
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	handler := NewCustomOAuthHandler(http.DefaultClient, otel.Tracer("test"), logger.NewLogger("debug"))

	cfg := &config.OAuthProviderConfig{
		UserInfoURL: server.URL,
		AttributeMapping: map[string]string{
			"id":         "user_id",
			"email":      "user_email",
			"first_name": "fname",
			"last_name":  "lname",
		},
	}

	userInfo, err := handler.GetUserInfo(context.Background(), "custom-access-token", cfg)
	if err != nil {
		t.Fatalf("GetUserInfo failed: %v", err)
	}

	// Should succeed with empty strings for missing fields
	if userInfo.ProviderUserID != "partial-789" {
		t.Errorf("Expected provider user ID 'partial-789', got '%s'", userInfo.ProviderUserID)
	}
	if userInfo.Email != "partial@example.com" {
		t.Errorf("Expected email 'partial@example.com', got '%s'", userInfo.Email)
	}
	if userInfo.FirstName != "" {
		t.Errorf("Expected empty first name, got '%s'", userInfo.FirstName)
	}
	if userInfo.LastName != "" {
		t.Errorf("Expected empty last name, got '%s'", userInfo.LastName)
	}
}
