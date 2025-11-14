package service

import (
	"context"
	"fmt"
	"time"

	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/pkg/logger"
	"slate/services/user-auth-service/pkg/validation"

	"golang.org/x/crypto/bcrypt"
)

type UserService struct {
	userRepo       UserRepositoryInterface
	roleRepo       RoleRepositoryInterface
	tokenSvc       TokenServiceInterface
	tokenBlacklist TokenBlacklistInterface
	validator      *validation.Validator
	logger         *logger.Logger
	metrics        MetricsInterface
}

func NewUserService(userRepo UserRepositoryInterface, roleRepo RoleRepositoryInterface, tokenSvc TokenServiceInterface, tokenBlacklist TokenBlacklistInterface, log *logger.Logger, metrics MetricsInterface) *UserService {
	return &UserService{
		userRepo:       userRepo,
		roleRepo:       roleRepo,
		tokenSvc:       tokenSvc,
		tokenBlacklist: tokenBlacklist,
		validator:      validation.NewValidator(),
		logger:         log,
		metrics:        metrics,
	}
}

// Register registers a new user
func (s *UserService) Register(ctx context.Context, email, password, firstName, lastName, phone string) (*models.User, *models.TokenPair, error) {
	start := time.Now()
	var success bool
	defer func() {
		s.metrics.IncrementRegistrations(success)
		s.metrics.ObserveRequestDuration("register", time.Since(start).Seconds())
	}()

	// Validate email
	if err := s.validator.ValidateEmail(email); err != nil {
		return nil, nil, err
	}

	// Validate password
	if err := s.validator.ValidatePassword(password); err != nil {
		return nil, nil, err
	}

	// Sanitize and validate first name
	sanitizedFirstName, err := s.validator.SanitizeName(firstName)
	if err != nil {
		return nil, nil, err
	}

	// Sanitize and validate last name
	sanitizedLastName, err := s.validator.SanitizeName(lastName)
	if err != nil {
		return nil, nil, err
	}

	// Validate phone (optional field)
	if err := s.validator.ValidatePhone(phone); err != nil {
		return nil, nil, err
	}

	// Check if user already exists
	existingUser, _ := s.userRepo.GetByEmail(ctx, email)
	if existingUser != nil {
		return nil, nil, fmt.Errorf("user with email %s already exists", email)
	}

	// Hash password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to hash password: %w", err)
	}

	// Create user with sanitized names
	user := models.NewUser(email, string(hashedPassword), sanitizedFirstName, sanitizedLastName, phone)
	if err := s.userRepo.Create(ctx, user); err != nil {
		return nil, nil, err
	}

	// Assign default "user" role
	if err := s.roleRepo.AssignRoleByName(ctx, user.ID, "user"); err != nil {
		// Log error but don't fail registration
		s.logger.Warn().
			Str("user_id", user.ID).
			Str("operation", "register").
			Str("error_type", "role_assignment_failed").
			Err(err).
			Msg("failed to assign default role")
	}

	// Reload user to get roles
	user, err = s.userRepo.GetByID(ctx, user.ID)
	if err != nil {
		s.logger.Error().
			Str("user_id", user.ID).
			Str("operation", "register").
			Str("error_type", "user_reload_failed").
			Err(err).
			Msg("failed to reload user after registration")
		return nil, nil, err
	}

	// Generate tokens
	tokens, err := s.generateTokens(user)
	if err != nil {
		s.logger.Error().
			Str("user_id", user.ID).
			Str("operation", "register").
			Str("error_type", "token_generation_failed").
			Err(err).
			Msg("failed to generate tokens")
		return nil, nil, err
	}

	s.logger.Info().
		Str("user_id", user.ID).
		Str("email", s.logger.RedactEmail(user.Email)).
		Str("operation", "register").
		Msg("user registered successfully")

	success = true
	return user, tokens, nil
}

// Login authenticates a user
func (s *UserService) Login(ctx context.Context, email, password string) (*models.User, *models.TokenPair, error) {
	start := time.Now()
	var success bool
	defer func() {
		s.metrics.IncrementLogins(success)
		s.metrics.ObserveRequestDuration("login", time.Since(start).Seconds())
	}()

	user, err := s.userRepo.GetByEmail(ctx, email)
	if err != nil {
		// Security: Return generic "invalid credentials" error to prevent user enumeration.
		// This prevents attackers from determining which email addresses are registered.
		// Detailed error is logged server-side for debugging but not exposed to client.
		s.logger.Warn().
			Str("email", s.logger.RedactEmail(email)).
			Str("operation", "login").
			Str("error_type", "user_not_found").
			Msg("login attempt for non-existent user")
		return nil, nil, fmt.Errorf("invalid credentials")
	}

	if !user.IsActive {
		s.logger.Warn().
			Str("user_id", user.ID).
			Str("email", s.logger.RedactEmail(email)).
			Str("operation", "login").
			Str("error_type", "inactive_account").
			Msg("login attempt for inactive account")
		return nil, nil, fmt.Errorf("user account is inactive")
	}

	// Security: Use bcrypt.CompareHashAndPassword for timing-safe password comparison.
	// This prevents timing attacks where an attacker could determine password correctness
	// by measuring response time differences. bcrypt's comparison is constant-time.
	if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(password)); err != nil {
		// Security: Return the same generic "invalid credentials" error as user-not-found case.
		// This prevents attackers from distinguishing between "user doesn't exist" and "wrong password",
		// which would allow user enumeration attacks.
		s.logger.Warn().
			Str("user_id", user.ID).
			Str("email", s.logger.RedactEmail(email)).
			Str("operation", "login").
			Str("error_type", "invalid_password").
			Msg("login attempt with invalid password")
		return nil, nil, fmt.Errorf("invalid credentials")
	}

	// Generate tokens
	tokens, err := s.generateTokens(user)
	if err != nil {
		s.logger.Error().
			Str("user_id", user.ID).
			Str("operation", "login").
			Str("error_type", "token_generation_failed").
			Err(err).
			Msg("failed to generate tokens")
		return nil, nil, err
	}

	s.logger.Info().
		Str("user_id", user.ID).
		Str("email", s.logger.RedactEmail(user.Email)).
		Str("operation", "login").
		Msg("user logged in successfully")

	success = true
	return user, tokens, nil
}

// Logout invalidates a user's access token by adding it to the blacklist
func (s *UserService) Logout(ctx context.Context, token string) error {
	// Validate the token to get claims (including expiration)
	claims, err := s.tokenSvc.ValidateAccessToken(token)
	if err != nil {
		// Token is already invalid, no need to blacklist
		s.logger.Warn().
			Str("operation", "logout").
			Str("error_type", "invalid_token").
			Err(err).
			Msg("logout attempt with invalid token")
		return nil
	}

	// Add token to blacklist with TTL matching token expiration
	if s.tokenBlacklist != nil {
		expiresAt := claims.ExpiresAt.Time
		err = s.tokenBlacklist.BlacklistToken(ctx, token, expiresAt)
		if err != nil {
			// Log error but don't fail logout (fail-open for logout)
			s.logger.Error().
				Str("user_id", claims.UserID).
				Str("operation", "logout").
				Str("error_type", "blacklist_failed").
				Err(err).
				Msg("failed to blacklist token, but allowing logout to proceed")
		} else {
			s.logger.Info().
				Str("user_id", claims.UserID).
				Str("operation", "logout").
				Msg("token blacklisted successfully")
		}
	}

	s.logger.Info().
		Str("user_id", claims.UserID).
		Str("operation", "logout").
		Msg("user logged out successfully")

	return nil
}

// ValidateToken validates a token and returns user info
func (s *UserService) ValidateToken(ctx context.Context, token string) (string, []string, error) {
	claims, err := s.tokenSvc.ValidateAccessToken(token)
	if err != nil {
		return "", nil, err
	}

	// Check if token is blacklisted
	if s.tokenBlacklist != nil {
		isBlacklisted, err := s.tokenBlacklist.IsTokenBlacklisted(ctx, token, claims.UserID, claims.IssuedAt.Time)
		if err != nil {
			s.logger.Error().
				Str("user_id", claims.UserID).
				Str("operation", "validate_token").
				Str("error_type", "blacklist_check_failed").
				Err(err).
				Msg("failed to check token blacklist")
			// Fail-secure: if we can't check blacklist, reject the token
			return "", nil, fmt.Errorf("token revoked")
		}

		if isBlacklisted {
			s.logger.Warn().
				Str("user_id", claims.UserID).
				Str("operation", "validate_token").
				Str("error_type", "token_blacklisted").
				Msg("attempt to use blacklisted token")
			return "", nil, fmt.Errorf("token revoked")
		}
	}

	return claims.UserID, claims.Roles, nil
}

// RefreshToken refreshes an access token
func (s *UserService) RefreshToken(ctx context.Context, refreshToken string) (*models.TokenPair, error) {
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
func (s *UserService) CreateUser(ctx context.Context, email, password, firstName, lastName, phone string, roles []string) (*models.User, error) {
	// Validate email
	if err := s.validator.ValidateEmail(email); err != nil {
		return nil, err
	}

	// Validate password
	if err := s.validator.ValidatePassword(password); err != nil {
		return nil, err
	}

	// Sanitize and validate first name
	sanitizedFirstName, err := s.validator.SanitizeName(firstName)
	if err != nil {
		return nil, err
	}

	// Sanitize and validate last name
	sanitizedLastName, err := s.validator.SanitizeName(lastName)
	if err != nil {
		return nil, err
	}

	// Validate phone (optional field)
	if err := s.validator.ValidatePhone(phone); err != nil {
		return nil, err
	}

	// Check if user already exists
	existingUser, _ := s.userRepo.GetByEmail(ctx, email)
	if existingUser != nil {
		return nil, fmt.Errorf("user with email %s already exists", email)
	}

	// Hash password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, fmt.Errorf("failed to hash password: %w", err)
	}

	// Create user with sanitized names
	user := models.NewUser(email, string(hashedPassword), sanitizedFirstName, sanitizedLastName, phone)
	if err := s.userRepo.Create(ctx, user); err != nil {
		return nil, err
	}

	// Assign roles
	for _, role := range roles {
		if err := s.roleRepo.AssignRoleByName(ctx, user.ID, role); err != nil {
			s.logger.Warn().
				Str("user_id", user.ID).
				Str("role", role).
				Str("operation", "create_user").
				Str("error_type", "role_assignment_failed").
				Err(err).
				Msg("failed to assign role")
		}
	}

	// Reload user to get roles
	return s.userRepo.GetByID(ctx, user.ID)
}

// GetUser retrieves a user by ID
func (s *UserService) GetUser(ctx context.Context, userID string) (*models.User, error) {
	return s.userRepo.GetByID(ctx, userID)
}

// UpdateUser updates a user
func (s *UserService) UpdateUser(ctx context.Context, userID string, email, firstName, lastName, phone *string, isActive *bool) (*models.User, error) {
	user, err := s.userRepo.GetByID(ctx, userID)
	if err != nil {
		return nil, err
	}

	// Update fields if provided with validation
	if email != nil {
		if err := s.validator.ValidateEmail(*email); err != nil {
			return nil, err
		}
		user.Email = *email
	}
	if firstName != nil {
		sanitizedFirstName, err := s.validator.SanitizeName(*firstName)
		if err != nil {
			return nil, err
		}
		user.FirstName = sanitizedFirstName
	}
	if lastName != nil {
		sanitizedLastName, err := s.validator.SanitizeName(*lastName)
		if err != nil {
			return nil, err
		}
		user.LastName = sanitizedLastName
	}
	if phone != nil {
		if err := s.validator.ValidatePhone(*phone); err != nil {
			return nil, err
		}
		user.Phone = *phone
	}
	if isActive != nil {
		user.IsActive = *isActive
	}

	user.UpdatedAt = time.Now()

	if err := s.userRepo.Update(ctx, user); err != nil {
		return nil, err
	}

	return s.userRepo.GetByID(ctx, userID)
}

// DeleteUser deletes a user (soft delete)
func (s *UserService) DeleteUser(ctx context.Context, userID string) error {
	return s.userRepo.Delete(ctx, userID)
}

// ListUsers retrieves a paginated list of users
func (s *UserService) ListUsers(ctx context.Context, page, pageSize int, search, role string, isActive *bool) ([]*models.User, int, error) {
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 10
	}

	return s.userRepo.List(ctx, page, pageSize, search, role, isActive)
}

// GetProfile retrieves a user's profile
func (s *UserService) GetProfile(ctx context.Context, userID string) (*models.Profile, error) {
	user, err := s.userRepo.GetByID(ctx, userID)
	if err != nil {
		return nil, err
	}

	return user.ToProfile(), nil
}

// UpdateProfile updates a user's profile
func (s *UserService) UpdateProfile(ctx context.Context, userID string, firstName, lastName, phone, avatarURL, bio, timezone *string) (*models.Profile, error) {
	user, err := s.userRepo.GetByID(ctx, userID)
	if err != nil {
		return nil, err
	}

	if firstName != nil {
		sanitizedFirstName, err := s.validator.SanitizeName(*firstName)
		if err != nil {
			return nil, err
		}
		user.FirstName = sanitizedFirstName
	}
	if lastName != nil {
		sanitizedLastName, err := s.validator.SanitizeName(*lastName)
		if err != nil {
			return nil, err
		}
		user.LastName = sanitizedLastName
	}
	if phone != nil {
		if err := s.validator.ValidatePhone(*phone); err != nil {
			return nil, err
		}
		user.Phone = *phone
	}
	if avatarURL != nil {
		user.AvatarURL = *avatarURL
	}
	if bio != nil {
		user.Bio = *bio
	}
	if timezone != nil {
		user.Timezone = *timezone
	}

	user.UpdatedAt = time.Now()

	if err := s.userRepo.Update(ctx, user); err != nil {
		return nil, err
	}

	user, err = s.userRepo.GetByID(ctx, userID)
	if err != nil {
		return nil, err
	}

	return user.ToProfile(), nil
}

// ChangePassword changes a user's password
func (s *UserService) ChangePassword(ctx context.Context, userID, oldPassword, newPassword string) error {
	user, err := s.userRepo.GetByID(ctx, userID)
	if err != nil {
		return err
	}

	// Security: Use bcrypt.CompareHashAndPassword for timing-safe password comparison.
	// This prevents timing attacks where an attacker could determine password correctness
	// by measuring response time differences. bcrypt's comparison is constant-time.
	if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(oldPassword)); err != nil {
		return fmt.Errorf("invalid old password")
	}

	// Validate new password complexity before hashing
	if err := s.validator.ValidatePassword(newPassword); err != nil {
		return err
	}

	// Hash new password
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(newPassword), bcrypt.DefaultCost)
	if err != nil {
		return fmt.Errorf("failed to hash password: %w", err)
	}

	// Update password in database
	err = s.userRepo.UpdatePassword(ctx, userID, string(hashedPassword))
	if err != nil {
		return err
	}

	// Security: Invalidate all existing tokens for this user after password change
	// This ensures that if the password was compromised, all sessions are terminated
	if s.tokenBlacklist != nil {
		// Use maximum token lifetime (refresh token duration is typically longest)
		// This ensures all tokens are invalidated, even long-lived refresh tokens
		maxTokenLifetime := 7 * 24 * time.Hour // 7 days (typical refresh token lifetime)

		err = s.tokenBlacklist.BlacklistUserTokens(ctx, userID, maxTokenLifetime)
		if err != nil {
			// Log error but don't fail the password change
			// Password change is more critical than token invalidation
			s.logger.Error().
				Str("user_id", userID).
				Str("operation", "change_password").
				Str("error_type", "token_invalidation_failed").
				Err(err).
				Msg("failed to invalidate user tokens after password change")
		} else {
			s.logger.Info().
				Str("user_id", userID).
				Str("operation", "change_password").
				Msg("all user tokens invalidated after password change")
		}
	}

	s.logger.Info().
		Str("user_id", userID).
		Str("operation", "change_password").
		Msg("password changed successfully")

	return nil
}

// AssignRole assigns a role to a user
func (s *UserService) AssignRole(ctx context.Context, userID, role string) error {
	// Check if user exists
	if _, err := s.userRepo.GetByID(ctx, userID); err != nil {
		return err
	}

	return s.roleRepo.AssignRoleByName(ctx, userID, role)
}

// RemoveRole removes a role from a user
func (s *UserService) RemoveRole(ctx context.Context, userID, role string) error {
	return s.roleRepo.RemoveRoleByName(ctx, userID, role)
}

// GetUserRoles retrieves all roles for a user
func (s *UserService) GetUserRoles(ctx context.Context, userID string) ([]string, error) {
	return s.roleRepo.GetUserRoles(ctx, userID)
}

// CheckPermission checks if a user has a specific permission
func (s *UserService) CheckPermission(ctx context.Context, userID, permission string) (bool, error) {
	return s.roleRepo.CheckPermission(ctx, userID, permission)
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
