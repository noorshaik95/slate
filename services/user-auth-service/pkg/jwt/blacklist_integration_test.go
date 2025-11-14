package jwt

import (
	"context"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestBlacklistUserTokens_Integration tests the complete flow of blacklisting user tokens
func TestBlacklistUserTokens_Integration(t *testing.T) {
	// Start a mini Redis server for testing
	mr, err := miniredis.Run()
	require.NoError(t, err)
	defer mr.Close()

	// Create Redis client
	client := redis.NewClient(&redis.Options{
		Addr: mr.Addr(),
	})
	defer client.Close()

	// Create blacklist
	blacklist := NewTokenBlacklist(client)
	ctx := context.Background()

	// Test data
	userID := "user-123"
	token1 := "token-issued-before-password-change"
	token2 := "token-issued-after-password-change"
	maxTokenLifetime := 7 * 24 * time.Hour

	// Simulate token issued before password change
	issuedAtBefore := time.Now().Add(-1 * time.Hour)

	// Blacklist all user tokens (simulating password change)
	err = blacklist.BlacklistUserTokens(ctx, userID, maxTokenLifetime)
	require.NoError(t, err)

	// Check that token issued before password change is blacklisted
	isBlacklisted, err := blacklist.IsTokenBlacklisted(ctx, token1, userID, issuedAtBefore)
	require.NoError(t, err)
	assert.True(t, isBlacklisted, "Token issued before password change should be blacklisted")

	// Simulate token issued after password change
	time.Sleep(10 * time.Millisecond) // Small delay to ensure different timestamp
	issuedAtAfter := time.Now()

	// Check that token issued after password change is NOT blacklisted
	isBlacklisted, err = blacklist.IsTokenBlacklisted(ctx, token2, userID, issuedAtAfter)
	require.NoError(t, err)
	assert.False(t, isBlacklisted, "Token issued after password change should NOT be blacklisted")
}

// TestBlacklistToken_Integration tests the complete flow of blacklisting a single token
func TestBlacklistToken_Integration(t *testing.T) {
	// Start a mini Redis server for testing
	mr, err := miniredis.Run()
	require.NoError(t, err)
	defer mr.Close()

	// Create Redis client
	client := redis.NewClient(&redis.Options{
		Addr: mr.Addr(),
	})
	defer client.Close()

	// Create blacklist
	blacklist := NewTokenBlacklist(client)
	ctx := context.Background()

	// Test data
	userID := "user-123"
	token := "token-to-blacklist"
	issuedAt := time.Now()
	expiresAt := time.Now().Add(1 * time.Hour)

	// Blacklist the token (simulating logout)
	err = blacklist.BlacklistToken(ctx, token, expiresAt)
	require.NoError(t, err)

	// Check that the token is blacklisted
	isBlacklisted, err := blacklist.IsTokenBlacklisted(ctx, token, userID, issuedAt)
	require.NoError(t, err)
	assert.True(t, isBlacklisted, "Token should be blacklisted after logout")
}

// TestBlacklistToken_ExpiredToken tests that expired tokens are not blacklisted
func TestBlacklistToken_ExpiredToken(t *testing.T) {
	// Start a mini Redis server for testing
	mr, err := miniredis.Run()
	require.NoError(t, err)
	defer mr.Close()

	// Create Redis client
	client := redis.NewClient(&redis.Options{
		Addr: mr.Addr(),
	})
	defer client.Close()

	// Create blacklist
	blacklist := NewTokenBlacklist(client)
	ctx := context.Background()

	// Test data - token that's already expired
	token := "expired-token"
	expiresAt := time.Now().Add(-1 * time.Hour) // Already expired

	// Try to blacklist the expired token
	err = blacklist.BlacklistToken(ctx, token, expiresAt)
	require.NoError(t, err) // Should not error, just skip

	// Verify the token was not added to Redis
	key := "blacklist:token:" + token
	exists, err := client.Exists(ctx, key).Result()
	require.NoError(t, err)
	assert.Equal(t, int64(0), exists, "Expired token should not be added to blacklist")
}
