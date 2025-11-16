package models

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNewOAuthProvider(t *testing.T) {
	userID := "user-123"
	provider := "google"
	providerUserID := "google-user-123"
	accessToken := "access-token"
	refreshToken := "refresh-token"
	tokenExpiry := time.Now().Add(1 * time.Hour)

	oauthProvider := NewOAuthProvider(userID, provider, providerUserID, accessToken, refreshToken, tokenExpiry)

	assert.NotNil(t, oauthProvider)
	assert.NotEmpty(t, oauthProvider.ID)
	assert.Equal(t, userID, oauthProvider.UserID)
	assert.Equal(t, provider, oauthProvider.Provider)
	assert.Equal(t, providerUserID, oauthProvider.ProviderUserID)
	assert.Equal(t, accessToken, oauthProvider.AccessToken)
	assert.Equal(t, refreshToken, oauthProvider.RefreshToken)
	assert.Equal(t, tokenExpiry, oauthProvider.TokenExpiry)
	assert.False(t, oauthProvider.CreatedAt.IsZero())
	assert.False(t, oauthProvider.UpdatedAt.IsZero())
}

func TestOAuthProvider_JSONSerialization(t *testing.T) {
	oauthProvider := &OAuthProvider{
		ID:             "oauth-123",
		UserID:         "user-123",
		Provider:       "google",
		ProviderUserID: "google-user-123",
		AccessToken:    "secret-access-token",
		RefreshToken:   "secret-refresh-token",
		TokenExpiry:    time.Now(),
		CreatedAt:      time.Now(),
		UpdatedAt:      time.Now(),
	}

	// AccessToken and RefreshToken should not be in JSON (tagged with json:"-")
	jsonData := string(mustMarshal(t, oauthProvider))
	assert.Contains(t, jsonData, "oauth-123")
	assert.NotContains(t, jsonData, "secret-access-token")
	assert.NotContains(t, jsonData, "secret-refresh-token")
}

func TestOAuthUserInfo(t *testing.T) {
	userInfo := &OAuthUserInfo{
		Provider:       "google",
		ProviderUserID: "google-123",
		Email:          "user@example.com",
		FirstName:      "John",
		LastName:       "Doe",
		AvatarURL:      "https://example.com/avatar.jpg",
		AccessToken:    "access",
		RefreshToken:   "refresh",
		TokenExpiry:    time.Now(),
	}

	assert.Equal(t, "google", userInfo.Provider)
	assert.Equal(t, "google-123", userInfo.ProviderUserID)
	assert.Equal(t, "user@example.com", userInfo.Email)
	assert.Equal(t, "John", userInfo.FirstName)
	assert.Equal(t, "Doe", userInfo.LastName)
	assert.Equal(t, "https://example.com/avatar.jpg", userInfo.AvatarURL)
}
