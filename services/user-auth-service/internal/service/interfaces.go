package service

import (
	"context"
	"time"

	"slate/services/user-auth-service/internal/models"
)

// UserRepositoryInterface defines the interface for user repository operations
type UserRepositoryInterface interface {
	Create(ctx context.Context, user *models.User) error
	GetByID(ctx context.Context, id string) (*models.User, error)
	GetByEmail(ctx context.Context, email string) (*models.User, error)
	Update(ctx context.Context, user *models.User) error
	Delete(ctx context.Context, id string) error
	List(ctx context.Context, page, pageSize int, search, role string, isActive *bool) ([]*models.User, int, error)
	UpdatePassword(ctx context.Context, userID, passwordHash string) error
}

// RoleRepositoryInterface defines the interface for role repository operations
type RoleRepositoryInterface interface {
	AssignRoleByName(ctx context.Context, userID, roleName string) error
	RemoveRoleByName(ctx context.Context, userID, roleName string) error
	GetUserRoles(ctx context.Context, userID string) ([]string, error)
	CheckPermission(ctx context.Context, userID, permission string) (bool, error)
}

// TokenServiceInterface defines the interface for token service operations
type TokenServiceInterface interface {
	GenerateAccessToken(userID, email string, roles []string) (string, int64, error)
	GenerateRefreshToken(userID, email string, roles []string) (string, error)
	ValidateAccessToken(token string) (*TokenClaims, error)
	RefreshAccessToken(refreshToken string) (string, string, int64, error)
}

// TokenClaims represents the claims in a JWT token
type TokenClaims struct {
	UserID    string
	Email     string
	Roles     []string
	IssuedAt  TimeWrapper
	ExpiresAt TimeWrapper
}

// TimeWrapper wraps time.Time for interface compatibility
type TimeWrapper struct {
	Time time.Time
}

// TokenBlacklistInterface defines the interface for token blacklist operations
type TokenBlacklistInterface interface {
	BlacklistToken(ctx context.Context, token string, expiresAt time.Time) error
	BlacklistUserTokens(ctx context.Context, userID string, maxTokenLifetime time.Duration) error
	IsTokenBlacklisted(ctx context.Context, token string, userID string, issuedAt time.Time) (bool, error)
}

// MetricsInterface defines the interface for metrics operations
type MetricsInterface interface {
	IncrementRegistrations(success bool)
	IncrementLogins(success bool)
	ObserveRequestDuration(operation string, durationSeconds float64)
	SetDBConnections(count int)
}
