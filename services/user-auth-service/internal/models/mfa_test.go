package models

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Helper function for tests
func mustMarshal(t *testing.T, v interface{}) []byte {
	data, err := json.Marshal(v)
	require.NoError(t, err)
	return data
}

func TestNewUserMFA(t *testing.T) {
	userID := "user-123"
	mfaType := "totp"
	secretKey := "encrypted-secret"
	backupCodes := []string{"code1", "code2", "code3"}

	mfa := NewUserMFA(userID, mfaType, secretKey, backupCodes)

	assert.NotNil(t, mfa)
	assert.NotEmpty(t, mfa.ID)
	assert.Equal(t, userID, mfa.UserID)
	assert.Equal(t, mfaType, mfa.MFAType)
	assert.Equal(t, secretKey, mfa.SecretKey)
	assert.Equal(t, backupCodes, mfa.BackupCodes)
	assert.False(t, mfa.IsEnabled) // Should be disabled by default
	assert.False(t, mfa.CreatedAt.IsZero())
	assert.False(t, mfa.UpdatedAt.IsZero())
}

func TestUserMFA_JSONSerialization(t *testing.T) {
	mfa := &UserMFA{
		ID:          "mfa-123",
		UserID:      "user-123",
		MFAType:     "totp",
		IsEnabled:   true,
		SecretKey:   "secret-key",
		BackupCodes: []string{"code1", "code2"},
	}

	// SecretKey and BackupCodes should not be in JSON (tagged with json:"-")
	jsonData := string(mustMarshal(t, mfa))
	assert.NotContains(t, jsonData, "secret-key")
	assert.NotContains(t, jsonData, "code1")
	assert.NotContains(t, jsonData, "code2")
	assert.Contains(t, jsonData, "mfa-123")
	assert.Contains(t, jsonData, "user-123")
}

func TestMFASetupResponse(t *testing.T) {
	response := &MFASetupResponse{
		Secret:      "secret",
		QRCodeURL:   "https://example.com/qr",
		BackupCodes: []string{"code1", "code2"},
	}

	assert.Equal(t, "secret", response.Secret)
	assert.Equal(t, "https://example.com/qr", response.QRCodeURL)
	assert.Len(t, response.BackupCodes, 2)
}

func TestMFAVerifyRequest(t *testing.T) {
	request := &MFAVerifyRequest{
		UserID: "user-123",
		Code:   "123456",
	}

	assert.Equal(t, "user-123", request.UserID)
	assert.Equal(t, "123456", request.Code)
}
