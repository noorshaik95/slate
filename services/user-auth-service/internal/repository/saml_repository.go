package repository

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"

	"slate/services/user-auth-service/internal/models"
)

type SAMLRepository struct {
	db *sql.DB
}

func NewSAMLRepository(db *sql.DB) *SAMLRepository {
	return &SAMLRepository{db: db}
}

// CreateConfig creates a new SAML configuration
func (r *SAMLRepository) CreateConfig(ctx context.Context, config *models.SAMLConfig) error {
	query := `
		INSERT INTO saml_configs (id, organization_id, entity_id, sso_url, slo_url, certificate, is_active, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
	`
	_, err := r.db.ExecContext(ctx, query, config.ID, config.OrganizationID, config.EntityID,
		config.SSOURL, config.SLOURL, config.Certificate, config.IsActive,
		config.CreatedAt, config.UpdatedAt)

	if err != nil {
		return fmt.Errorf("failed to create SAML config: %w", err)
	}
	return nil
}

// GetConfigByEntityID retrieves SAML configuration by entity ID
func (r *SAMLRepository) GetConfigByEntityID(ctx context.Context, entityID string) (*models.SAMLConfig, error) {
	config := &models.SAMLConfig{}
	query := `
		SELECT id, organization_id, entity_id, sso_url, slo_url, certificate, is_active, created_at, updated_at
		FROM saml_configs
		WHERE entity_id = $1 AND is_active = true
	`

	err := r.db.QueryRowContext(ctx, query, entityID).Scan(
		&config.ID, &config.OrganizationID, &config.EntityID, &config.SSOURL,
		&config.SLOURL, &config.Certificate, &config.IsActive,
		&config.CreatedAt, &config.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("SAML config not found")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get SAML config: %w", err)
	}

	return config, nil
}

// GetConfigByOrganization retrieves SAML configuration by organization ID
func (r *SAMLRepository) GetConfigByOrganization(ctx context.Context, organizationID string) (*models.SAMLConfig, error) {
	config := &models.SAMLConfig{}
	query := `
		SELECT id, organization_id, entity_id, sso_url, slo_url, certificate, is_active, created_at, updated_at
		FROM saml_configs
		WHERE organization_id = $1 AND is_active = true
		ORDER BY created_at DESC
		LIMIT 1
	`

	err := r.db.QueryRowContext(ctx, query, organizationID).Scan(
		&config.ID, &config.OrganizationID, &config.EntityID, &config.SSOURL,
		&config.SLOURL, &config.Certificate, &config.IsActive,
		&config.CreatedAt, &config.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("SAML config not found")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get SAML config: %w", err)
	}

	return config, nil
}

// CreateSession creates a new SAML session
func (r *SAMLRepository) CreateSession(ctx context.Context, session *models.SAMLSession) error {
	attributesJSON, err := json.Marshal(session.Attributes)
	if err != nil {
		return fmt.Errorf("failed to marshal attributes: %w", err)
	}

	query := `
		INSERT INTO saml_sessions (id, user_id, saml_config_id, session_index, name_id, attributes, created_at, expires_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
	`
	_, err = r.db.ExecContext(ctx, query, session.ID, session.UserID, session.SAMLConfigID,
		session.SessionIndex, session.NameID, attributesJSON, session.CreatedAt, session.ExpiresAt)

	if err != nil {
		return fmt.Errorf("failed to create SAML session: %w", err)
	}
	return nil
}

// GetSessionByID retrieves a SAML session by ID
func (r *SAMLRepository) GetSessionByID(ctx context.Context, sessionID string) (*models.SAMLSession, error) {
	session := &models.SAMLSession{}
	var attributesJSON []byte

	query := `
		SELECT id, user_id, saml_config_id, session_index, name_id, attributes, created_at, expires_at
		FROM saml_sessions
		WHERE id = $1 AND expires_at > NOW()
	`

	err := r.db.QueryRowContext(ctx, query, sessionID).Scan(
		&session.ID, &session.UserID, &session.SAMLConfigID, &session.SessionIndex,
		&session.NameID, &attributesJSON, &session.CreatedAt, &session.ExpiresAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("SAML session not found or expired")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get SAML session: %w", err)
	}

	if err := json.Unmarshal(attributesJSON, &session.Attributes); err != nil {
		return nil, fmt.Errorf("failed to unmarshal attributes: %w", err)
	}

	return session, nil
}

// DeleteExpiredSessions removes expired SAML sessions
func (r *SAMLRepository) DeleteExpiredSessions(ctx context.Context) error {
	query := `DELETE FROM saml_sessions WHERE expires_at <= NOW()`
	_, err := r.db.ExecContext(ctx, query)
	if err != nil {
		return fmt.Errorf("failed to delete expired sessions: %w", err)
	}
	return nil
}
