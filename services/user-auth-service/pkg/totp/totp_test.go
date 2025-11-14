package totp

import (
	"testing"
	"time"

	otp_totp "github.com/pquerna/otp/totp"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewTOTPManager(t *testing.T) {
	issuer := "TestApp"
	manager := NewTOTPManager(issuer)

	assert.NotNil(t, manager)
	assert.Equal(t, issuer, manager.issuer)
}

func TestGenerateSecret(t *testing.T) {
	manager := NewTOTPManager("TestApp")
	accountName := "test@example.com"

	secret, url, err := manager.GenerateSecret(accountName)

	require.NoError(t, err)
	assert.NotEmpty(t, secret)
	assert.NotEmpty(t, url)
	assert.Contains(t, url, "otpauth://totp/")
	assert.Contains(t, url, accountName)
	assert.Contains(t, url, "TestApp")
}

func TestGenerateQRCodeURL(t *testing.T) {
	manager := NewTOTPManager("TestApp")
	secret := "JBSWY3DPEHPK3PXP"
	accountName := "test@example.com"

	url := manager.GenerateQRCodeURL(secret, accountName)

	assert.NotEmpty(t, url)
	assert.Contains(t, url, "otpauth://totp/")
	assert.Contains(t, url, secret)
	// Email is URL-encoded in the URL
	assert.Contains(t, url, "test%40example.com")
	assert.Contains(t, url, "TestApp")
}

func TestValidateCode(t *testing.T) {
	manager := NewTOTPManager("TestApp")

	// Generate a secret
	secret, _, err := manager.GenerateSecret("test@example.com")
	require.NoError(t, err)

	// Generate current TOTP code
	code, err := generateCurrentCode(secret)
	require.NoError(t, err)

	// Validate the code
	valid := manager.ValidateCode(secret, code)
	assert.True(t, valid, "Current code should be valid")

	// Invalid code should fail
	invalid := manager.ValidateCode(secret, "000000")
	assert.False(t, invalid, "Invalid code should not validate")
}

func TestValidateCodeWithWindow(t *testing.T) {
	manager := NewTOTPManager("TestApp")

	// Generate a secret
	secret, _, err := manager.GenerateSecret("test@example.com")
	require.NoError(t, err)

	// Generate current TOTP code
	code, err := generateCurrentCode(secret)
	require.NoError(t, err)

	// Validate with window
	valid := manager.ValidateCodeWithWindow(secret, code, 1)
	assert.True(t, valid, "Current code should be valid with window")

	// Invalid code should still fail
	invalid := manager.ValidateCodeWithWindow(secret, "000000", 1)
	assert.False(t, invalid, "Invalid code should not validate even with window")
}

func TestGenerateBackupCodes(t *testing.T) {
	manager := NewTOTPManager("TestApp")

	tests := []struct {
		name  string
		count int
	}{
		{"Generate 5 codes", 5},
		{"Generate 10 codes", 10},
		{"Generate 1 code", 1},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			codes, err := manager.GenerateBackupCodes(tt.count)

			require.NoError(t, err)
			assert.Len(t, codes, tt.count)

			// All codes should be unique
			codeMap := make(map[string]bool)
			for _, code := range codes {
				assert.NotEmpty(t, code)
				assert.Len(t, code, 8, "Backup codes should be 8 characters")
				assert.False(t, codeMap[code], "Codes should be unique")
				codeMap[code] = true
			}
		})
	}
}

func TestGenerateBackupCodesUniqueness(t *testing.T) {
	manager := NewTOTPManager("TestApp")

	// Generate multiple sets of codes
	codes1, err := manager.GenerateBackupCodes(10)
	require.NoError(t, err)

	codes2, err := manager.GenerateBackupCodes(10)
	require.NoError(t, err)

	// Codes from different generations should be different
	assert.NotEqual(t, codes1, codes2, "Different generations should produce different codes")
}

func TestGenerateSecretUniqueness(t *testing.T) {
	manager := NewTOTPManager("TestApp")

	// Generate multiple secrets
	secret1, _, err := manager.GenerateSecret("test1@example.com")
	require.NoError(t, err)

	secret2, _, err := manager.GenerateSecret("test2@example.com")
	require.NoError(t, err)

	// Secrets should be different
	assert.NotEqual(t, secret1, secret2, "Different generations should produce different secrets")
}

func TestQRCodeURLFormat(t *testing.T) {
	manager := NewTOTPManager("MyApp")
	secret := "JBSWY3DPEHPK3PXP"
	accountName := "user@example.com"

	url := manager.GenerateQRCodeURL(secret, accountName)

	// Check format: otpauth://totp/MyApp:user%40example.com?secret=JBSWY3DPEHPK3PXP&issuer=MyApp
	assert.Contains(t, url, "otpauth://totp/")
	assert.Contains(t, url, "MyApp:")
	// Email is URL-encoded in the URL
	assert.Contains(t, url, "user%40example.com")
	assert.Contains(t, url, "secret="+secret)
	assert.Contains(t, url, "issuer=MyApp")
}

func TestValidateCodeTimeSensitivity(t *testing.T) {
	manager := NewTOTPManager("TestApp")

	// Generate a secret
	secret, _, err := manager.GenerateSecret("test@example.com")
	require.NoError(t, err)

	// Generate code
	code, err := generateCurrentCode(secret)
	require.NoError(t, err)

	// Should be valid immediately
	valid := manager.ValidateCode(secret, code)
	assert.True(t, valid)

	// Note: We can't easily test expiration without waiting 30 seconds
	// or mocking time, so we just verify it validates correctly now
}

// Helper function to generate current TOTP code for testing
func generateCurrentCode(secret string) (string, error) {
	// Use the pquerna/otp library to generate a code
	code, err := otp_totp.GenerateCode(secret, time.Now())
	return code, err
}
