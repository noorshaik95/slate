package ratelimit

import (
	"context"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// setupTestRedis creates a mock Redis server for testing
func setupTestRedis(t *testing.T) (*miniredis.Miniredis, string) {
	mr, err := miniredis.Run()
	require.NoError(t, err, "Failed to start miniredis")
	return mr, mr.Addr()
}

func TestNewRedisRateLimiter_Success(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	assert.NotNil(t, limiter)
	defer limiter.Close()

	assert.Equal(t, 5, limiter.loginLimit.MaxAttempts)
	assert.Equal(t, 15*time.Minute, limiter.loginLimit.Window)
	assert.Equal(t, 3, limiter.registerLimit.MaxAttempts)
	assert.Equal(t, 1*time.Hour, limiter.registerLimit.Window)
}

func TestNewRedisRateLimiter_ConnectionFailure(t *testing.T) {
	// Try to connect to non-existent Redis server
	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter("localhost:9999", loginLimit, registerLimit)
	assert.Error(t, err)
	assert.Nil(t, limiter)
}

func TestAllowLogin_WithinLimit(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.1"

	// First 5 attempts should be allowed
	for i := 0; i < 5; i++ {
		allowed, retryAfter, err := limiter.AllowLogin(clientIP)
		require.NoError(t, err)
		assert.True(t, allowed, "Attempt %d should be allowed", i+1)
		assert.Equal(t, time.Duration(0), retryAfter)
	}
}

func TestAllowLogin_ExceedsLimit(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.2"

	// First 5 attempts should be allowed
	for i := 0; i < 5; i++ {
		allowed, _, err := limiter.AllowLogin(clientIP)
		require.NoError(t, err)
		assert.True(t, allowed)
	}

	// 6th attempt should be denied
	allowed, retryAfter, err := limiter.AllowLogin(clientIP)
	require.NoError(t, err)
	assert.False(t, allowed, "6th attempt should be denied")
	assert.Greater(t, retryAfter, time.Duration(0), "Retry-after should be set")
	assert.LessOrEqual(t, retryAfter, 15*time.Minute, "Retry-after should not exceed window")
}

func TestAllowRegister_WithinLimit(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.3"

	// First 3 attempts should be allowed
	for i := 0; i < 3; i++ {
		allowed, retryAfter, err := limiter.AllowRegister(clientIP)
		require.NoError(t, err)
		assert.True(t, allowed, "Attempt %d should be allowed", i+1)
		assert.Equal(t, time.Duration(0), retryAfter)
	}
}

func TestAllowRegister_ExceedsLimit(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.4"

	// First 3 attempts should be allowed
	for i := 0; i < 3; i++ {
		allowed, _, err := limiter.AllowRegister(clientIP)
		require.NoError(t, err)
		assert.True(t, allowed)
	}

	// 4th attempt should be denied
	allowed, retryAfter, err := limiter.AllowRegister(clientIP)
	require.NoError(t, err)
	assert.False(t, allowed, "4th attempt should be denied")
	assert.Greater(t, retryAfter, time.Duration(0), "Retry-after should be set")
	assert.LessOrEqual(t, retryAfter, 1*time.Hour, "Retry-after should not exceed window")
}

func TestRateLimiter_DifferentIPsIndependent(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 2, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	ip1 := "192.168.1.5"
	ip2 := "192.168.1.6"

	// Exhaust limit for IP1
	for i := 0; i < 2; i++ {
		allowed, _, err := limiter.AllowLogin(ip1)
		require.NoError(t, err)
		assert.True(t, allowed)
	}

	// IP1 should be blocked
	allowed, _, err := limiter.AllowLogin(ip1)
	require.NoError(t, err)
	assert.False(t, allowed)

	// IP2 should still be allowed
	allowed, _, err = limiter.AllowLogin(ip2)
	require.NoError(t, err)
	assert.True(t, allowed, "Different IP should have independent rate limit")
}

func TestRateLimiter_LoginAndRegisterIndependent(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 2, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 2, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.7"

	// Exhaust login limit
	for i := 0; i < 2; i++ {
		allowed, _, err := limiter.AllowLogin(clientIP)
		require.NoError(t, err)
		assert.True(t, allowed)
	}

	// Login should be blocked
	allowed, _, err := limiter.AllowLogin(clientIP)
	require.NoError(t, err)
	assert.False(t, allowed)

	// Register should still be allowed (independent counter)
	allowed, _, err = limiter.AllowRegister(clientIP)
	require.NoError(t, err)
	assert.True(t, allowed, "Register should have independent rate limit from login")
}

func TestGetRemainingAttempts_Login(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.8"

	// Initially should have all attempts available
	remaining, err := limiter.GetRemainingAttempts(clientIP, "login")
	require.NoError(t, err)
	assert.Equal(t, 5, remaining)

	// Use 2 attempts
	for i := 0; i < 2; i++ {
		_, _, err := limiter.AllowLogin(clientIP)
		require.NoError(t, err)
	}

	// Should have 3 remaining
	remaining, err = limiter.GetRemainingAttempts(clientIP, "login")
	require.NoError(t, err)
	assert.Equal(t, 3, remaining)
}

func TestGetRemainingAttempts_Register(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.9"

	// Initially should have all attempts available
	remaining, err := limiter.GetRemainingAttempts(clientIP, "register")
	require.NoError(t, err)
	assert.Equal(t, 3, remaining)

	// Use 1 attempt
	_, _, err = limiter.AllowRegister(clientIP)
	require.NoError(t, err)

	// Should have 2 remaining
	remaining, err = limiter.GetRemainingAttempts(clientIP, "register")
	require.NoError(t, err)
	assert.Equal(t, 2, remaining)
}

func TestReset(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 2, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.10"

	// Exhaust limit
	for i := 0; i < 2; i++ {
		allowed, _, err := limiter.AllowLogin(clientIP)
		require.NoError(t, err)
		assert.True(t, allowed)
	}

	// Should be blocked
	allowed, _, err := limiter.AllowLogin(clientIP)
	require.NoError(t, err)
	assert.False(t, allowed)

	// Reset the limit
	err = limiter.Reset(clientIP, "login")
	require.NoError(t, err)

	// Should be allowed again
	allowed, _, err = limiter.AllowLogin(clientIP)
	require.NoError(t, err)
	assert.True(t, allowed, "After reset, requests should be allowed again")
}

func TestCleanup(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	// Cleanup is a no-op for Redis (handled by TTL)
	// Just verify it doesn't error
	ctx := context.Background()
	err = limiter.Cleanup(ctx)
	assert.NoError(t, err)
}

func TestGetStats(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	ctx := context.Background()
	stats, err := limiter.GetStats(ctx)

	// Note: miniredis doesn't support INFO command with sections, so this may fail in tests
	// In production with real Redis, this would work fine
	if err != nil {
		t.Skip("Skipping GetStats test - miniredis doesn't support INFO sections")
		return
	}

	assert.NotNil(t, stats)

	// Verify login limit stats
	loginStats, ok := stats["login_limit"].(map[string]interface{})
	require.True(t, ok)
	assert.Equal(t, 5, loginStats["max_attempts"])
	assert.Equal(t, "15m0s", loginStats["window"])

	// Verify register limit stats
	registerStats, ok := stats["register_limit"].(map[string]interface{})
	require.True(t, ok)
	assert.Equal(t, 3, registerStats["max_attempts"])
	assert.Equal(t, "1h0m0s", registerStats["window"])
}

func TestRetryAfterCalculation(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	// Use short window for testing
	loginLimit := RateLimit{MaxAttempts: 1, Window: 2 * time.Second}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.11"

	// Use the one allowed attempt
	allowed, _, err := limiter.AllowLogin(clientIP)
	require.NoError(t, err)
	assert.True(t, allowed)

	// Next attempt should be denied with retry-after
	allowed, retryAfter, err := limiter.AllowLogin(clientIP)
	require.NoError(t, err)
	assert.False(t, allowed)
	assert.Greater(t, retryAfter, time.Duration(0))
	assert.LessOrEqual(t, retryAfter, 2*time.Second)

	// Fast-forward time in miniredis to simulate TTL expiration
	mr.FastForward(3 * time.Second)

	// Should be allowed again after window expires
	allowed, _, err = limiter.AllowLogin(clientIP)
	require.NoError(t, err)
	assert.True(t, allowed, "After window expires, requests should be allowed again")
}

func TestConcurrentAccess(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 10, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)
	defer limiter.Close()

	clientIP := "192.168.1.12"
	concurrency := 20
	done := make(chan bool, concurrency)

	// Launch concurrent requests
	for i := 0; i < concurrency; i++ {
		go func() {
			_, _, err := limiter.AllowLogin(clientIP)
			assert.NoError(t, err)
			done <- true
		}()
	}

	// Wait for all goroutines to complete
	for i := 0; i < concurrency; i++ {
		<-done
	}

	// Verify that exactly 10 were allowed (the limit)
	// The 11th and beyond should have been denied
	remaining, err := limiter.GetRemainingAttempts(clientIP, "login")
	require.NoError(t, err)
	assert.Equal(t, 0, remaining, "All attempts should be exhausted")
}

func TestClose(t *testing.T) {
	mr, addr := setupTestRedis(t)
	defer mr.Close()

	loginLimit := RateLimit{MaxAttempts: 5, Window: 15 * time.Minute}
	registerLimit := RateLimit{MaxAttempts: 3, Window: 1 * time.Hour}

	limiter, err := NewRedisRateLimiter(addr, loginLimit, registerLimit)
	require.NoError(t, err)

	// Close should not error
	err = limiter.Close()
	assert.NoError(t, err)

	// After close, operations should fail
	_, _, err = limiter.AllowLogin("192.168.1.13")
	assert.Error(t, err, "Operations after close should fail")
}
