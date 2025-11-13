package service

import "github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/models"

// UserRepositoryInterface defines the interface for user repository operations
type UserRepositoryInterface interface {
	Create(user *models.User) error
	GetByID(id string) (*models.User, error)
	GetByEmail(email string) (*models.User, error)
	Update(user *models.User) error
	Delete(id string) error
	List(page, pageSize int, search, role string, isActive *bool) ([]*models.User, int, error)
	UpdatePassword(userID, passwordHash string) error
}

// RoleRepositoryInterface defines the interface for role repository operations
type RoleRepositoryInterface interface {
	AssignRoleByName(userID, roleName string) error
	RemoveRoleByName(userID, roleName string) error
	GetUserRoles(userID string) ([]string, error)
	CheckPermission(userID, permission string) (bool, error)
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
	UserID string
	Email  string
	Roles  []string
}
