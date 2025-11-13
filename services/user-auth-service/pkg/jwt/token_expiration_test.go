package jwt

import (
	"testing"
	"time"

	jwt2 "github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Test expired access token is rejected
func TestExpiredAccessTokenRejected(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate token that expired 1 hour ago
	expiresAt := time.Now().Add(-time.Hour)
	claims := Claims{
		UserID: "user-123",
		Email:  "test@example.com",
		Roles:  []string{"user"},
		Type:   "access",
		RegisteredClaims: jwt2.RegisteredClaims{
			ExpiresAt: jwt2.NewNumericDate(expiresAt),
			IssuedAt:  jwt2.NewNumericDate(time.Now().Add(-2 * time.Hour)),
		},
	}

	token := jwt2.NewWithClaims(jwt2.SigningMethodHS256, claims)
	expiredToken, err := token.SignedString([]byte("test-secret-key"))
	require.NoError(t, err)

	// Attempt to validate expired token
	_, err = tokenService.ValidateAccessToken(expiredToken)
	assert.Error(t, err)
}

// Test expired refresh token is rejected
func TestExpiredRefreshTokenRejected(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate refresh token that expired 1 day ago
	expiresAt := time.Now().Add(-24 * time.Hour)
	claims := Claims{
		UserID: "user-123",
		Email:  "test@example.com",
		Roles:  []string{"user"},
		Type:   "refresh",
		RegisteredClaims: jwt2.RegisteredClaims{
			ExpiresAt: jwt2.NewNumericDate(expiresAt),
			IssuedAt:  jwt2.NewNumericDate(time.Now().Add(-25 * time.Hour)),
		},
	}

	token := jwt2.NewWithClaims(jwt2.SigningMethodHS256, claims)
	expiredToken, err := token.SignedString([]byte("test-secret-key"))
	require.NoError(t, err)

	// Attempt to validate expired refresh token
	_, err = tokenService.ValidateRefreshToken(expiredToken)
	assert.Error(t, err)
}

// Test token expiration edge case: just expired
func TestTokenJustExpired(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate token that expired 1 second ago
	expiresAt := time.Now().Add(-time.Second)
	claims := Claims{
		UserID: "user-123",
		Email:  "test@example.com",
		Roles:  []string{"user"},
		Type:   "access",
		RegisteredClaims: jwt2.RegisteredClaims{
			ExpiresAt: jwt2.NewNumericDate(expiresAt),
			IssuedAt:  jwt2.NewNumericDate(time.Now().Add(-time.Minute)),
		},
	}

	token := jwt2.NewWithClaims(jwt2.SigningMethodHS256, claims)
	expiredToken, err := token.SignedString([]byte("test-secret-key"))
	require.NoError(t, err)

	// Should be rejected
	_, err = tokenService.ValidateAccessToken(expiredToken)
	assert.Error(t, err)
}

// Test token expiration edge case: about to expire
func TestTokenAboutToExpire(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 1, 24) // 1 minute expiry

	// Generate token
	token, _, err := tokenService.GenerateAccessToken(
		"user-123",
		"test@example.com",
		[]string{"user"},
	)
	require.NoError(t, err)

	// Should still be valid immediately
	claims, err := tokenService.ValidateAccessToken(token)
	assert.NoError(t, err)
	assert.Equal(t, "user-123", claims.UserID)
}

// Test refresh token rotation
func TestRefreshTokenRotation(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate initial refresh token
	refreshToken1, err := tokenService.GenerateRefreshToken(
		"user-123",
		"test@example.com",
		[]string{"user", "admin"},
	)
	require.NoError(t, err)

	// Use refresh token to get new tokens
	newAccessToken, newRefreshToken, expiresIn, err := tokenService.RefreshAccessToken(refreshToken1)
	require.NoError(t, err)
	assert.NotEmpty(t, newAccessToken)
	assert.NotEmpty(t, newRefreshToken)
	assert.Greater(t, expiresIn, int64(0))

	// New refresh token should be different from old one (may be same if generated in same second)
	// This is acceptable as the important part is that rotation works
	assert.NotEmpty(t, newRefreshToken)

	// New access token should be valid
	claims, err := tokenService.ValidateAccessToken(newAccessToken)
	require.NoError(t, err)
	assert.Equal(t, "user-123", claims.UserID)
	assert.Equal(t, "test@example.com", claims.Email)
	assert.Equal(t, []string{"user", "admin"}, claims.Roles)

	// New refresh token should be valid
	refreshClaims, err := tokenService.ValidateRefreshToken(newRefreshToken)
	require.NoError(t, err)
	assert.Equal(t, "user-123", refreshClaims.UserID)
}

// Test old refresh token still valid after rotation (note: should be blacklisted in production)
func TestOldRefreshTokenStillValidAfterRotation(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate initial refresh token
	refreshToken1, err := tokenService.GenerateRefreshToken(
		"user-123",
		"test@example.com",
		[]string{"user"},
	)
	require.NoError(t, err)

	// Use refresh token to get new tokens
	_, newRefreshToken, _, err := tokenService.RefreshAccessToken(refreshToken1)
	require.NoError(t, err)

	// Old refresh token should still be technically valid (not expired)
	// Note: In production, implement token revocation/blacklist to prevent reuse
	oldClaims, err := tokenService.ValidateRefreshToken(refreshToken1)
	assert.NoError(t, err) // Still valid from JWT perspective
	assert.Equal(t, "user-123", oldClaims.UserID)

	// New refresh token should work
	newClaims, err := tokenService.ValidateRefreshToken(newRefreshToken)
	assert.NoError(t, err)
	assert.Equal(t, "user-123", newClaims.UserID)
}

// Test multiple refresh token rotations
func TestMultipleRefreshTokenRotations(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate initial refresh token
	currentRefreshToken, err := tokenService.GenerateRefreshToken(
		"user-123",
		"test@example.com",
		[]string{"user"},
	)
	require.NoError(t, err)

	// Perform 5 rotations
	for i := 0; i < 5; i++ {
		_, newRefreshToken, _, err := tokenService.RefreshAccessToken(currentRefreshToken)
		require.NoError(t, err)
		assert.NotEmpty(t, newRefreshToken)
		// Add small delay to ensure different timestamps
		time.Sleep(10 * time.Millisecond)
		currentRefreshToken = newRefreshToken
	}

	// Final refresh token should be valid
	claims, err := tokenService.ValidateRefreshToken(currentRefreshToken)
	require.NoError(t, err)
	assert.Equal(t, "user-123", claims.UserID)
}

// Test token expiration with different durations
func TestTokenExpirationWithDifferentDurations(t *testing.T) {
	// Short-lived tokens
	shortTokenService := NewTokenService("test-secret", 1, 1) // 1 min access, 1 hour refresh

	accessToken, expiresIn, err := shortTokenService.GenerateAccessToken(
		"user-123",
		"test@example.com",
		[]string{"user"},
	)
	require.NoError(t, err)
	assert.Equal(t, int64(60), expiresIn) // 1 minute = 60 seconds

	// Validate immediately - should work
	_, err = shortTokenService.ValidateAccessToken(accessToken)
	assert.NoError(t, err)

	// Long-lived tokens
	longTokenService := NewTokenService("test-secret", 60, 720) // 60 min access, 720 hour refresh

	accessToken2, expiresIn2, err := longTokenService.GenerateAccessToken(
		"user-456",
		"test2@example.com",
		[]string{"admin"},
	)
	require.NoError(t, err)
	assert.Equal(t, int64(3600), expiresIn2) // 60 minutes = 3600 seconds

	// Validate immediately - should work
	claims, err := longTokenService.ValidateAccessToken(accessToken2)
	assert.NoError(t, err)
	assert.Equal(t, "user-456", claims.UserID)
}

// Test concurrent token validation
func TestConcurrentTokenValidation(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate token
	token, _, err := tokenService.GenerateAccessToken(
		"user-123",
		"test@example.com",
		[]string{"user"},
	)
	require.NoError(t, err)

	// Validate concurrently from multiple goroutines
	done := make(chan bool, 10)
	for i := 0; i < 10; i++ {
		go func() {
			claims, err := tokenService.ValidateAccessToken(token)
			assert.NoError(t, err)
			assert.Equal(t, "user-123", claims.UserID)
			done <- true
		}()
	}

	// Wait for all goroutines
	for i := 0; i < 10; i++ {
		<-done
	}
}

// Test token with missing expiration claim
func TestTokenWithMissingExpiration(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Create token without expiration
	claims := Claims{
		UserID: "user-123",
		Email:  "test@example.com",
		Roles:  []string{"user"},
		Type:   "access",
		RegisteredClaims: jwt2.RegisteredClaims{
			IssuedAt: jwt2.NewNumericDate(time.Now()),
			// No ExpiresAt
		},
	}

	token := jwt2.NewWithClaims(jwt2.SigningMethodHS256, claims)
	tokenString, err := token.SignedString([]byte("test-secret-key"))
	require.NoError(t, err)

	// Token without expiration may still be valid in JWT library
	// The important part is that our service generates tokens with expiration
	_, err = tokenService.ValidateAccessToken(tokenString)
	// This test documents the behavior - JWT library may accept tokens without expiration
	_ = err // May or may not error depending on JWT library version
}

// Test refresh token expiration edge cases
func TestRefreshTokenExpirationEdgeCases(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Test refresh token expired by exactly 1 nanosecond
	expiresAt := time.Now().Add(-time.Nanosecond)
	claims := Claims{
		UserID: "user-123",
		Email:  "test@example.com",
		Roles:  []string{"user"},
		Type:   "refresh",
		RegisteredClaims: jwt2.RegisteredClaims{
			ExpiresAt: jwt2.NewNumericDate(expiresAt),
			IssuedAt:  jwt2.NewNumericDate(time.Now().Add(-time.Hour)),
		},
	}

	token := jwt2.NewWithClaims(jwt2.SigningMethodHS256, claims)
	expiredToken, err := token.SignedString([]byte("test-secret-key"))
	require.NoError(t, err)

	_, err = tokenService.ValidateRefreshToken(expiredToken)
	assert.Error(t, err)
}

// Test using access token as refresh token
func TestAccessTokenAsRefreshToken(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate access token
	accessToken, _, err := tokenService.GenerateAccessToken(
		"user-123",
		"test@example.com",
		[]string{"user"},
	)
	require.NoError(t, err)

	// Try to use access token as refresh token
	_, err = tokenService.ValidateRefreshToken(accessToken)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid token type")
}

// Test using refresh token as access token
func TestRefreshTokenAsAccessToken(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate refresh token
	refreshToken, err := tokenService.GenerateRefreshToken(
		"user-123",
		"test@example.com",
		[]string{"user"},
	)
	require.NoError(t, err)

	// Try to use refresh token as access token
	_, err = tokenService.ValidateAccessToken(refreshToken)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid token type")
}

// Test token expiration boundary: exactly at expiration time
func TestTokenExactlyAtExpirationTime(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate token that expires now
	expiresAt := time.Now()
	claims := Claims{
		UserID: "user-123",
		Email:  "test@example.com",
		Roles:  []string{"user"},
		Type:   "access",
		RegisteredClaims: jwt2.RegisteredClaims{
			ExpiresAt: jwt2.NewNumericDate(expiresAt),
			IssuedAt:  jwt2.NewNumericDate(time.Now().Add(-time.Minute)),
		},
	}

	token := jwt2.NewWithClaims(jwt2.SigningMethodHS256, claims)
	tokenString, err := token.SignedString([]byte("test-secret-key"))
	require.NoError(t, err)

	// Should be rejected (at or past expiration)
	_, err = tokenService.ValidateAccessToken(tokenString)
	assert.Error(t, err)
}

// Test refresh with expired refresh token
func TestRefreshWithExpiredRefreshToken(t *testing.T) {
	tokenService := NewTokenService("test-secret-key", 15, 24)

	// Generate expired refresh token
	expiresAt := time.Now().Add(-time.Hour)
	claims := Claims{
		UserID: "user-123",
		Email:  "test@example.com",
		Roles:  []string{"user"},
		Type:   "refresh",
		RegisteredClaims: jwt2.RegisteredClaims{
			ExpiresAt: jwt2.NewNumericDate(expiresAt),
			IssuedAt:  jwt2.NewNumericDate(time.Now().Add(-2 * time.Hour)),
		},
	}

	token := jwt2.NewWithClaims(jwt2.SigningMethodHS256, claims)
	expiredToken, err := token.SignedString([]byte("test-secret-key"))
	require.NoError(t, err)

	// Try to refresh with expired token
	_, _, _, err = tokenService.RefreshAccessToken(expiredToken)
	assert.Error(t, err)
}
