package totp

import (
	"crypto/rand"
	"encoding/base32"
	"errors"
	"net/url"
	"time"

	"github.com/pquerna/otp"
	"github.com/pquerna/otp/totp"
)

// TOTPManager handles TOTP operations
type TOTPManager struct {
	issuer string
}

// NewTOTPManager creates a new TOTP manager
func NewTOTPManager(issuer string) *TOTPManager {
	return &TOTPManager{
		issuer: issuer,
	}
}

// GenerateSecret generates a new TOTP secret for a user
func (m *TOTPManager) GenerateSecret(accountName string) (string, string, error) {
	key, err := totp.Generate(totp.GenerateOpts{
		Issuer:      m.issuer,
		AccountName: accountName,
		Period:      30,
		Digits:      otp.DigitsSix,
		Algorithm:   otp.AlgorithmSHA1,
	})
	if err != nil {
		return "", "", err
	}

	return key.Secret(), key.URL(), nil
}

// GenerateQRCodeURL generates a QR code URL for the secret
func (m *TOTPManager) GenerateQRCodeURL(secret, accountName string) string {
	return "otpauth://totp/" +
		url.QueryEscape(m.issuer) + ":" +
		url.QueryEscape(accountName) +
		"?secret=" + secret +
		"&issuer=" + url.QueryEscape(m.issuer)
}

// ValidateCode validates a TOTP code
func (m *TOTPManager) ValidateCode(secret, code string) bool {
	return totp.Validate(code, secret)
}

// ValidateCodeWithWindow validates a TOTP code with a time window
func (m *TOTPManager) ValidateCodeWithWindow(secret, code string, window uint) bool {
	// Validate with skew (allows for time drift)
	opts := totp.ValidateOpts{
		Period:    30,
		Skew:      window,
		Digits:    otp.DigitsSix,
		Algorithm: otp.AlgorithmSHA1,
	}
	valid, err := totp.ValidateCustom(code, secret, time.Now(), opts)
	return err == nil && valid
}

// GenerateBackupCodes generates backup codes for MFA
func (m *TOTPManager) GenerateBackupCodes(count int) ([]string, error) {
	codes := make([]string, count)
	for i := 0; i < count; i++ {
		code, err := generateRandomCode(8)
		if err != nil {
			return nil, errors.New("failed to generate backup code")
		}
		codes[i] = code
	}
	return codes, nil
}

// generateRandomCode generates a random alphanumeric code
func generateRandomCode(length int) (string, error) {
	randomBytes := make([]byte, 32)
	_, err := rand.Read(randomBytes)
	if err != nil {
		return "", err
	}

	// Use base32 encoding and take the required length
	encoded := base32.StdEncoding.EncodeToString(randomBytes)
	if len(encoded) > length {
		encoded = encoded[:length]
	}

	return encoded, nil
}
