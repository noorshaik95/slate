package jwt

import (
	"context"
	"strconv"
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
	key := "blacklist:token:" + token
	ttl := time.Until(expiresAt)

	// If token is already expired, no need to blacklist
	if ttl <= 0 {
		return nil
	}

	// Set the token in Redis with TTL matching token expiration
	err := b.client.Set(ctx, key, "1", ttl).Err()
	if err != nil {
		return err
	}

	return nil
}

// BlacklistUserTokens adds all tokens for a user to the blacklist
// This is used when a user changes their password
// The timestamp is used to invalidate all tokens issued before this time
func (b *TokenBlacklist) BlacklistUserTokens(ctx context.Context, userID string, maxTokenLifetime time.Duration) error {
	key := "blacklist:user:" + userID

	// Store the current timestamp
	// Any token issued before this time will be considered blacklisted
	timestamp := time.Now().Unix()

	err := b.client.Set(ctx, key, timestamp, maxTokenLifetime).Err()
	if err != nil {
		return err
	}

	return nil
}

// IsTokenBlacklisted checks if a token is blacklisted
// It checks both token-specific blacklist and user-specific blacklist
func (b *TokenBlacklist) IsTokenBlacklisted(ctx context.Context, token string, userID string, issuedAt time.Time) (bool, error) {
	// Check token-specific blacklist (for logout)
	tokenKey := "blacklist:token:" + token
	exists, err := b.client.Exists(ctx, tokenKey).Result()
	if err != nil {
		return false, err
	}
	if exists > 0 {
		return true, nil
	}

	// Check user-specific blacklist (for password change)
	userKey := "blacklist:user:" + userID
	val, err := b.client.Get(ctx, userKey).Result()
	if err == redis.Nil {
		// Key doesn't exist, token is not blacklisted
		return false, nil
	}
	if err != nil {
		return false, err
	}

	// Parse the blacklist timestamp
	blacklistTime, err := strconv.ParseInt(val, 10, 64)
	if err != nil {
		return false, err
	}

	// If token was issued before the blacklist time, it's blacklisted
	if issuedAt.Unix() < blacklistTime {
		return true, nil
	}

	return false, nil
}
