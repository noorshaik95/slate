package ratelimit

import (
	"sync"
	"time"
)

const (
	actionLogin = "login"
)

// MemoryRateLimiter implements rate limiting using in-memory storage
// This is used as a fallback when Redis is unavailable
type MemoryRateLimiter struct {
	mu            sync.RWMutex
	clients       map[string]*ClientState
	loginLimit    RateLimit
	registerLimit RateLimit
}

// ClientState tracks rate limit state for a single client
type ClientState struct {
	attempts []time.Time
	mu       sync.Mutex
}

// NewMemoryRateLimiter creates a new in-memory rate limiter
func NewMemoryRateLimiter(loginLimit, registerLimit RateLimit) *MemoryRateLimiter {
	return &MemoryRateLimiter{
		clients:       make(map[string]*ClientState),
		loginLimit:    loginLimit,
		registerLimit: registerLimit,
	}
}

// AllowLogin checks if a login attempt is allowed for the given client IP
func (m *MemoryRateLimiter) AllowLogin(clientIP string) (bool, time.Duration, error) {
	return m.checkLimit(clientIP, actionLogin, m.loginLimit)
}

// AllowRegister checks if a registration attempt is allowed for the given client IP
func (m *MemoryRateLimiter) AllowRegister(clientIP string) (bool, time.Duration, error) {
	return m.checkLimit(clientIP, "register", m.registerLimit)
}

// checkLimit checks if an action is allowed based on the rate limit
func (m *MemoryRateLimiter) checkLimit(clientIP, action string, limit RateLimit) (bool, time.Duration, error) {
	key := clientIP + ":" + action
	now := time.Now()

	// Get or create client state
	m.mu.Lock()
	state, exists := m.clients[key]
	if !exists {
		state = &ClientState{
			attempts: make([]time.Time, 0, limit.MaxAttempts),
		}
		m.clients[key] = state
	}
	m.mu.Unlock()

	// Lock the specific client state
	state.mu.Lock()
	defer state.mu.Unlock()

	// Remove expired attempts
	cutoff := now.Add(-limit.Window)
	validAttempts := make([]time.Time, 0, len(state.attempts))
	for _, attempt := range state.attempts {
		if attempt.After(cutoff) {
			validAttempts = append(validAttempts, attempt)
		}
	}
	state.attempts = validAttempts

	// Check if limit is exceeded
	if len(state.attempts) >= limit.MaxAttempts {
		// Calculate retry-after duration
		oldestAttempt := state.attempts[0]
		retryAfter := limit.Window - now.Sub(oldestAttempt)
		if retryAfter < 0 {
			retryAfter = 0
		}
		return false, retryAfter, nil
	}

	// Add current attempt
	state.attempts = append(state.attempts, now)
	return true, 0, nil
}

// Cleanup removes expired entries from memory
// This should be called periodically to prevent memory leaks
func (m *MemoryRateLimiter) Cleanup() int {
	m.mu.Lock()
	defer m.mu.Unlock()

	now := time.Now()
	removed := 0

	// Find keys to remove
	keysToRemove := make([]string, 0)
	for key, state := range m.clients {
		state.mu.Lock()

		// Determine which limit applies based on key suffix
		var window time.Duration
		if len(state.attempts) > 0 {
			if key[len(key)-5:] == actionLogin {
				window = m.loginLimit.Window
			} else {
				window = m.registerLimit.Window
			}

			// Check if all attempts are expired
			cutoff := now.Add(-window)
			allExpired := true
			for _, attempt := range state.attempts {
				if attempt.After(cutoff) {
					allExpired = false
					break
				}
			}

			if allExpired {
				keysToRemove = append(keysToRemove, key)
			}
		} else if len(state.attempts) == 0 {
			// Remove empty states
			keysToRemove = append(keysToRemove, key)
		}

		state.mu.Unlock()
	}

	// Remove expired keys
	for _, key := range keysToRemove {
		delete(m.clients, key)
		removed++
	}

	return removed
}

// GetTrackedClients returns the number of clients currently being tracked
func (m *MemoryRateLimiter) GetTrackedClients() int {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return len(m.clients)
}

// Close cleans up resources (no-op for memory limiter)
func (m *MemoryRateLimiter) Close() error {
	return nil
}
