package jwt

import (
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewTokenService(t *testing.T) {
	secretKey := "test-secret-key"
	accessDuration := 15
	refreshDuration := 24

	svc := NewTokenService(secretKey, accessDuration, refreshDuration)

	assert.NotNil(t, svc)
	assert.Equal(t, []byte(secretKey), svc.secretKey)
	assert.Equal(t, time.Duration(accessDuration)*time.Minute, svc.accessTokenDuration)
	assert.Equal(t, time.Duration(refreshDuration)*time.Hour, svc.refreshTokenDuration)
}

func TestGenerateAccessToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	userID := "user-123"
	email := "test@example.com"
	roles := []string{"user", "admin"}

	token, expiresIn, err := svc.GenerateAccessToken(userID, email, roles)

	require.NoError(t, err)
	assert.NotEmpty(t, token)
	assert.Greater(t, expiresIn, int64(0))
	assert.Equal(t, int64(15*60), expiresIn) // 15 minutes in seconds

	// Verify token can be parsed
	claims, err := svc.ValidateAccessToken(token)
	require.NoError(t, err)
	assert.Equal(t, userID, claims.UserID)
	assert.Equal(t, email, claims.Email)
	assert.Equal(t, roles, claims.Roles)
	assert.Equal(t, "access", claims.Type)
}

func TestGenerateRefreshToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	userID := "user-123"
	email := "test@example.com"
	roles := []string{"user"}

	token, err := svc.GenerateRefreshToken(userID, email, roles)

	require.NoError(t, err)
	assert.NotEmpty(t, token)

	// Verify token can be parsed
	claims, err := svc.ValidateRefreshToken(token)
	require.NoError(t, err)
	assert.Equal(t, userID, claims.UserID)
	assert.Equal(t, email, claims.Email)
	assert.Equal(t, roles, claims.Roles)
	assert.Equal(t, "refresh", claims.Type)
}

func TestValidateToken_ValidToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	userID := "user-123"
	email := "test@example.com"
	roles := []string{"user"}

	token, _, err := svc.GenerateAccessToken(userID, email, roles)
	require.NoError(t, err)

	claims, err := svc.ValidateToken(token)

	require.NoError(t, err)
	assert.Equal(t, userID, claims.UserID)
	assert.Equal(t, email, claims.Email)
	assert.Equal(t, roles, claims.Roles)
}

func TestValidateToken_InvalidToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)

	_, err := svc.ValidateToken("invalid-token")

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to parse token")
}

func TestValidateToken_ExpiredToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)

	// Create an expired token
	expiresAt := time.Now().Add(-1 * time.Hour)
	claims := Claims{
		UserID: "user-123",
		Email:  "test@example.com",
		Roles:  []string{"user"},
		Type:   "access",
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(expiresAt),
			IssuedAt:  jwt.NewNumericDate(time.Now().Add(-2 * time.Hour)),
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenString, err := token.SignedString(svc.secretKey)
	require.NoError(t, err)

	_, err = svc.ValidateToken(tokenString)

	assert.Error(t, err)
}

func TestValidateToken_WrongSigningKey(t *testing.T) {
	svc1 := NewTokenService("secret-1", 15, 24)
	svc2 := NewTokenService("secret-2", 15, 24)

	token, _, err := svc1.GenerateAccessToken("user-123", "test@example.com", []string{"user"})
	require.NoError(t, err)

	_, err = svc2.ValidateToken(token)

	assert.Error(t, err)
}

func TestValidateAccessToken_ValidAccessToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	token, _, err := svc.GenerateAccessToken("user-123", "test@example.com", []string{"user"})
	require.NoError(t, err)

	claims, err := svc.ValidateAccessToken(token)

	require.NoError(t, err)
	assert.Equal(t, "access", claims.Type)
}

func TestValidateAccessToken_RefreshTokenProvided(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	token, err := svc.GenerateRefreshToken("user-123", "test@example.com", []string{"user"})
	require.NoError(t, err)

	_, err = svc.ValidateAccessToken(token)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid token type")
}

func TestValidateRefreshToken_ValidRefreshToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	token, err := svc.GenerateRefreshToken("user-123", "test@example.com", []string{"user"})
	require.NoError(t, err)

	claims, err := svc.ValidateRefreshToken(token)

	require.NoError(t, err)
	assert.Equal(t, "refresh", claims.Type)
}

func TestValidateRefreshToken_AccessTokenProvided(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	token, _, err := svc.GenerateAccessToken("user-123", "test@example.com", []string{"user"})
	require.NoError(t, err)

	_, err = svc.ValidateRefreshToken(token)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid token type")
}

func TestRefreshAccessToken_ValidRefreshToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	userID := "user-123"
	email := "test@example.com"
	roles := []string{"user", "admin"}

	refreshToken, err := svc.GenerateRefreshToken(userID, email, roles)
	require.NoError(t, err)

	newAccessToken, newRefreshToken, expiresIn, err := svc.RefreshAccessToken(refreshToken)

	require.NoError(t, err)
	assert.NotEmpty(t, newAccessToken)
	assert.NotEmpty(t, newRefreshToken)
	assert.Greater(t, expiresIn, int64(0))

	// Verify new access token
	claims, err := svc.ValidateAccessToken(newAccessToken)
	require.NoError(t, err)
	assert.Equal(t, userID, claims.UserID)
	assert.Equal(t, email, claims.Email)
	assert.Equal(t, roles, claims.Roles)

	// Verify new refresh token
	refreshClaims, err := svc.ValidateRefreshToken(newRefreshToken)
	require.NoError(t, err)
	assert.Equal(t, userID, refreshClaims.UserID)
}

func TestRefreshAccessToken_InvalidRefreshToken(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)

	_, _, _, err := svc.RefreshAccessToken("invalid-token")

	assert.Error(t, err)
}

func TestRefreshAccessToken_AccessTokenProvided(t *testing.T) {
	svc := NewTokenService("test-secret", 15, 24)
	accessToken, _, err := svc.GenerateAccessToken("user-123", "test@example.com", []string{"user"})
	require.NoError(t, err)

	_, _, _, err = svc.RefreshAccessToken(accessToken)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid token type")
}
