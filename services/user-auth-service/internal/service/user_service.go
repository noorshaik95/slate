package service

import (
	"fmt"
	"time"

	"github.com/noorshaik95/axum-grafana-example/services/user-auth-service/internal/models"
	"golang.org/x/crypto/bcrypt"
)

type UserService struct {
	userRepo UserRepositoryInterface
	roleRepo RoleRepositoryInterface
	tokenSvc TokenServiceInterface
}

func NewUserService(userRepo UserRepositoryInterface, roleRepo RoleRepositoryInterface, tokenSvc TokenServiceInterface) *UserService {
	return &UserService{
		userRepo: userRepo,
		roleRepo: roleRepo,
		tokenSvc: tokenSvc,
	}
}

// Register registers a new user
func (s *UserService) Register(email, password, firstName, lastName, phone string) (*models.User, *models.TokenPair, error) {
	// Check if user already exists
	existingUser, _ := s.userRepo.GetByEmail(email)
	if existingUser != nil {
		return nil, nil, fmt.Errorf("user with email %s already exists", email)
	}

	// Hash password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to hash password: %w", err)
	}

	// Create user
	user := models.NewUser(email, string(hashedPassword), firstName, lastName, phone)
	if err := s.userRepo.Create(user); err != nil {
		return nil, nil, err
	}

	// Assign default "user" role
	if err := s.roleRepo.AssignRoleByName(user.ID, "user"); err != nil {
		// Log error but don't fail registration
		fmt.Printf("Warning: failed to assign default role: %v\n", err)
	}

	// Reload user to get roles
	user, err = s.userRepo.GetByID(user.ID)
	if err != nil {
		return nil, nil, err
	}

	// Generate tokens
	tokens, err := s.generateTokens(user)
	if err != nil {
		return nil, nil, err
	}

	return user, tokens, nil
}

// Login authenticates a user
func (s *UserService) Login(email, password string) (*models.User, *models.TokenPair, error) {
	user, err := s.userRepo.GetByEmail(email)
	if err != nil {
		return nil, nil, fmt.Errorf("invalid credentials")
	}

	if !user.IsActive {
		return nil, nil, fmt.Errorf("user account is inactive")
	}

	// Verify password
	if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(password)); err != nil {
		return nil, nil, fmt.Errorf("invalid credentials")
	}

	// Generate tokens
	tokens, err := s.generateTokens(user)
	if err != nil {
		return nil, nil, err
	}

	return user, tokens, nil
}

// ValidateToken validates a token and returns user info
func (s *UserService) ValidateToken(token string) (string, []string, error) {
	claims, err := s.tokenSvc.ValidateAccessToken(token)
	if err != nil {
		return "", nil, err
	}

	return claims.UserID, claims.Roles, nil
}

// RefreshToken refreshes an access token
func (s *UserService) RefreshToken(refreshToken string) (*models.TokenPair, error) {
	accessToken, newRefreshToken, expiresIn, err := s.tokenSvc.RefreshAccessToken(refreshToken)
	if err != nil {
		return nil, err
	}

	return &models.TokenPair{
		AccessToken:  accessToken,
		RefreshToken: newRefreshToken,
		ExpiresIn:    expiresIn,
	}, nil
}

// CreateUser creates a new user (admin operation)
func (s *UserService) CreateUser(email, password, firstName, lastName, phone string, roles []string) (*models.User, error) {
	// Check if user already exists
	existingUser, _ := s.userRepo.GetByEmail(email)
	if existingUser != nil {
		return nil, fmt.Errorf("user with email %s already exists", email)
	}

	// Hash password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, fmt.Errorf("failed to hash password: %w", err)
	}

	// Create user
	user := models.NewUser(email, string(hashedPassword), firstName, lastName, phone)
	if err := s.userRepo.Create(user); err != nil {
		return nil, err
	}

	// Assign roles
	for _, role := range roles {
		if err := s.roleRepo.AssignRoleByName(user.ID, role); err != nil {
			fmt.Printf("Warning: failed to assign role %s: %v\n", role, err)
		}
	}

	// Reload user to get roles
	return s.userRepo.GetByID(user.ID)
}

// GetUser retrieves a user by ID
func (s *UserService) GetUser(userID string) (*models.User, error) {
	return s.userRepo.GetByID(userID)
}

// UpdateUser updates a user
func (s *UserService) UpdateUser(userID string, email, firstName, lastName, phone *string, isActive *bool) (*models.User, error) {
	user, err := s.userRepo.GetByID(userID)
	if err != nil {
		return nil, err
	}

	// Update fields if provided
	if email != nil {
		user.Email = *email
	}
	if firstName != nil {
		user.FirstName = *firstName
	}
	if lastName != nil {
		user.LastName = *lastName
	}
	if phone != nil {
		user.Phone = *phone
	}
	if isActive != nil {
		user.IsActive = *isActive
	}

	user.UpdatedAt = time.Now()

	if err := s.userRepo.Update(user); err != nil {
		return nil, err
	}

	return s.userRepo.GetByID(userID)
}

// DeleteUser deletes a user (soft delete)
func (s *UserService) DeleteUser(userID string) error {
	return s.userRepo.Delete(userID)
}

// ListUsers retrieves a paginated list of users
func (s *UserService) ListUsers(page, pageSize int, search, role string, isActive *bool) ([]*models.User, int, error) {
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 10
	}

	return s.userRepo.List(page, pageSize, search, role, isActive)
}

// GetProfile retrieves a user's profile
func (s *UserService) GetProfile(userID string) (*models.Profile, error) {
	user, err := s.userRepo.GetByID(userID)
	if err != nil {
		return nil, err
	}

	return user.ToProfile(), nil
}

// UpdateProfile updates a user's profile
func (s *UserService) UpdateProfile(userID string, firstName, lastName, phone, avatarURL, bio *string) (*models.Profile, error) {
	user, err := s.userRepo.GetByID(userID)
	if err != nil {
		return nil, err
	}

	if firstName != nil {
		user.FirstName = *firstName
	}
	if lastName != nil {
		user.LastName = *lastName
	}
	if phone != nil {
		user.Phone = *phone
	}
	// Note: avatarURL and bio would need additional fields in the User model/table

	user.UpdatedAt = time.Now()

	if err := s.userRepo.Update(user); err != nil {
		return nil, err
	}

	user, err = s.userRepo.GetByID(userID)
	if err != nil {
		return nil, err
	}

	return user.ToProfile(), nil
}

// ChangePassword changes a user's password
func (s *UserService) ChangePassword(userID, oldPassword, newPassword string) error {
	user, err := s.userRepo.GetByID(userID)
	if err != nil {
		return err
	}

	// Verify old password
	if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(oldPassword)); err != nil {
		return fmt.Errorf("invalid old password")
	}

	// Hash new password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(newPassword), bcrypt.DefaultCost)
	if err != nil {
		return fmt.Errorf("failed to hash password: %w", err)
	}

	return s.userRepo.UpdatePassword(userID, string(hashedPassword))
}

// AssignRole assigns a role to a user
func (s *UserService) AssignRole(userID, role string) error {
	// Check if user exists
	if _, err := s.userRepo.GetByID(userID); err != nil {
		return err
	}

	return s.roleRepo.AssignRoleByName(userID, role)
}

// RemoveRole removes a role from a user
func (s *UserService) RemoveRole(userID, role string) error {
	return s.roleRepo.RemoveRoleByName(userID, role)
}

// GetUserRoles retrieves all roles for a user
func (s *UserService) GetUserRoles(userID string) ([]string, error) {
	return s.roleRepo.GetUserRoles(userID)
}

// CheckPermission checks if a user has a specific permission
func (s *UserService) CheckPermission(userID, permission string) (bool, error) {
	return s.roleRepo.CheckPermission(userID, permission)
}

// generateTokens generates access and refresh tokens for a user
func (s *UserService) generateTokens(user *models.User) (*models.TokenPair, error) {
	accessToken, expiresIn, err := s.tokenSvc.GenerateAccessToken(user.ID, user.Email, user.Roles)
	if err != nil {
		return nil, err
	}

	refreshToken, err := s.tokenSvc.GenerateRefreshToken(user.ID, user.Email, user.Roles)
	if err != nil {
		return nil, err
	}

	return &models.TokenPair{
		AccessToken:  accessToken,
		RefreshToken: refreshToken,
		ExpiresIn:    expiresIn,
	}, nil
}
