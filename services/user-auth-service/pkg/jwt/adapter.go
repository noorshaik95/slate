package jwt

import "slate/services/user-auth-service/internal/service"

// TokenServiceAdapter adapts TokenService to implement service.TokenServiceInterface
type TokenServiceAdapter struct {
	*TokenService
}

// NewTokenServiceAdapter creates a new adapter
func NewTokenServiceAdapter(ts *TokenService) *TokenServiceAdapter {
	return &TokenServiceAdapter{TokenService: ts}
}

// ValidateAccessToken implements service.TokenServiceInterface
func (a *TokenServiceAdapter) ValidateAccessToken(token string) (*service.TokenClaims, error) {
	claims, err := a.TokenService.ValidateAccessToken(token)
	if err != nil {
		return nil, err
	}

	return &service.TokenClaims{
		UserID: claims.UserID,
		Email:  claims.Email,
		Roles:  claims.Roles,
		IssuedAt: service.TimeWrapper{
			Time: claims.IssuedAt.Time,
		},
		ExpiresAt: service.TimeWrapper{
			Time: claims.ExpiresAt.Time,
		},
	}, nil
}
