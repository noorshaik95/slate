package ratelimit

import (
	"context"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
)

// RateLimit defines rate limiting configuration
type RateLimit struct {
	MaxAttempts int
	Window      time.Duration
}

// RedisRateLimiter implements distributed rate limiting using Redis
type RedisRateLimiter struct {
	client        *redis.Client
	loginLimit    RateLimit
	registerLimit RateLimit
}

// NewRedisRateLimiter creates a new Redis-based rate limiter
func NewRedisRateLimiter(redisAddr string, loginLimit, registerLimit RateLimit) (*RedisRateLimiter, error) {
	client := redis.NewClient(&redis.Options{
		Addr:         redisAddr,
		Password:     "", // No password for local development
		DB:           0,  // Default DB
		DialTimeout:  5 * time.Second,
		ReadTimeout:  3 * time.Second,
		WriteTimeout: 3 * time.Second,
		PoolSize:     10,
		MinIdleConns: 5,
	})

	// Test connection
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := client.Ping(ctx).Err(); err != nil {
		return nil, fmt.Errorf("failed to connect to Redis: %w", err)
	}

	return &RedisRateLimiter{
		client:        client,
		loginLimit:    loginLimit,
		registerLimit: registerLimit,
	}, nil
}

// AllowLogin checks if a login attempt is allowed for the given client IP
// Returns: allowed (bool), retryAfter (time.Duration), error
func (r *RedisRateLimiter) AllowLogin(clientIP string) (bool, time.Duration, error) {
	return r.checkRateLimit(clientIP, "login", r.loginLimit)
}

// AllowRegister checks if a registration attempt is allowed for the given client IP
// Returns: allowed (bool), retryAfter (time.Duration), error
func (r *RedisRateLimiter) AllowRegister(clientIP string) (bool, time.Duration, error) {
	return r.checkRateLimit(clientIP, "register", r.registerLimit)
}

// checkRateLimit implements the sliding window rate limiting algorithm using Redis
func (r *RedisRateLimiter) checkRateLimit(clientIP, operation string, limit RateLimit) (bool, time.Duration, error) {
	ctx := context.Background()
	key := fmt.Sprintf("ratelimit:%s:%s", operation, clientIP)

	// Use Redis pipeline for atomic operations
	pipe := r.client.Pipeline()

	// Increment counter
	incrCmd := pipe.Incr(ctx, key)

	// Set expiration if key is new
	pipe.Expire(ctx, key, limit.Window)

	// Execute pipeline
	_, err := pipe.Exec(ctx)
	if err != nil && err != redis.Nil {
		return false, 0, fmt.Errorf("failed to check rate limit: %w", err)
	}

	// Get the incremented value
	count, err := incrCmd.Result()
	if err != nil {
		return false, 0, fmt.Errorf("failed to get count: %w", err)
	}

	// Check if limit exceeded
	if count > int64(limit.MaxAttempts) {
		// Get TTL to calculate retry-after
		ttl, err := r.client.TTL(ctx, key).Result()
		if err != nil {
			ttl = limit.Window // Fallback to window duration
		}
		return false, ttl, nil
	}

	// If this is the first request, ensure expiration is set
	if count == 1 {
		r.client.Expire(ctx, key, limit.Window)
	}

	return true, 0, nil
}

// Cleanup removes expired entries (Redis handles this automatically with EXPIRE)
// This method is provided for interface compatibility
func (r *RedisRateLimiter) Cleanup(ctx context.Context) error {
	// Redis automatically removes expired keys, so this is a no-op
	// We could add metrics collection here if needed
	return nil
}

// GetRemainingAttempts returns the number of remaining attempts for a client IP
func (r *RedisRateLimiter) GetRemainingAttempts(clientIP, operation string) (int, error) {
	ctx := context.Background()
	key := fmt.Sprintf("ratelimit:%s:%s", operation, clientIP)

	count, err := r.client.Get(ctx, key).Int()
	if err == redis.Nil {
		// No attempts yet
		if operation == "login" {
			return r.loginLimit.MaxAttempts, nil
		}
		return r.registerLimit.MaxAttempts, nil
	}
	if err != nil {
		return 0, fmt.Errorf("failed to get remaining attempts: %w", err)
	}

	var maxAttempts int
	if operation == "login" {
		maxAttempts = r.loginLimit.MaxAttempts
	} else {
		maxAttempts = r.registerLimit.MaxAttempts
	}

	remaining := maxAttempts - count
	if remaining < 0 {
		remaining = 0
	}

	return remaining, nil
}

// Reset resets the rate limit for a specific client IP and operation
// Useful for testing or manual intervention
func (r *RedisRateLimiter) Reset(clientIP, operation string) error {
	ctx := context.Background()
	key := fmt.Sprintf("ratelimit:%s:%s", operation, clientIP)

	err := r.client.Del(ctx, key).Err()
	if err != nil {
		return fmt.Errorf("failed to reset rate limit: %w", err)
	}

	return nil
}

// Close closes the Redis connection
func (r *RedisRateLimiter) Close() error {
	return r.client.Close()
}

// GetStats returns statistics about rate limiting
func (r *RedisRateLimiter) GetStats(ctx context.Context) (map[string]interface{}, error) {
	stats := make(map[string]interface{})

	// Get Redis info
	info, err := r.client.Info(ctx, "stats").Result()
	if err != nil {
		return nil, fmt.Errorf("failed to get Redis stats: %w", err)
	}

	stats["redis_info"] = info
	stats["login_limit"] = map[string]interface{}{
		"max_attempts": r.loginLimit.MaxAttempts,
		"window":       r.loginLimit.Window.String(),
	}
	stats["register_limit"] = map[string]interface{}{
		"max_attempts": r.registerLimit.MaxAttempts,
		"window":       r.registerLimit.Window.String(),
	}

	return stats, nil
}
