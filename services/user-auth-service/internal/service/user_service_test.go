package service

import (
	"bytes"
	"context"
	"errors"
	"testing"
	"time"

	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/pkg/logger"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/bcrypt"
)

// Mock repositories
type MockUserRepository struct {
	mock.Mock
}

func (m *MockUserRepository) Create(ctx context.Context, user *models.User) error {
	args := m.Called(user)
	return args.Error(0)
}

func (m *MockUserRepository) GetByID(ctx context.Context, id string) (*models.User, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockUserRepository) GetByEmail(ctx context.Context, email string) (*models.User, error) {
	args := m.Called(email)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockUserRepository) Update(ctx context.Context, user *models.User) error {
	args := m.Called(user)
	return args.Error(0)
}

func (m *MockUserRepository) Delete(ctx context.Context, id string) error {
	args := m.Called(id)
	return args.Error(0)
}

func (m *MockUserRepository) List(ctx context.Context, page, pageSize int, search, role string, isActive *bool) ([]*models.User, int, error) {
	args := m.Called(page, pageSize, search, role, isActive)
	if args.Get(0) == nil {
		return nil, args.Int(1), args.Error(2)
	}
	return args.Get(0).([]*models.User), args.Int(1), args.Error(2)
}

func (m *MockUserRepository) UpdatePassword(ctx context.Context, userID, passwordHash string) error {
	args := m.Called(userID, passwordHash)
	return args.Error(0)
}

type MockRoleRepository struct {
	mock.Mock
}

func (m *MockRoleRepository) AssignRoleByName(ctx context.Context, userID, roleName string) error {
	args := m.Called(userID, roleName)
	return args.Error(0)
}

func (m *MockRoleRepository) RemoveRoleByName(ctx context.Context, userID, roleName string) error {
	args := m.Called(userID, roleName)
	return args.Error(0)
}

func (m *MockRoleRepository) GetUserRoles(ctx context.Context, userID string) ([]string, error) {
	args := m.Called(userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]string), args.Error(1)
}

func (m *MockRoleRepository) CheckPermission(ctx context.Context, userID, permission string) (bool, error) {
	args := m.Called(userID, permission)
	return args.Bool(0), args.Error(1)
}

type MockTokenService struct {
	mock.Mock
}

func (m *MockTokenService) GenerateAccessToken(userID, email string, roles []string) (string, int64, error) {
	args := m.Called(userID, email, roles)
	return args.String(0), args.Get(1).(int64), args.Error(2)
}

func (m *MockTokenService) GenerateRefreshToken(userID, email string, roles []string) (string, error) {
	args := m.Called(userID, email, roles)
	return args.String(0), args.Error(1)
}

func (m *MockTokenService) ValidateAccessToken(token string) (*TokenClaims, error) {
	args := m.Called(token)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*TokenClaims), args.Error(1)
}

func (m *MockTokenService) RefreshAccessToken(refreshToken string) (string, string, int64, error) {
	args := m.Called(refreshToken)
	return args.String(0), args.String(1), args.Get(2).(int64), args.Error(3)
}

type MockTokenBlacklist struct {
	mock.Mock
}

func (m *MockTokenBlacklist) BlacklistToken(ctx context.Context, token string, expiresAt time.Time) error {
	args := m.Called(ctx, token, expiresAt)
	return args.Error(0)
}

func (m *MockTokenBlacklist) BlacklistUserTokens(ctx context.Context, userID string, maxTokenLifetime time.Duration) error {
	args := m.Called(ctx, userID, maxTokenLifetime)
	return args.Error(0)
}

func (m *MockTokenBlacklist) IsTokenBlacklisted(ctx context.Context, token string, userID string, issuedAt time.Time) (bool, error) {
	args := m.Called(ctx, token, userID, issuedAt)
	return args.Bool(0), args.Error(1)
}

type MockMetrics struct {
	mock.Mock
}

func (m *MockMetrics) IncrementRegistrations(success bool) {
	m.Called(success)
}

func (m *MockMetrics) IncrementLogins(success bool) {
	m.Called(success)
}

func (m *MockMetrics) ObserveRequestDuration(operation string, durationSeconds float64) {
	m.Called(operation, durationSeconds)
}

func (m *MockMetrics) SetDBConnections(count int) {
	m.Called(count)
}

// Helper function to create test user
func createTestUser() *models.User {
	hashedPassword, _ := bcrypt.GenerateFromPassword([]byte("Password123!"), bcrypt.DefaultCost)
	return &models.User{
		ID:           "user-123",
		Email:        "test@example.com",
		PasswordHash: string(hashedPassword),
		FirstName:    "John",
		LastName:     "Doe",
		Phone:        "+1234567890",
		Timezone:     "UTC",
		IsActive:     true,
		Roles:        []string{"user"},
		CreatedAt:    time.Now(),
		UpdatedAt:    time.Now(),
	}
}

// Helper function to create test logger
func createTestLogger() *logger.Logger {
	var buf bytes.Buffer
	return logger.NewLoggerWithWriter("info", &buf)
}

// Helper function to create test metrics
func createTestMetrics() *MockMetrics {
	m := new(MockMetrics)
	// Set up default expectations for metrics calls
	m.On("IncrementRegistrations", mock.Anything).Return()
	m.On("IncrementLogins", mock.Anything).Return()
	m.On("ObserveRequestDuration", mock.Anything, mock.Anything).Return()
	m.On("SetDBConnections", mock.Anything).Return()
	return m
}

func TestRegister_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)
	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	email := "newuser@example.com"
	password := "Password123!"
	firstName := "Jane"
	lastName := "Smith"
	phone := "+1234567890"

	// Mock expectations
	mockUserRepo.On("GetByEmail", email).Return(nil, errors.New("not found"))
	mockUserRepo.On("Create", mock.AnythingOfType("*models.User")).Return(nil)
	mockRoleRepo.On("AssignRoleByName", mock.Anything, "user").Return(nil)

	createdUser := createTestUser()
	createdUser.Email = email
	createdUser.FirstName = firstName
	createdUser.LastName = lastName
	mockUserRepo.On("GetByID", mock.Anything).Return(createdUser, nil)

	mockTokenSvc.On("GenerateAccessToken", mock.Anything, email, []string{"user"}).
		Return("access-token", int64(900), nil)
	mockTokenSvc.On("GenerateRefreshToken", mock.Anything, email, []string{"user"}).
		Return("refresh-token", nil)

	user, tokens, err := svc.Register(context.Background(), email, password, firstName, lastName, phone)

	require.NoError(t, err)
	assert.NotNil(t, user)
	assert.NotNil(t, tokens)
	assert.Equal(t, email, user.Email)
	assert.Equal(t, firstName, user.FirstName)
	assert.Equal(t, lastName, user.LastName)
	assert.Equal(t, "access-token", tokens.AccessToken)
	assert.Equal(t, "refresh-token", tokens.RefreshToken)

	mockUserRepo.AssertExpectations(t)
	mockRoleRepo.AssertExpectations(t)
	mockTokenSvc.AssertExpectations(t)
}

func TestRegister_UserAlreadyExists(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	email := "existing@example.com"
	existingUser := createTestUser()
	existingUser.Email = email

	mockUserRepo.On("GetByEmail", email).Return(existingUser, nil)

	user, tokens, err := svc.Register(context.Background(), email, "Password123!", "John", "Doe", "+1234567890")

	assert.Error(t, err)
	assert.Nil(t, user)
	assert.Nil(t, tokens)
	assert.Contains(t, err.Error(), "already exists")

	mockUserRepo.AssertExpectations(t)
}

func TestLogin_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	testUser := createTestUser()
	email := testUser.Email
	password := "Password123!"

	mockUserRepo.On("GetByEmail", email).Return(testUser, nil)
	mockTokenSvc.On("GenerateAccessToken", testUser.ID, email, testUser.Roles).
		Return("access-token", int64(900), nil)
	mockTokenSvc.On("GenerateRefreshToken", testUser.ID, email, testUser.Roles).
		Return("refresh-token", nil)

	user, tokens, err := svc.Login(context.Background(), email, password)

	require.NoError(t, err)
	assert.NotNil(t, user)
	assert.NotNil(t, tokens)
	assert.Equal(t, testUser.ID, user.ID)
	assert.Equal(t, "access-token", tokens.AccessToken)
	assert.Equal(t, "refresh-token", tokens.RefreshToken)

	mockUserRepo.AssertExpectations(t)
	mockTokenSvc.AssertExpectations(t)
}

func TestLogin_InvalidCredentials(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	mockUserRepo.On("GetByEmail", "test@example.com").Return(nil, errors.New("not found"))

	user, tokens, err := svc.Login(context.Background(), "test@example.com", "wrongpassword")

	assert.Error(t, err)
	assert.Nil(t, user)
	assert.Nil(t, tokens)
	assert.Contains(t, err.Error(), "invalid credentials")

	mockUserRepo.AssertExpectations(t)
}

func TestLogin_WrongPassword(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	testUser := createTestUser()
	mockUserRepo.On("GetByEmail", testUser.Email).Return(testUser, nil)

	user, tokens, err := svc.Login(context.Background(), testUser.Email, "wrongpassword")

	assert.Error(t, err)
	assert.Nil(t, user)
	assert.Nil(t, tokens)
	assert.Contains(t, err.Error(), "invalid credentials")

	mockUserRepo.AssertExpectations(t)
}

func TestLogin_InactiveUser(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	testUser := createTestUser()
	testUser.IsActive = false
	mockUserRepo.On("GetByEmail", testUser.Email).Return(testUser, nil)

	user, tokens, err := svc.Login(context.Background(), testUser.Email, "password123")

	assert.Error(t, err)
	assert.Nil(t, user)
	assert.Nil(t, tokens)
	assert.Contains(t, err.Error(), "inactive")

	mockUserRepo.AssertExpectations(t)
}

func TestValidateToken_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	token := "valid-token"
	claims := &TokenClaims{
		UserID: "user-123",
		Roles:  []string{"user", "admin"},
	}

	mockTokenSvc.On("ValidateAccessToken", token).Return(claims, nil)
	mockBlacklist.On("IsTokenBlacklisted", mock.Anything, token, claims.UserID, mock.Anything).Return(false, nil)

	userID, roles, err := svc.ValidateToken(context.Background(), token)

	require.NoError(t, err)
	assert.Equal(t, claims.UserID, userID)
	assert.Equal(t, claims.Roles, roles)

	mockTokenSvc.AssertExpectations(t)
	mockBlacklist.AssertExpectations(t)
}

func TestValidateToken_InvalidToken(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	token := "invalid-token"
	mockTokenSvc.On("ValidateAccessToken", token).Return(nil, errors.New("invalid token"))

	userID, roles, err := svc.ValidateToken(context.Background(), token)

	assert.Error(t, err)
	assert.Empty(t, userID)
	assert.Nil(t, roles)

	mockTokenSvc.AssertExpectations(t)
}

func TestGetUser_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	testUser := createTestUser()
	mockUserRepo.On("GetByID", testUser.ID).Return(testUser, nil)

	user, err := svc.GetUser(context.Background(), testUser.ID)

	require.NoError(t, err)
	assert.Equal(t, testUser.ID, user.ID)
	assert.Equal(t, testUser.Email, user.Email)

	mockUserRepo.AssertExpectations(t)
}

func TestGetUser_NotFound(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	mockUserRepo.On("GetByID", "nonexistent").Return(nil, errors.New("user not found"))

	user, err := svc.GetUser(context.Background(), "nonexistent")

	assert.Error(t, err)
	assert.Nil(t, user)

	mockUserRepo.AssertExpectations(t)
}

func TestUpdateUser_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	testUser := createTestUser()
	newEmail := "newemail@example.com"
	newFirstName := "Jane"
	isActive := false

	mockUserRepo.On("GetByID", testUser.ID).Return(testUser, nil).Once()
	mockUserRepo.On("Update", mock.AnythingOfType("*models.User")).Return(nil)

	updatedUser := createTestUser()
	updatedUser.Email = newEmail
	updatedUser.FirstName = newFirstName
	updatedUser.IsActive = isActive
	mockUserRepo.On("GetByID", testUser.ID).Return(updatedUser, nil).Once()

	user, err := svc.UpdateUser(context.Background(), testUser.ID, &newEmail, &newFirstName, nil, nil, &isActive)

	require.NoError(t, err)
	assert.Equal(t, newEmail, user.Email)
	assert.Equal(t, newFirstName, user.FirstName)
	assert.Equal(t, isActive, user.IsActive)

	mockUserRepo.AssertExpectations(t)
}

func TestDeleteUser_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	userID := "user-123"
	mockUserRepo.On("Delete", userID).Return(nil)

	err := svc.DeleteUser(context.Background(), userID)

	require.NoError(t, err)
	mockUserRepo.AssertExpectations(t)
}

func TestChangePassword_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	testUser := createTestUser()
	oldPassword := "Password123!"
	newPassword := "NewPassword456!"

	mockUserRepo.On("GetByID", testUser.ID).Return(testUser, nil)
	mockUserRepo.On("UpdatePassword", testUser.ID, mock.AnythingOfType("string")).Return(nil)
	mockBlacklist.On("BlacklistUserTokens", mock.Anything, testUser.ID, mock.AnythingOfType("time.Duration")).Return(nil)

	err := svc.ChangePassword(context.Background(), testUser.ID, oldPassword, newPassword)

	require.NoError(t, err)
	mockUserRepo.AssertExpectations(t)
	mockBlacklist.AssertExpectations(t)
}

func TestChangePassword_WrongOldPassword(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	testUser := createTestUser()
	mockUserRepo.On("GetByID", testUser.ID).Return(testUser, nil)

	err := svc.ChangePassword(context.Background(), testUser.ID, "wrongpassword", "newpassword")

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid old password")
	mockUserRepo.AssertExpectations(t)
}

func TestAssignRole_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	userID := "user-123"
	role := "admin"
	testUser := createTestUser()

	mockUserRepo.On("GetByID", userID).Return(testUser, nil)
	mockRoleRepo.On("AssignRoleByName", userID, role).Return(nil)

	err := svc.AssignRole(context.Background(), userID, role)

	require.NoError(t, err)
	mockUserRepo.AssertExpectations(t)
	mockRoleRepo.AssertExpectations(t)
}

func TestRemoveRole_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	userID := "user-123"
	role := "admin"

	mockRoleRepo.On("RemoveRoleByName", userID, role).Return(nil)

	err := svc.RemoveRole(context.Background(), userID, role)

	require.NoError(t, err)
	mockRoleRepo.AssertExpectations(t)
}

func TestGetUserRoles_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	userID := "user-123"
	expectedRoles := []string{"user", "admin"}

	mockRoleRepo.On("GetUserRoles", userID).Return(expectedRoles, nil)

	roles, err := svc.GetUserRoles(context.Background(), userID)

	require.NoError(t, err)
	assert.Equal(t, expectedRoles, roles)
	mockRoleRepo.AssertExpectations(t)
}

func TestCheckPermission_Success(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockRoleRepo := new(MockRoleRepository)
	mockTokenSvc := new(MockTokenService)

	mockBlacklist := new(MockTokenBlacklist)

	svc := NewUserService(mockUserRepo, mockRoleRepo, mockTokenSvc, mockBlacklist, createTestLogger(), createTestMetrics())

	userID := "user-123"
	permission := "users.read"

	mockRoleRepo.On("CheckPermission", userID, permission).Return(true, nil)

	hasPermission, err := svc.CheckPermission(context.Background(), userID, permission)

	require.NoError(t, err)
	assert.True(t, hasPermission)
	mockRoleRepo.AssertExpectations(t)
}
