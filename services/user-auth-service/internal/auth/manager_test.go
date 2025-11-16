package auth

import (
	"bytes"
	"context"
	"testing"

	"slate/services/user-auth-service/internal/config"
	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/pkg/logger"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/otel"
)

// TokenClaims represents JWT token claims (defined locally to avoid import cycle)
type TokenClaims struct {
	UserID string
	Email  string
	Roles  []string
}

// Mock implementations for testing

type MockUserRepository struct {
	mock.Mock
}

func (m *MockUserRepository) Create(ctx context.Context, user *models.User) error {
	args := m.Called(ctx, user)
	return args.Error(0)
}

func (m *MockUserRepository) GetByID(ctx context.Context, id string) (*models.User, error) {
	args := m.Called(ctx, id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockUserRepository) GetByEmail(ctx context.Context, email string) (*models.User, error) {
	args := m.Called(ctx, email)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockUserRepository) Update(ctx context.Context, user *models.User) error {
	args := m.Called(ctx, user)
	return args.Error(0)
}

func (m *MockUserRepository) Delete(ctx context.Context, id string) error {
	args := m.Called(ctx, id)
	return args.Error(0)
}

func (m *MockUserRepository) List(ctx context.Context, limit, offset int) ([]*models.User, error) {
	args := m.Called(ctx, limit, offset)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]*models.User), args.Error(1)
}

type MockOAuthRepository struct {
	mock.Mock
}

func (m *MockOAuthRepository) CreateOrUpdate(ctx context.Context, provider *models.OAuthProvider) error {
	args := m.Called(ctx, provider)
	return args.Error(0)
}

func (m *MockOAuthRepository) GetByProviderAndUserID(ctx context.Context, provider, providerUserID string) (*models.OAuthProvider, error) {
	args := m.Called(ctx, provider, providerUserID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.OAuthProvider), args.Error(1)
}

func (m *MockOAuthRepository) GetByUserID(ctx context.Context, userID string) ([]*models.OAuthProvider, error) {
	args := m.Called(ctx, userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]*models.OAuthProvider), args.Error(1)
}

func (m *MockOAuthRepository) Delete(ctx context.Context, id string) error {
	args := m.Called(ctx, id)
	return args.Error(0)
}

type MockSAMLRepository struct {
	mock.Mock
}

func (m *MockSAMLRepository) CreateConfig(ctx context.Context, config *models.SAMLConfig) error {
	args := m.Called(ctx, config)
	return args.Error(0)
}

func (m *MockSAMLRepository) GetConfigByEntityID(ctx context.Context, entityID string) (*models.SAMLConfig, error) {
	args := m.Called(ctx, entityID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.SAMLConfig), args.Error(1)
}

func (m *MockSAMLRepository) GetConfigByOrganization(ctx context.Context, organizationID string) (*models.SAMLConfig, error) {
	args := m.Called(ctx, organizationID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.SAMLConfig), args.Error(1)
}

func (m *MockSAMLRepository) CreateSession(ctx context.Context, session *models.SAMLSession) error {
	args := m.Called(ctx, session)
	return args.Error(0)
}

func (m *MockSAMLRepository) GetSessionByID(ctx context.Context, sessionID string) (*models.SAMLSession, error) {
	args := m.Called(ctx, sessionID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.SAMLSession), args.Error(1)
}

func (m *MockSAMLRepository) DeleteExpiredSessions(ctx context.Context) error {
	args := m.Called(ctx)
	return args.Error(0)
}

func (m *MockSAMLRepository) StoreMetadata(ctx context.Context, configKey, metadataXML string) error {
	args := m.Called(ctx, configKey, metadataXML)
	return args.Error(0)
}

func (m *MockSAMLRepository) GetMetadata(ctx context.Context, configKey string) (string, error) {
	args := m.Called(ctx, configKey)
	return args.String(0), args.Error(1)
}

type MockRoleRepository struct {
	mock.Mock
}

func (m *MockRoleRepository) GetUserRoles(ctx context.Context, userID string) ([]string, error) {
	args := m.Called(ctx, userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]string), args.Error(1)
}

func (m *MockRoleRepository) AssignRoleByName(ctx context.Context, userID, roleName string) error {
	args := m.Called(ctx, userID, roleName)
	return args.Error(0)
}

func (m *MockRoleRepository) RemoveRoleByName(ctx context.Context, userID, roleName string) error {
	args := m.Called(ctx, userID, roleName)
	return args.Error(0)
}

type MockTokenService struct {
	mock.Mock
}

func (m *MockTokenService) GenerateAccessToken(userID, email string, roles []string) (string, int, error) {
	args := m.Called(userID, email, roles)
	return args.String(0), args.Int(1), args.Error(2)
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

func (m *MockTokenService) ValidateRefreshToken(token string) (*TokenClaims, error) {
	args := m.Called(token)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*TokenClaims), args.Error(1)
}

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

// MockAuthenticationStrategy for testing
type MockAuthenticationStrategy struct {
	mock.Mock
	authType AuthType
}

func (m *MockAuthenticationStrategy) Authenticate(ctx context.Context, req *AuthRequest) (*AuthResult, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*AuthResult), args.Error(1)
}

func (m *MockAuthenticationStrategy) HandleCallback(ctx context.Context, req *CallbackRequest) (*AuthResult, error) {
	args := m.Called(ctx, req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*AuthResult), args.Error(1)
}

func (m *MockAuthenticationStrategy) GetType() AuthType {
	args := m.Called()
	return args.Get(0).(AuthType)
}

func (m *MockAuthenticationStrategy) ValidateConfig() error {
	args := m.Called()
	return args.Error(0)
}

// Helper functions

func createTestLogger() *logger.Logger {
	var buf bytes.Buffer
	return logger.NewLoggerWithWriter("info", &buf)
}

func createTestConfig(authType string) *config.Config {
	cfg := &config.Config{
		Auth: config.AuthConfig{
			Type: authType,
		},
		OAuth: config.OAuthConfig{
			Providers: make(map[string]config.OAuthProviderConfig),
		},
		SAML: config.SAMLConfig{
			Providers: make(map[string]config.SAMLProviderConfig),
		},
	}
	return cfg
}

// Test cases for sub-task 11.6: Constructor tests

func TestNewStrategyManager_Success(t *testing.T) {
	// Create mock dependencies
	cfg := createTestConfig("normal")
	tracer := otel.Tracer("test")
	log := createTestLogger()

	// Call NewStrategyManager
	manager := NewStrategyManager(cfg, tracer, log)

	// Assert StrategyManager is created
	require.NotNil(t, manager)
	assert.NotNil(t, manager.strategies)
	assert.NotNil(t, manager.config)
	assert.NotNil(t, manager.tracer)
	assert.NotNil(t, manager.logger)

	// Assert strategies map is initialized
	assert.Equal(t, 0, len(manager.strategies))
}

// Test cases for sub-task 11.7: Registration tests

func TestStrategyManager_RegisterStrategy_Success(t *testing.T) {
	// Create manager
	cfg := createTestConfig("normal")
	manager := NewStrategyManager(cfg, otel.Tracer("test"), createTestLogger())

	// Create mock strategy
	mockStrategy := new(MockAuthenticationStrategy)
	mockStrategy.On("ValidateConfig").Return(nil)
	mockStrategy.On("GetType").Return(AuthTypeNormal)

	// Call RegisterStrategy
	err := manager.RegisterStrategy(mockStrategy)

	// Assert no error
	assert.NoError(t, err)

	// Assert strategy is stored in map
	strategy, exists := manager.strategies[AuthTypeNormal]
	assert.True(t, exists)
	assert.Equal(t, mockStrategy, strategy)

	// Verify mock expectations
	mockStrategy.AssertExpectations(t)
}

func TestStrategyManager_RegisterStrategy_ValidationFails(t *testing.T) {
	// Create manager
	cfg := createTestConfig("normal")
	manager := NewStrategyManager(cfg, otel.Tracer("test"), createTestLogger())

	// Create mock strategy that fails validation
	mockStrategy := new(MockAuthenticationStrategy)
	mockStrategy.On("ValidateConfig").Return(assert.AnError)
	mockStrategy.On("GetType").Return(AuthTypeNormal)

	// Call RegisterStrategy
	err := manager.RegisterStrategy(mockStrategy)

	// Assert error is returned
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "strategy validation failed")

	// Assert strategy is not stored
	_, exists := manager.strategies[AuthTypeNormal]
	assert.False(t, exists)

	// Verify mock expectations
	mockStrategy.AssertExpectations(t)
}

func TestStrategyManager_RegisterStrategy_DuplicateType(t *testing.T) {
	// Create manager
	cfg := createTestConfig("normal")
	manager := NewStrategyManager(cfg, otel.Tracer("test"), createTestLogger())

	// Create first mock strategy
	mockStrategy1 := new(MockAuthenticationStrategy)
	mockStrategy1.On("ValidateConfig").Return(nil)
	mockStrategy1.On("GetType").Return(AuthTypeNormal)

	// Register first strategy successfully
	err := manager.RegisterStrategy(mockStrategy1)
	require.NoError(t, err)

	// Create second mock strategy with same type
	mockStrategy2 := new(MockAuthenticationStrategy)
	mockStrategy2.On("ValidateConfig").Return(nil)
	mockStrategy2.On("GetType").Return(AuthTypeNormal)

	// Try to register second strategy with same type
	err = manager.RegisterStrategy(mockStrategy2)

	// Assert error "strategy already registered"
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "strategy already registered")

	// Verify first strategy is still registered
	strategy, exists := manager.strategies[AuthTypeNormal]
	assert.True(t, exists)
	assert.Equal(t, mockStrategy1, strategy)
}

// Test cases for sub-task 11.8: GetStrategy tests

func TestStrategyManager_GetStrategy_Success(t *testing.T) {
	// Create manager
	cfg := createTestConfig("normal")
	manager := NewStrategyManager(cfg, otel.Tracer("test"), createTestLogger())

	// Register NormalAuthStrategy
	mockStrategy := new(MockAuthenticationStrategy)
	mockStrategy.On("ValidateConfig").Return(nil)
	mockStrategy.On("GetType").Return(AuthTypeNormal)
	err := manager.RegisterStrategy(mockStrategy)
	require.NoError(t, err)

	// Call GetStrategy
	strategy, err := manager.GetStrategy(AuthTypeNormal)

	// Assert strategy is returned
	assert.NoError(t, err)
	assert.NotNil(t, strategy)
	assert.Equal(t, mockStrategy, strategy)
}

func TestStrategyManager_GetStrategy_NotFound(t *testing.T) {
	// Create manager
	cfg := createTestConfig("normal")
	manager := NewStrategyManager(cfg, otel.Tracer("test"), createTestLogger())

	// Call GetStrategy without registering OAuth strategy
	strategy, err := manager.GetStrategy(AuthTypeOAuth)

	// Assert error "no strategy registered"
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "no strategy registered")
	assert.Nil(t, strategy)
}

func TestStrategyManager_GetStrategy_MultipleStrategies(t *testing.T) {
	// Create manager
	cfg := createTestConfig("normal")
	manager := NewStrategyManager(cfg, otel.Tracer("test"), createTestLogger())

	// Register Normal, OAuth, and SAML strategies
	mockNormalStrategy := new(MockAuthenticationStrategy)
	mockNormalStrategy.On("ValidateConfig").Return(nil)
	mockNormalStrategy.On("GetType").Return(AuthTypeNormal)
	err := manager.RegisterStrategy(mockNormalStrategy)
	require.NoError(t, err)

	mockOAuthStrategy := new(MockAuthenticationStrategy)
	mockOAuthStrategy.On("ValidateConfig").Return(nil)
	mockOAuthStrategy.On("GetType").Return(AuthTypeOAuth)
	err = manager.RegisterStrategy(mockOAuthStrategy)
	require.NoError(t, err)

	mockSAMLStrategy := new(MockAuthenticationStrategy)
	mockSAMLStrategy.On("ValidateConfig").Return(nil)
	mockSAMLStrategy.On("GetType").Return(AuthTypeSAML)
	err = manager.RegisterStrategy(mockSAMLStrategy)
	require.NoError(t, err)

	// Call GetStrategy for each type
	normalStrategy, err := manager.GetStrategy(AuthTypeNormal)
	assert.NoError(t, err)
	assert.Equal(t, mockNormalStrategy, normalStrategy)

	oauthStrategy, err := manager.GetStrategy(AuthTypeOAuth)
	assert.NoError(t, err)
	assert.Equal(t, mockOAuthStrategy, oauthStrategy)

	samlStrategy, err := manager.GetStrategy(AuthTypeSAML)
	assert.NoError(t, err)
	assert.Equal(t, mockSAMLStrategy, samlStrategy)
}

// Test cases for sub-task 11.9: Initialization tests
// Note: InitializeStrategies is now in a separate init package to avoid import cycles
// These tests verify the GetActiveAuthType method

func TestStrategyManager_GetActiveAuthType(t *testing.T) {
	tests := []struct {
		name     string
		authType string
		expected AuthType
	}{
		{
			name:     "Normal auth type",
			authType: "normal",
			expected: AuthTypeNormal,
		},
		{
			name:     "OAuth auth type",
			authType: "oauth",
			expected: AuthTypeOAuth,
		},
		{
			name:     "SAML auth type",
			authType: "saml",
			expected: AuthTypeSAML,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cfg := createTestConfig(tt.authType)
			manager := NewStrategyManager(cfg, otel.Tracer("test"), createTestLogger())

			result := manager.GetActiveAuthType()
			assert.Equal(t, tt.expected, result)
		})
	}
}
