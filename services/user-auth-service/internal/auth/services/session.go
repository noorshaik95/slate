package services

import (
	"context"
	"fmt"

	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
)

// OAuthRepositoryInterface defines the interface for OAuth repository operations
type OAuthRepositoryInterface interface {
	CreateOrUpdate(ctx context.Context, provider *models.OAuthProvider) error
	GetByProviderAndUserID(ctx context.Context, provider, providerUserID string) (*models.OAuthProvider, error)
	GetByUserID(ctx context.Context, userID string) ([]*models.OAuthProvider, error)
	Delete(ctx context.Context, id string) error
}

// SAMLRepositoryInterface defines the interface for SAML repository operations
type SAMLRepositoryInterface interface {
	CreateConfig(ctx context.Context, config *models.SAMLConfig) error
	GetConfigByEntityID(ctx context.Context, entityID string) (*models.SAMLConfig, error)
	GetConfigByOrganization(ctx context.Context, organizationID string) (*models.SAMLConfig, error)
	CreateSession(ctx context.Context, session *models.SAMLSession) error
	GetSessionByID(ctx context.Context, sessionID string) (*models.SAMLSession, error)
	DeleteExpiredSessions(ctx context.Context) error
}

// SessionManager handles OAuth/SAML token storage. JWT generation already exists in UserService via TokenService.
type SessionManager struct {
	oauthRepo OAuthRepositoryInterface
	samlRepo  SAMLRepositoryInterface
	tracer    trace.Tracer
	logger    *logger.Logger
}

// NewSessionManager creates a new SessionManager instance
func NewSessionManager(
	oauthRepo OAuthRepositoryInterface,
	samlRepo SAMLRepositoryInterface,
	tracer trace.Tracer,
	logger *logger.Logger,
) *SessionManager {
	return &SessionManager{
		oauthRepo: oauthRepo,
		samlRepo:  samlRepo,
		tracer:    tracer,
		logger:    logger,
	}
}

// StoreOAuthTokens stores OAuth tokens for a user
func (m *SessionManager) StoreOAuthTokens(ctx context.Context, userID string, provider *models.OAuthProvider) error {
	ctx, span := m.tracer.Start(ctx, "session.StoreOAuthTokens",
		trace.WithAttributes(
			attribute.String("user_id", userID),
			attribute.String("provider", provider.Provider),
		),
	)
	defer span.End()

	// Ensure the provider has the correct user ID
	provider.UserID = userID

	// Store or update the OAuth provider tokens
	if err := m.oauthRepo.CreateOrUpdate(ctx, provider); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to store OAuth tokens")
		m.logger.ErrorWithContext(ctx).
			Str("user_id", userID).
			Str("provider", provider.Provider).
			Err(err).
			Msg("Failed to store OAuth tokens")
		return fmt.Errorf("failed to store OAuth tokens: %w", err)
	}

	m.logger.WithContext(ctx).
		Str("user_id", userID).
		Str("provider", provider.Provider).
		Msg("OAuth tokens stored successfully")

	return nil
}

// StoreSAMLSession stores a SAML session
func (m *SessionManager) StoreSAMLSession(ctx context.Context, session *models.SAMLSession) error {
	ctx, span := m.tracer.Start(ctx, "session.StoreSAMLSession",
		trace.WithAttributes(
			attribute.String("user_id", session.UserID),
			attribute.String("saml_config_id", session.SAMLConfigID),
		),
	)
	defer span.End()

	// Store the SAML session
	if err := m.samlRepo.CreateSession(ctx, session); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to store SAML session")
		m.logger.ErrorWithContext(ctx).
			Str("user_id", session.UserID).
			Str("session_index", session.SessionIndex).
			Err(err).
			Msg("Failed to store SAML session")
		return fmt.Errorf("failed to store SAML session: %w", err)
	}

	m.logger.WithContext(ctx).
		Str("user_id", session.UserID).
		Str("session_index", session.SessionIndex).
		Msg("SAML session stored successfully")

	return nil
}
