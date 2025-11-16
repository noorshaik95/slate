package auth

import (
	"fmt"

	"slate/services/user-auth-service/internal/config"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel/trace"
)

// StrategyManager manages authentication strategies and provides access to them
// based on the configured authentication type. It acts as a registry for all
// available authentication strategies and handles their lifecycle.
type StrategyManager struct {
	// strategies maps authentication types to their corresponding strategy implementations
	strategies map[AuthType]AuthenticationStrategy

	// config contains the full application configuration
	config *config.Config

	// tracer is used for distributed tracing of authentication operations
	tracer trace.Tracer

	// logger is used for structured logging of authentication events
	logger *logger.Logger
}

// NewStrategyManager creates a new strategy manager with the provided dependencies.
// The manager is initialized with an empty strategies map and must have strategies
// registered using RegisterStrategy before they can be used.
//
// Parameters:
//   - config: Full application configuration including auth, OAuth, and SAML settings
//   - tracer: OpenTelemetry tracer for distributed tracing
//   - logger: Structured logger for authentication events
//
// Returns:
//   - Initialized StrategyManager ready for strategy registration
func NewStrategyManager(
	config *config.Config,
	tracer trace.Tracer,
	logger *logger.Logger,
) *StrategyManager {
	return &StrategyManager{
		strategies: make(map[AuthType]AuthenticationStrategy),
		config:     config,
		tracer:     tracer,
		logger:     logger,
	}
}

// RegisterStrategy registers an authentication strategy with the manager.
// The strategy's configuration is validated before registration. If a strategy
// for the same auth type is already registered, an error is returned.
//
// Parameters:
//   - strategy: The authentication strategy to register
//
// Returns:
//   - nil if registration is successful
//   - Error if validation fails or a strategy for this type already exists
func (m *StrategyManager) RegisterStrategy(strategy AuthenticationStrategy) error {
	// Validate strategy configuration
	if err := strategy.ValidateConfig(); err != nil {
		return fmt.Errorf("strategy validation failed for type %s: %w", strategy.GetType(), err)
	}

	// Get strategy type
	strategyType := strategy.GetType()

	// Check if strategy already registered
	if _, exists := m.strategies[strategyType]; exists {
		return fmt.Errorf("strategy already registered for type: %s", strategyType)
	}

	// Store strategy in map
	m.strategies[strategyType] = strategy

	// Log strategy registration
	m.logger.Info().
		Str("strategy_type", string(strategyType)).
		Msg("Authentication strategy registered")

	return nil
}

// GetStrategy retrieves the authentication strategy for the specified auth type.
// If no strategy is registered for the given type, an error is returned.
//
// Parameters:
//   - authType: The authentication type to retrieve (AuthTypeNormal, AuthTypeOAuth, or AuthTypeSAML)
//
// Returns:
//   - The registered authentication strategy for the given type
//   - Error if no strategy is registered for the specified type
func (m *StrategyManager) GetStrategy(authType AuthType) (AuthenticationStrategy, error) {
	strategy, exists := m.strategies[authType]
	if !exists {
		return nil, fmt.Errorf("no strategy registered for auth type: %s", authType)
	}
	return strategy, nil
}

// GetActiveAuthType returns the configured primary authentication type from configuration.
// This indicates which authentication method is the default for the system.
//
// Returns:
//   - The configured authentication type (normal, oauth, or saml)
func (m *StrategyManager) GetActiveAuthType() AuthType {
	return AuthType(m.config.Auth.Type)
}

// Note: Strategy initialization is implemented in cmd/server/main.go in the
// initializeAuthStrategies function. This avoids import cycles while providing
// a centralized place to initialize all authentication strategies based on
// the application configuration.
