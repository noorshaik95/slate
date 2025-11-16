package auth

import "slate/services/user-auth-service/internal/models"

// AuthType represents the authentication method type.
// It determines which authentication strategy will be used for user authentication.
type AuthType string

const (
	// AuthTypeNormal represents traditional username/password authentication
	AuthTypeNormal AuthType = "normal"
	// AuthTypeOAuth represents OAuth 2.0 authentication with external providers
	AuthTypeOAuth AuthType = "oauth"
	// AuthTypeSAML represents SAML 2.0 authentication with identity providers
	AuthTypeSAML AuthType = "saml"
)

// AuthRequest contains the data required to initiate an authentication request.
// Different fields are used depending on the authentication type:
// - Normal: Email and Password are required
// - OAuth: Provider is required
// - SAML: OrganizationID is required
type AuthRequest struct {
	Email          string `json:"email" validate:"omitempty,email"`
	Password       string `json:"password" validate:"omitempty,min=8"`
	OrganizationID string `json:"organization_id" validate:"omitempty"`
	Provider       string `json:"provider" validate:"omitempty"`
}

// CallbackRequest contains the data received from authentication provider callbacks.
// Used for OAuth and SAML authentication flows:
// - OAuth: Code and State are required
// - SAML: SAMLResponse is required
type CallbackRequest struct {
	Code         string `json:"code" validate:"omitempty"`
	State        string `json:"state" validate:"omitempty"`
	SAMLResponse string `json:"saml_response" validate:"omitempty"`
}

// AuthResult contains the result of an authentication attempt.
// For successful authentication, Success is true and User/Tokens are populated.
// For OAuth/SAML initiation, Success is false and redirect information is provided.
type AuthResult struct {
	// Success indicates whether authentication was completed successfully
	Success bool `json:"success"`

	// User contains the authenticated user information (populated on success)
	User *models.User `json:"user,omitempty"`

	// Tokens contains the JWT access and refresh tokens (populated on success)
	Tokens *models.TokenPair `json:"tokens,omitempty"`

	// AuthorizationURL is the OAuth provider's authorization URL (OAuth initiation only)
	AuthorizationURL string `json:"authorization_url,omitempty"`

	// State is the CSRF protection token for OAuth (OAuth initiation only)
	State string `json:"state,omitempty"`

	// SAMLRequest is the encoded SAML authentication request (SAML initiation only)
	SAMLRequest string `json:"saml_request,omitempty"`

	// SSOURL is the SAML identity provider's SSO URL (SAML initiation only)
	SSOURL string `json:"sso_url,omitempty"`
}
