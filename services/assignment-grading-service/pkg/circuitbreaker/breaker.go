package circuitbreaker

import (
	"time"

	"github.com/sony/gobreaker"
)

// CircuitBreaker wraps the gobreaker circuit breaker
type CircuitBreaker struct {
	breaker *gobreaker.CircuitBreaker
}

// Settings for circuit breaker configuration
type Settings struct {
	Name          string
	MaxRequests   uint32        // Max requests allowed in half-open state
	Interval      time.Duration // Interval to clear request counts
	Timeout       time.Duration // Timeout to stay open before half-open
	ReadyToTrip   func(counts gobreaker.Counts) bool
	OnStateChange func(name string, from gobreaker.State, to gobreaker.State)
}

// NewCircuitBreaker creates a new circuit breaker with the given settings
func NewCircuitBreaker(settings Settings) *CircuitBreaker {
	cbSettings := gobreaker.Settings{
		Name:          settings.Name,
		MaxRequests:   settings.MaxRequests,
		Interval:      settings.Interval,
		Timeout:       settings.Timeout,
		ReadyToTrip:   settings.ReadyToTrip,
		OnStateChange: settings.OnStateChange,
	}

	return &CircuitBreaker{
		breaker: gobreaker.NewCircuitBreaker(cbSettings),
	}
}

// NewDefaultCircuitBreaker creates a circuit breaker with default settings
// Default: opens after 5 consecutive failures, half-open timeout is 60s
func NewDefaultCircuitBreaker(name string) *CircuitBreaker {
	return NewCircuitBreaker(Settings{
		Name:        name,
		MaxRequests: 3,               // Allow 3 requests in half-open state
		Interval:    2 * time.Minute, // Clear counts every 2 minutes
		Timeout:     60 * time.Second, // Wait 60s before trying half-open
		ReadyToTrip: func(counts gobreaker.Counts) bool {
			failureRatio := float64(counts.TotalFailures) / float64(counts.Requests)
			return counts.Requests >= 3 && failureRatio >= 0.6
		},
	})
}

// Execute runs the given function with circuit breaker protection
func (cb *CircuitBreaker) Execute(fn func() (interface{}, error)) (interface{}, error) {
	return cb.breaker.Execute(fn)
}

// State returns the current state of the circuit breaker
func (cb *CircuitBreaker) State() gobreaker.State {
	return cb.breaker.State()
}

// Name returns the name of the circuit breaker
func (cb *CircuitBreaker) Name() string {
	return cb.breaker.Name()
}
