package jwt

import (
	"context"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
)

// TokenBlacklist manages blacklisted tokens using Redis
type TokenBlacklist struct {
	client *redis.Client
}

// NewTokenBlacklist creates a new token blacklist
func NewTokenBlacklist(client *redis.Client) *TokenBlacklist {
	return &TokenBlacklist{
		client: client,
	}
}

// BlacklistToken adds a single token to the blacklist
// The token will be automatically removed when it expires
func (b *TokenBlacklist) BlacklistToken(ctx context.Context, token string, expiresAt time.Time) error {
	key := fmt.Sprintf("blacklist:token:%s", token)
	ttl := time.Until(expiresAt)

	// If token is already expired, no need to blacklist
	if ttl <= 0 {
		return nil
	}

	// Set the token in Redis with TTL matching token expiration
	err := b.client.Set(ctx, key, "1", ttl).Err()
	if err != nil {
		return fmt.Errorf("failed to blacklist token: %w", err)
	}

	return nil
}

// BlacklistUserTokens adds all tokens for a user to the blacklist
// This is used when a user changes their password
// The timestamp is used to invalidate all tokens issued before this time
func (b *TokenBlacklist) BlacklistUserTokens(ctx context.Context, userID string, maxTokenLifetime time.Duration) error {
	key := fmt.Sprintf("blacklist:user:%s", userID)

	// Store the current timestamp
	// Any token issued before this time will be considered blacklisted
	timestamp := time.Now().Unix()

	err := b.client.Set(ctx, key, timestamp, maxTokenLifetime).Err()
	if err != nil {
		return fmt.Errorf("failed to blacklist user tokens: %w", err)
	}

	return nil
}

// IsTokenBlacklisted checks if a token is blacklisted
// It checks both token-specific blacklist and user-specific blacklist
func (b *TokenBlacklist) IsTokenBlacklisted(ctx context.Context, token string, userID string, issuedAt time.Time) (bool, error) {
	// Check token-specific blacklist (for logout)
	tokenKey := fmt.Sprintf("blacklist:token:%s", token)
	exists, err := b.client.Exists(ctx, tokenKey).Result()
	if err != nil {
		return false, fmt.Errorf("failed to check token blacklist: %w", err)
	}
	if exists > 0 {
		return true, nil
	}

	// Check user-specific blacklist (for password change)
	userKey := fmt.Sprintf("blacklist:user:%s", userID)
	val, err := b.client.Get(ctx, userKey).Result()
	if err == redis.Nil {
		// Key doesn't exist, token is not blacklisted
		return false, nil
	}
	if err != nil {
		return false, fmt.Errorf("failed to check user blacklist: %w", err)
	}

	// Parse the blacklist timestamp
	var blacklistTime int64
	_, err = fmt.Sscanf(val, "%d", &blacklistTime)
	if err != nil {
		return false, fmt.Errorf("failed to parse blacklist timestamp: %w", err)
	}

	// If token was issued before the blacklist time, it's blacklisted
	if issuedAt.Unix() < blacklistTime {
		return true, nil
	}

	return false, nil
}
