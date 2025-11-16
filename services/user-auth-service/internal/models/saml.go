package models

import (
	"time"

	"github.com/google/uuid"
)

// SAMLConfig represents SAML SSO configuration for an organization
type SAMLConfig struct {
	ID             string    `json:"id"`
	OrganizationID string    `json:"organization_id,omitempty"`
	EntityID       string    `json:"entity_id"`
	SSOURL         string    `json:"sso_url"`
	SLOURL         string    `json:"slo_url,omitempty"` // Single Logout URL
	Certificate    string    `json:"-"` // X.509 certificate, never expose fully in JSON
	IsActive       bool      `json:"is_active"`
	CreatedAt      time.Time `json:"created_at"`
	UpdatedAt      time.Time `json:"updated_at"`
}

// NewSAMLConfig creates a new SAML configuration
func NewSAMLConfig(organizationID, entityID, ssoURL, slourl, certificate string) *SAMLConfig {
	now := time.Now()
	return &SAMLConfig{
		ID:             uuid.New().String(),
		OrganizationID: organizationID,
		EntityID:       entityID,
		SSOURL:         ssoURL,
		SLOURL:         slourl,
		Certificate:    certificate,
		IsActive:       true,
		CreatedAt:      now,
		UpdatedAt:      now,
	}
}

// SAMLSession represents an active SAML session
type SAMLSession struct {
	ID           string                 `json:"id"`
	UserID       string                 `json:"user_id"`
	SAMLConfigID string                 `json:"saml_config_id"`
	SessionIndex string                 `json:"session_index,omitempty"`
	NameID       string                 `json:"name_id"`
	Attributes   map[string]interface{} `json:"attributes,omitempty"`
	CreatedAt    time.Time              `json:"created_at"`
	ExpiresAt    time.Time              `json:"expires_at"`
}

// NewSAMLSession creates a new SAML session
func NewSAMLSession(userID, samlConfigID, sessionIndex, nameID string, attributes map[string]interface{}, duration time.Duration) *SAMLSession {
	now := time.Now()
	return &SAMLSession{
		ID:           uuid.New().String(),
		UserID:       userID,
		SAMLConfigID: samlConfigID,
		SessionIndex: sessionIndex,
		NameID:       nameID,
		Attributes:   attributes,
		CreatedAt:    now,
		ExpiresAt:    now.Add(duration),
	}
}
