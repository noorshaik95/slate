package models

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNewSAMLConfig(t *testing.T) {
	organizationID := "org-123"
	entityID := "https://example.com/saml"
	ssoURL := "https://idp.example.com/sso"
	slourl := "https://idp.example.com/slo"
	certificate := "-----BEGIN CERTIFICATE-----\nMIIC..."

	config := NewSAMLConfig(organizationID, entityID, ssoURL, slourl, certificate)

	assert.NotNil(t, config)
	assert.NotEmpty(t, config.ID)
	assert.Equal(t, organizationID, config.OrganizationID)
	assert.Equal(t, entityID, config.EntityID)
	assert.Equal(t, ssoURL, config.SSOURL)
	assert.Equal(t, slourl, config.SLOURL)
	assert.Equal(t, certificate, config.Certificate)
	assert.True(t, config.IsActive) // Should be active by default
	assert.False(t, config.CreatedAt.IsZero())
	assert.False(t, config.UpdatedAt.IsZero())
}

func TestSAMLConfig_Fields(t *testing.T) {
	config := &SAMLConfig{
		ID:             "saml-123",
		OrganizationID: "org-123",
		EntityID:       "https://example.com/saml",
		SSOURL:         "https://idp.example.com/sso",
		SLOURL:         "https://idp.example.com/slo",
		Certificate:    "-----BEGIN CERTIFICATE-----",
		IsActive:       true,
	}

	assert.Equal(t, "saml-123", config.ID)
	assert.Equal(t, "org-123", config.OrganizationID)
	assert.Equal(t, "https://example.com/saml", config.EntityID)
	assert.Equal(t, "https://idp.example.com/sso", config.SSOURL)
	assert.Equal(t, "https://idp.example.com/slo", config.SLOURL)
	assert.Equal(t, "-----BEGIN CERTIFICATE-----", config.Certificate)
	assert.True(t, config.IsActive)
}

func TestSAMLConfig_JSONSerialization(t *testing.T) {
	config := &SAMLConfig{
		ID:          "saml-123",
		EntityID:    "https://example.com/saml",
		Certificate: "secret-certificate",
	}

	// Certificate should not be in JSON (tagged with json:"-")
	jsonData := string(mustMarshal(t, config))
	assert.NotContains(t, jsonData, "secret-certificate")
	assert.Contains(t, jsonData, "saml-123")
}

func TestNewSAMLSession(t *testing.T) {
	userID := "user-123"
	samlConfigID := "saml-config-123"
	sessionIndex := "session-idx-123"
	nameID := "user@example.com"
	attributes := map[string]interface{}{
		"email":      "user@example.com",
		"first_name": "John",
		"last_name":  "Doe",
		"groups":     []string{"admin", "users"},
	}
	duration := 24 * time.Hour

	session := NewSAMLSession(userID, samlConfigID, sessionIndex, nameID, attributes, duration)

	assert.NotNil(t, session)
	assert.NotEmpty(t, session.ID)
	assert.Equal(t, userID, session.UserID)
	assert.Equal(t, samlConfigID, session.SAMLConfigID)
	assert.Equal(t, sessionIndex, session.SessionIndex)
	assert.Equal(t, nameID, session.NameID)
	assert.Equal(t, attributes, session.Attributes)
	assert.False(t, session.CreatedAt.IsZero())
	assert.False(t, session.ExpiresAt.IsZero())
	// Session should expire after the specified duration
	assert.True(t, session.ExpiresAt.After(session.CreatedAt))
}

func TestSAMLSession_Fields(t *testing.T) {
	session := &SAMLSession{
		ID:           "session-123",
		UserID:       "user-123",
		SAMLConfigID: "config-123",
		SessionIndex: "session-idx-123",
		NameID:       "user@example.com",
	}

	assert.Equal(t, "session-123", session.ID)
	assert.Equal(t, "user-123", session.UserID)
	assert.Equal(t, "config-123", session.SAMLConfigID)
	assert.Equal(t, "session-idx-123", session.SessionIndex)
	assert.Equal(t, "user@example.com", session.NameID)
}

func TestSAMLSession_Attributes(t *testing.T) {
	tests := []struct {
		name       string
		attributes map[string]interface{}
	}{
		{
			"User attributes",
			map[string]interface{}{
				"email":      "user@example.com",
				"first_name": "John",
				"last_name":  "Doe",
			},
		},
		{
			"Extended attributes",
			map[string]interface{}{
				"email":       "user@example.com",
				"groups":      []string{"admin", "users"},
				"permissions": []string{"read", "write"},
				"department":  "Engineering",
			},
		},
		{
			"Empty attributes",
			map[string]interface{}{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			session := NewSAMLSession("user-123", "config-123", "session-idx", "user@example.com", tt.attributes, 24*time.Hour)
			assert.Equal(t, tt.attributes, session.Attributes)
		})
	}
}

func TestSAMLSession_Expiry(t *testing.T) {
	tests := []struct {
		name     string
		duration time.Duration
	}{
		{"1 hour", 1 * time.Hour},
		{"24 hours", 24 * time.Hour},
		{"7 days", 7 * 24 * time.Hour},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			session := NewSAMLSession("user-123", "config-123", "session-idx", "user@example.com", nil, tt.duration)
			expectedExpiry := session.CreatedAt.Add(tt.duration)
			assert.Equal(t, expectedExpiry.Unix(), session.ExpiresAt.Unix())
		})
	}
}
