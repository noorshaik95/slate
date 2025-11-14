package ratelimit

import (
	"context"
	"fmt"
	"sync"
	"time"
)

// MetricsRecorder is an interface for recording rate limiter metrics
type MetricsRecorder interface {
	SetRateLimiterMode(usingMemory bool)
}

// FallbackRateLimiter provides automatic fallback to in-memory rate limiting
// when Redis is unavailable, with automatic recovery when Redis comes back online
type FallbackRateLimiter struct {
	redis      *RedisRateLimiter
	memory     *MemoryRateLimiter
	metrics    MetricsRecorder
	mu         sync.RWMutex
	usingRedis bool
	lastCheck  time.Time
}

// NewFallbackRateLimiter creates a new fallback rate limiter
// It attempts to use Redis, but falls back to in-memory if Redis is unavailable
func NewFallbackRateLimiter(redisAddr string, loginLimit, registerLimit RateLimit, metrics MetricsRecorder) (*FallbackRateLimiter, error) {
	// Try to create Redis limiter
	redis, err := NewRedisRateLimiter(redisAddr, loginLimit, registerLimit)

	// Create in-memory limiter as fallback
	memory := NewMemoryRateLimiter(loginLimit, registerLimit)

	usingRedis := err == nil
	fallback := &FallbackRateLimiter{
		redis:      redis,
		memory:     memory,
		metrics:    metrics,
		usingRedis: usingRedis,
		lastCheck:  time.Now(),
	}

	// Update metrics
	if metrics != nil {
		metrics.SetRateLimiterMode(!usingRedis)
	}

	if err != nil {
		fmt.Printf("WARNING: Redis unavailable, using in-memory rate limiter: %v\n", err)
	}

	return fallback, nil
}

// AllowLogin checks if a login attempt is allowed
func (f *FallbackRateLimiter) AllowLogin(clientIP string) (bool, time.Duration, error) {
	f.mu.RLock()
	usingRedis := f.usingRedis
	f.mu.RUnlock()

	if usingRedis {
		allowed, retryAfter, err := f.redis.AllowLogin(clientIP)
		if err != nil {
			// Redis failed, switch to memory
			f.switchToMemory()
			return f.memory.AllowLogin(clientIP)
		}
		return allowed, retryAfter, nil
	}

	// Check if Redis is back online (every 30 seconds)
	if time.Since(f.lastCheck) > 30*time.Second {
		f.checkRedisHealth()
	}

	return f.memory.AllowLogin(clientIP)
}

// AllowRegister checks if a registration attempt is allowed
func (f *FallbackRateLimiter) AllowRegister(clientIP string) (bool, time.Duration, error) {
	f.mu.RLock()
	usingRedis := f.usingRedis
	f.mu.RUnlock()

	if usingRedis {
		allowed, retryAfter, err := f.redis.AllowRegister(clientIP)
		if err != nil {
			// Redis failed, switch to memory
			f.switchToMemory()
			return f.memory.AllowRegister(clientIP)
		}
		return allowed, retryAfter, nil
	}

	// Check if Redis is back online (every 30 seconds)
	if time.Since(f.lastCheck) > 30*time.Second {
		f.checkRedisHealth()
	}

	return f.memory.AllowRegister(clientIP)
}

// switchToMemory switches from Redis to in-memory rate limiting
func (f *FallbackRateLimiter) switchToMemory() {
	f.mu.Lock()
	defer f.mu.Unlock()

	if f.usingRedis {
		fmt.Println("WARNING: Switching to in-memory rate limiter due to Redis failure")
		f.usingRedis = false
		f.lastCheck = time.Now()

		// Update metrics
		if f.metrics != nil {
			f.metrics.SetRateLimiterMode(true) // true = using memory
		}
	}
}

// checkRedisHealth checks if Redis is back online and switches back if available
func (f *FallbackRateLimiter) checkRedisHealth() {
	f.mu.Lock()
	defer f.mu.Unlock()

	f.lastCheck = time.Now()

	// Only check if we're currently using memory
	if !f.usingRedis && f.redis != nil {
		// Try to ping Redis
		ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
		defer cancel()

		if err := f.redis.client.Ping(ctx).Err(); err == nil {
			fmt.Println("INFO: Redis is back online, switching from in-memory rate limiter")
			f.usingRedis = true

			// Update metrics
			if f.metrics != nil {
				f.metrics.SetRateLimiterMode(false) // false = using Redis
			}
		}
	}
}

// IsUsingRedis returns true if currently using Redis, false if using in-memory fallback
func (f *FallbackRateLimiter) IsUsingRedis() bool {
	f.mu.RLock()
	defer f.mu.RUnlock()
	return f.usingRedis
}

// Cleanup performs cleanup on the active rate limiter
func (f *FallbackRateLimiter) Cleanup() int {
	f.mu.RLock()
	usingRedis := f.usingRedis
	f.mu.RUnlock()

	if !usingRedis {
		return f.memory.Cleanup()
	}
	return 0
}

// GetTrackedClients returns the number of clients being tracked
func (f *FallbackRateLimiter) GetTrackedClients() int {
	f.mu.RLock()
	usingRedis := f.usingRedis
	f.mu.RUnlock()

	if !usingRedis {
		return f.memory.GetTrackedClients()
	}
	return 0 // Redis doesn't expose this easily
}

// Close closes both rate limiters
func (f *FallbackRateLimiter) Close() error {
	var err error
	if f.redis != nil {
		err = f.redis.Close()
	}
	if f.memory != nil {
		memErr := f.memory.Close()
		if err == nil {
			err = memErr
		}
	}
	return err
}
