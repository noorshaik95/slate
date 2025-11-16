package services

import (
	"context"
	"errors"
	"testing"
	"time"

	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/pkg/logger"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"go.opentelemetry.io/otel/trace/noop"
)

// MockOAuthRepository is a mock implementation of OAuthRepositoryInterface
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

// MockSAMLRepository is a mock implementation of SAMLRepositoryInterface
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

func TestStoreOAuthTokens_Success(t *testing.T) {
	mockOAuthRepo := new(MockOAuthRepository)
	mockSAMLRepo := new(MockSAMLRepository)
	tracer := noop.NewTracerProvider().Tracer("test")
	log := logger.NewLogger("debug")

	sessionMgr := NewSessionManager(mockOAuthRepo, mockSAMLRepo, tracer, log)

	now := time.Now()
	provider := &models.OAuthProvider{
		ID:             "oauth-123",
		UserID:         "",
		Provider:       "google",
		ProviderUserID: "google-user-123",
		AccessToken:    "access-token",
		RefreshToken:   "refresh-token",
		TokenExpiry:    now.Add(1 * time.Hour),
		CreatedAt:      now,
		UpdatedAt:      now,
	}

	mockOAuthRepo.On("CreateOrUpdate", mock.Anything, mock.MatchedBy(func(p *models.OAuthProvider) bool {
		return p.UserID == "user-123" && p.Provider == "google"
	})).Return(nil)

	err := sessionMgr.StoreOAuthTokens(context.Background(), "user-123", provider)
	assert.NoError(t, err)
	assert.Equal(t, "user-123", provider.UserID)

	mockOAuthRepo.AssertExpectations(t)
}

func TestStoreOAuthTokens_RepositoryError(t *testing.T) {
	mockOAuthRepo := new(MockOAuthRepository)
	mockSAMLRepo := new(MockSAMLRepository)
	tracer := noop.NewTracerProvider().Tracer("test")
	log := logger.NewLogger("debug")

	sessionMgr := NewSessionManager(mockOAuthRepo, mockSAMLRepo, tracer, log)

	now := time.Now()
	provider := &models.OAuthProvider{
		ID:             "oauth-123",
		UserID:         "",
		Provider:       "google",
		ProviderUserID: "google-user-123",
		AccessToken:    "access-token",
		RefreshToken:   "refresh-token",
		TokenExpiry:    now.Add(1 * time.Hour),
		CreatedAt:      now,
		UpdatedAt:      now,
	}

	expectedErr := errors.New("database error")
	mockOAuthRepo.On("CreateOrUpdate", mock.Anything, mock.Anything).Return(expectedErr)

	err := sessionMgr.StoreOAuthTokens(context.Background(), "user-123", provider)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to store OAuth tokens")

	mockOAuthRepo.AssertExpectations(t)
}

func TestStoreSAMLSession_Success(t *testing.T) {
	mockOAuthRepo := new(MockOAuthRepository)
	mockSAMLRepo := new(MockSAMLRepository)
	tracer := noop.NewTracerProvider().Tracer("test")
	log := logger.NewLogger("debug")

	sessionMgr := NewSessionManager(mockOAuthRepo, mockSAMLRepo, tracer, log)

	session := &models.SAMLSession{
		ID:           "session-123",
		UserID:       "user-123",
		SAMLConfigID: "config-123",
		SessionIndex: "session-index-123",
		NameID:       "user@example.com",
		Attributes: map[string]interface{}{
			"email": "user@example.com",
		},
		CreatedAt: time.Now(),
		ExpiresAt: time.Now().Add(8 * time.Hour),
	}

	mockSAMLRepo.On("CreateSession", mock.Anything, session).Return(nil)

	err := sessionMgr.StoreSAMLSession(context.Background(), session)
	assert.NoError(t, err)

	mockSAMLRepo.AssertExpectations(t)
}

func TestStoreSAMLSession_RepositoryError(t *testing.T) {
	mockOAuthRepo := new(MockOAuthRepository)
	mockSAMLRepo := new(MockSAMLRepository)
	tracer := noop.NewTracerProvider().Tracer("test")
	log := logger.NewLogger("debug")

	sessionMgr := NewSessionManager(mockOAuthRepo, mockSAMLRepo, tracer, log)

	session := &models.SAMLSession{
		ID:           "session-123",
		UserID:       "user-123",
		SAMLConfigID: "config-123",
		SessionIndex: "session-index-123",
		NameID:       "user@example.com",
		Attributes: map[string]interface{}{
			"email": "user@example.com",
		},
		CreatedAt: time.Now(),
		ExpiresAt: time.Now().Add(8 * time.Hour),
	}

	expectedErr := errors.New("database error")
	mockSAMLRepo.On("CreateSession", mock.Anything, session).Return(expectedErr)

	err := sessionMgr.StoreSAMLSession(context.Background(), session)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to store SAML session")

	mockSAMLRepo.AssertExpectations(t)
}
