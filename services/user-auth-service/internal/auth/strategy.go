package auth

import "context"

// AuthenticationStrategy defines the interface that all authentication methods must implement.
// This interface enables the strategy pattern, allowing different authentication methods
// (normal, OAuth, SAML) to be used interchangeably based on configuration.
//
// Example usage:
//
//	// Create a strategy manager
//	manager := NewStrategyManager(config, dependencies...)
//
//	// Get the appropriate strategy based on auth type
//	strategy, err := manager.GetStrategy(AuthTypeOAuth)
//	if err != nil {
//	    return err
//	}
//
//	// Initiate authentication
//	result, err := strategy.Authenticate(ctx, &AuthRequest{
//	    Provider: "google",
//	})
//	if err != nil {
//	    return err
//	}
//
//	// For OAuth/SAML, redirect user to authorization URL
//	if !result.Success {
//	    // Redirect to result.AuthorizationURL or result.SSOURL
//	}
//
//	// Later, handle the callback
//	result, err = strategy.HandleCallback(ctx, &CallbackRequest{
//	    Code: "auth_code",
//	    State: "csrf_token",
//	})
type AuthenticationStrategy interface {
	// Authenticate initiates the authentication process for the given request.
	// For normal authentication, this validates credentials and returns tokens immediately.
	// For OAuth/SAML, this returns the authorization URL where the user should be redirected.
	//
	// Parameters:
	//   - ctx: Context for cancellation and tracing
	//   - req: Authentication request containing credentials or provider information
	//
	// Returns:
	//   - AuthResult with Success=true and tokens for immediate authentication (normal)
	//   - AuthResult with Success=false and redirect URL for deferred authentication (OAuth/SAML)
	//   - Error if the authentication request is invalid or fails
	Authenticate(ctx context.Context, req *AuthRequest) (*AuthResult, error)

	// HandleCallback processes authentication callbacks from external providers.
	// This method is used for OAuth and SAML flows after the user has been redirected
	// back from the external provider.
	//
	// Parameters:
	//   - ctx: Context for cancellation and tracing
	//   - req: Callback request containing authorization code, state, or SAML response
	//
	// Returns:
	//   - AuthResult with Success=true, user information, and JWT tokens
	//   - Error if the callback is invalid, expired, or authentication fails
	//
	// Note: This method returns an error for normal authentication as it doesn't use callbacks.
	HandleCallback(ctx context.Context, req *CallbackRequest) (*AuthResult, error)

	// GetType returns the authentication type that this strategy implements.
	// This is used by the strategy manager to register and retrieve strategies.
	//
	// Returns:
	//   - AuthType constant (AuthTypeNormal, AuthTypeOAuth, or AuthTypeSAML)
	GetType() AuthType

	// ValidateConfig validates that the strategy has all required configuration
	// and dependencies to function properly. This is called during strategy registration.
	//
	// Returns:
	//   - nil if the configuration is valid
	//   - Error describing what configuration is missing or invalid
	ValidateConfig() error
}
