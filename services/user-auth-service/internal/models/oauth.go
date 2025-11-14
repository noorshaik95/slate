package models

import (
	"time"

	"github.com/google/uuid"
)

// OAuthProvider represents an OAuth provider linked to a user
type OAuthProvider struct {
	ID             string    `json:"id"`
	UserID         string    `json:"user_id"`
	Provider       string    `json:"provider"` // google, github, microsoft, etc.
	ProviderUserID string    `json:"provider_user_id"`
	AccessToken    string    `json:"-"` // Never expose in JSON
	RefreshToken   string    `json:"-"` // Never expose in JSON
	TokenExpiry    time.Time `json:"token_expiry,omitempty"`
	CreatedAt      time.Time `json:"created_at"`
	UpdatedAt      time.Time `json:"updated_at"`
}

// NewOAuthProvider creates a new OAuth provider
func NewOAuthProvider(userID, provider, providerUserID, accessToken, refreshToken string, tokenExpiry time.Time) *OAuthProvider {
	now := time.Now()
	return &OAuthProvider{
		ID:             uuid.New().String(),
		UserID:         userID,
		Provider:       provider,
		ProviderUserID: providerUserID,
		AccessToken:    accessToken,
		RefreshToken:   refreshToken,
		TokenExpiry:    tokenExpiry,
		CreatedAt:      now,
		UpdatedAt:      now,
	}
}

// OAuthUserInfo represents user information from OAuth provider
type OAuthUserInfo struct {
	Provider       string `json:"provider"`
	ProviderUserID string `json:"provider_user_id"`
	Email          string `json:"email"`
	FirstName      string `json:"first_name"`
	LastName       string `json:"last_name"`
	AvatarURL      string `json:"avatar_url"`
	AccessToken    string `json:"-"`
	RefreshToken   string `json:"-"`
	TokenExpiry    time.Time `json:"token_expiry,omitempty"`
}
