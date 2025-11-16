package saml

import (
	"context"
	"encoding/base64"
	"fmt"

	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel/trace"
)

// MockSAMLProvider implements a mock SAML provider for testing
// It returns predefined responses without actual SAML processing
type MockSAMLProvider struct {
	tracer trace.Tracer
	logger *logger.Logger
}

// NewMockSAMLProvider creates a new mock SAML provider
func NewMockSAMLProvider(tracer trace.Tracer, logger *logger.Logger) *MockSAMLProvider {
	return &MockSAMLProvider{
		tracer: tracer,
		logger: logger,
	}
}

// GenerateSAMLRequest generates a mock SAML authentication request
func (p *MockSAMLProvider) GenerateSAMLRequest(ctx context.Context, organizationID string, acsURL string, entityID string) (string, string, error) {
	p.logger.WithContext(ctx).
		Str("organization_id", organizationID).
		Str("provider", "mock").
		Msg("Mock SAML request generation")

	// Create a mock SAML request XML
	samlRequestXML := fmt.Sprintf(`<?xml version="1.0"?>
<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                    ID="mock_request_id_%s"
                    Version="2.0"
                    IssueInstant="2025-01-01T00:00:00Z"
                    Destination="https://mock-idp.example.com/sso"
                    AssertionConsumerServiceURL="%s">
  <saml:Issuer xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">%s</saml:Issuer>
  <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"/>
</samlp:AuthnRequest>`, organizationID, acsURL, entityID)

	// Base64 encode the request
	encodedRequest := base64.StdEncoding.EncodeToString([]byte(samlRequestXML))

	// Return mock SSO URL
	ssoURL := "https://mock-idp.example.com/sso"

	return encodedRequest, ssoURL, nil
}

// ValidateSAMLAssertion validates a mock SAML assertion
func (p *MockSAMLProvider) ValidateSAMLAssertion(ctx context.Context, samlResponse string) (*SAMLUserInfo, error) {
	p.logger.WithContext(ctx).
		Str("provider", "mock").
		Msg("Mock SAML assertion validation")

	// Return mock user info
	return &SAMLUserInfo{
		NameID:    "mock.saml.user@example.com",
		Email:     "mock.saml.user@example.com",
		FirstName: "Mock",
		LastName:  "SAML User",
		Groups:    []string{"users", "developers"},
		Attributes: map[string]interface{}{
			"department": "Engineering",
			"title":      "Software Engineer",
		},
	}, nil
}

// SAMLUserInfo contains user information extracted from SAML assertion
type SAMLUserInfo struct {
	NameID     string
	Email      string
	FirstName  string
	LastName   string
	Groups     []string
	Attributes map[string]interface{}
}
