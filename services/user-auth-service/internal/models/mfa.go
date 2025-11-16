package models

import (
	"time"

	"github.com/google/uuid"
)

// UserMFA represents multi-factor authentication settings for a user
type UserMFA struct {
	ID          string    `json:"id"`
	UserID      string    `json:"user_id"`
	MFAType     string    `json:"mfa_type"` // totp, sms, email
	IsEnabled   bool      `json:"is_enabled"`
	SecretKey   string    `json:"-"` // Never expose secret in JSON
	BackupCodes []string  `json:"-"` // Never expose backup codes in JSON
	LastUsedAt  time.Time `json:"last_used_at,omitempty"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
}

// NewUserMFA creates a new MFA configuration
func NewUserMFA(userID, mfaType, secretKey string, backupCodes []string) *UserMFA {
	now := time.Now()
	return &UserMFA{
		ID:          uuid.New().String(),
		UserID:      userID,
		MFAType:     mfaType,
		IsEnabled:   false, // Disabled by default until verified
		SecretKey:   secretKey,
		BackupCodes: backupCodes,
		CreatedAt:   now,
		UpdatedAt:   now,
	}
}

// MFASetupResponse is the response for MFA setup
type MFASetupResponse struct {
	Secret      string   `json:"secret"`
	QRCodeURL   string   `json:"qr_code_url"`
	BackupCodes []string `json:"backup_codes"`
}

// MFAVerifyRequest is the request to verify MFA code
type MFAVerifyRequest struct {
	UserID string `json:"user_id"`
	Code   string `json:"code"`
}
