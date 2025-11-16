package strategies

import (
	"bytes"
	"context"
	"errors"
	"testing"
	"time"

	"slate/services/user-auth-service/internal/auth"
	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/pkg/logger"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/otel"
	"golang.org/x/crypto/bcrypt"
)

// MockUserService mocks the UserService for testing
type MockUserService struct {
	mock.Mock
}

func (m *MockUserService) Login(ctx context.Context, email, password string) (*models.User, *models.TokenPair, error) {
	args := m.Called(ctx, email, password)
	if args.Get(0) == nil {
		return nil, nil, args.Error(2)
	}
	if args.Get(1) == nil {
		return args.Get(0).(*models.User), nil, args.Error(2)
	}
	return args.Get(0).(*models.User), args.Get(1).(*models.TokenPair), args.Error(2)
}

// Helper function to create test logger
func createTestLogger() *logger.Logger {
	var buf bytes.Buffer
	return logger.NewLoggerWithWriter("info", &buf)
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

// Helper function to create test tokens
func createTestTokens() *models.TokenPair {
	return &models.TokenPair{
		AccessToken:  "access-token-123",
		RefreshToken: "refresh-token-456",
		ExpiresIn:    900,
	}
}

func TestNormalAuthStrategy_GetType(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	assert.Equal(t, auth.AuthTypeNormal, strategy.GetType())
}

func TestNormalAuthStrategy_ValidateConfig_Success(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	err := strategy.ValidateConfig()

	assert.NoError(t, err)
}

func TestNormalAuthStrategy_ValidateConfig_MissingDependency(t *testing.T) {
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(nil, tracer, log)

	err := strategy.ValidateConfig()

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "userService is required")
}

func TestNormalAuthStrategy_HandleCallback_NotSupported(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	result, err := strategy.HandleCallback(context.Background(), &auth.CallbackRequest{
		Code:  "some-code",
		State: "some-state",
	})

	assert.Error(t, err)
	assert.Nil(t, result)
	assert.Contains(t, err.Error(), "normal authentication does not support callbacks")
}

func TestNormalAuthStrategy_Authenticate_Success(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	testUser := createTestUser()
	testTokens := createTestTokens()

	mockUserService.On("Login", mock.Anything, "test@example.com", "Password123!").
		Return(testUser, testTokens, nil)

	req := &auth.AuthRequest{
		Email:    "test@example.com",
		Password: "Password123!",
	}

	result, err := strategy.Authenticate(context.Background(), req)

	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.True(t, result.Success)
	assert.Equal(t, testUser, result.User)
	assert.Equal(t, testTokens, result.Tokens)
	assert.Equal(t, testUser.ID, result.User.ID)
	assert.Equal(t, testUser.Email, result.User.Email)

	mockUserService.AssertExpectations(t)
}

func TestNormalAuthStrategy_Authenticate_InvalidEmail(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	req := &auth.AuthRequest{
		Email:    "",
		Password: "Password123!",
	}

	result, err := strategy.Authenticate(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, result)
	assert.Contains(t, err.Error(), "email is required")

	mockUserService.AssertNotCalled(t, "Login")
}

func TestNormalAuthStrategy_Authenticate_InvalidPassword(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	req := &auth.AuthRequest{
		Email:    "test@example.com",
		Password: "",
	}

	result, err := strategy.Authenticate(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, result)
	assert.Contains(t, err.Error(), "password is required")

	mockUserService.AssertNotCalled(t, "Login")
}

func TestNormalAuthStrategy_Authenticate_UserNotFound(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	mockUserService.On("Login", mock.Anything, "nonexistent@example.com", "Password123!").
		Return(nil, nil, errors.New("invalid credentials"))

	req := &auth.AuthRequest{
		Email:    "nonexistent@example.com",
		Password: "Password123!",
	}

	result, err := strategy.Authenticate(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, result)
	assert.Contains(t, err.Error(), "invalid credentials")

	mockUserService.AssertExpectations(t)
}

func TestNormalAuthStrategy_Authenticate_WrongPassword(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	mockUserService.On("Login", mock.Anything, "test@example.com", "wrongpassword").
		Return(nil, nil, errors.New("invalid credentials"))

	req := &auth.AuthRequest{
		Email:    "test@example.com",
		Password: "wrongpassword",
	}

	result, err := strategy.Authenticate(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, result)
	assert.Contains(t, err.Error(), "invalid credentials")

	mockUserService.AssertExpectations(t)
}

func TestNormalAuthStrategy_Authenticate_InactiveUser(t *testing.T) {
	mockUserService := new(MockUserService)
	tracer := otel.Tracer("test")
	log := createTestLogger()

	strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

	mockUserService.On("Login", mock.Anything, "inactive@example.com", "Password123!").
		Return(nil, nil, errors.New("user account is inactive"))

	req := &auth.AuthRequest{
		Email:    "inactive@example.com",
		Password: "Password123!",
	}

	result, err := strategy.Authenticate(context.Background(), req)

	assert.Error(t, err)
	assert.Nil(t, result)
	assert.Contains(t, err.Error(), "user account is inactive")

	mockUserService.AssertExpectations(t)
}

// Table-driven test for multiple authentication failure scenarios
func TestNormalAuthStrategy_Authenticate_FailureScenarios(t *testing.T) {
	tests := []struct {
		name          string
		email         string
		password      string
		mockError     error
		expectedError string
	}{
		{
			name:          "empty email",
			email:         "",
			password:      "Password123!",
			mockError:     nil,
			expectedError: "email is required",
		},
		{
			name:          "empty password",
			email:         "test@example.com",
			password:      "",
			mockError:     nil,
			expectedError: "password is required",
		},
		{
			name:          "user not found",
			email:         "notfound@example.com",
			password:      "Password123!",
			mockError:     errors.New("invalid credentials"),
			expectedError: "invalid credentials",
		},
		{
			name:          "wrong password",
			email:         "test@example.com",
			password:      "wrongpassword",
			mockError:     errors.New("invalid credentials"),
			expectedError: "invalid credentials",
		},
		{
			name:          "inactive user",
			email:         "inactive@example.com",
			password:      "Password123!",
			mockError:     errors.New("user account is inactive"),
			expectedError: "user account is inactive",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockUserService := new(MockUserService)
			tracer := otel.Tracer("test")
			log := createTestLogger()

			strategy := NewNormalAuthStrategy(mockUserService, tracer, log)

			// Only mock Login if we expect it to be called (email and password are not empty)
			if tt.email != "" && tt.password != "" {
				mockUserService.On("Login", mock.Anything, tt.email, tt.password).
					Return(nil, nil, tt.mockError)
			}

			req := &auth.AuthRequest{
				Email:    tt.email,
				Password: tt.password,
			}

			result, err := strategy.Authenticate(context.Background(), req)

			assert.Error(t, err)
			assert.Nil(t, result)
			assert.Contains(t, err.Error(), tt.expectedError)

			if tt.email != "" && tt.password != "" {
				mockUserService.AssertExpectations(t)
			}
		})
	}
}
