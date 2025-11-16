package service

import (
	"context"

	"slate/services/user-auth-service/internal/auth"
)

// StrategyManagerAdapter adapts auth.StrategyManager to service.StrategyManagerInterface
// This adapter bridges the gap between the auth and service packages to avoid import cycles
type StrategyManagerAdapter struct {
	manager *auth.StrategyManager
}

// NewStrategyManagerAdapter creates a new adapter for auth.StrategyManager
func NewStrategyManagerAdapter(manager *auth.StrategyManager) *StrategyManagerAdapter {
	return &StrategyManagerAdapter{
		manager: manager,
	}
}

// GetActiveAuthType returns the configured primary authentication type
func (a *StrategyManagerAdapter) GetActiveAuthType() AuthType {
	authType := a.manager.GetActiveAuthType()
	return AuthType(authType)
}

// GetStrategy retrieves the authentication strategy for the specified auth type
func (a *StrategyManagerAdapter) GetStrategy(authType AuthType) (AuthenticationStrategyInterface, error) {
	strategy, err := a.manager.GetStrategy(auth.AuthType(authType))
	if err != nil {
		return nil, err
	}
	return &AuthenticationStrategyAdapter{strategy: strategy}, nil
}

// AuthenticationStrategyAdapter adapts auth.AuthenticationStrategy to service.AuthenticationStrategyInterface
type AuthenticationStrategyAdapter struct {
	strategy auth.AuthenticationStrategy
}

// Authenticate initiates the authentication process
func (a *AuthenticationStrategyAdapter) Authenticate(ctx context.Context, req *AuthRequest) (*AuthResult, error) {
	authReq := &auth.AuthRequest{
		Email:          req.Email,
		Password:       req.Password,
		OrganizationID: req.OrganizationID,
		Provider:       req.Provider,
	}

	result, err := a.strategy.Authenticate(ctx, authReq)
	if err != nil {
		return nil, err
	}

	return &AuthResult{
		Success:          result.Success,
		User:             result.User,
		Tokens:           result.Tokens,
		AuthorizationURL: result.AuthorizationURL,
		State:            result.State,
		SAMLRequest:      result.SAMLRequest,
		SSOURL:           result.SSOURL,
	}, nil
}

// HandleCallback processes authentication callbacks
func (a *AuthenticationStrategyAdapter) HandleCallback(ctx context.Context, req *CallbackRequest) (*AuthResult, error) {
	callbackReq := &auth.CallbackRequest{
		Code:         req.Code,
		State:        req.State,
		SAMLResponse: req.SAMLResponse,
	}

	result, err := a.strategy.HandleCallback(ctx, callbackReq)
	if err != nil {
		return nil, err
	}

	return &AuthResult{
		Success:          result.Success,
		User:             result.User,
		Tokens:           result.Tokens,
		AuthorizationURL: result.AuthorizationURL,
		State:            result.State,
		SAMLRequest:      result.SAMLRequest,
		SSOURL:           result.SSOURL,
	}, nil
}

// GetType returns the authentication type
func (a *AuthenticationStrategyAdapter) GetType() AuthType {
	authType := a.strategy.GetType()
	return AuthType(authType)
}

// ValidateConfig validates the strategy configuration
func (a *AuthenticationStrategyAdapter) ValidateConfig() error {
	return a.strategy.ValidateConfig()
}
