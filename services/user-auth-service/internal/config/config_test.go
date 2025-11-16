package config

import (
	"os"
	"testing"
	"time"
)

func TestLoad_AuthTypeNormal(t *testing.T) {
	// Set environment variables
	t.Setenv("AUTH_TYPE", "normal")
	t.Setenv("SESSION_DURATION", "24h")

	config, err := Load()
	if err != nil {
		t.Fatalf("Load() failed: %v", err)
	}

	if config.Auth.Type != "normal" {
		t.Errorf("Expected Auth.Type to be 'normal', got '%s'", config.Auth.Type)
	}

	if config.Auth.SessionDuration != 24*time.Hour {
		t.Errorf("Expected SessionDuration to be 24h, got %v", config.Auth.SessionDuration)
	}
}

func TestLoad_AuthTypeOAuth(t *testing.T) {
	// Set environment variables
	t.Setenv("AUTH_TYPE", "oauth")
	t.Setenv("OAUTH_GOOGLE_CLIENT_ID", "test-client-id")
	t.Setenv("OAUTH_GOOGLE_CLIENT_SECRET", "test-client-secret")
	t.Setenv("OAUTH_GOOGLE_REDIRECT_URI", "http://localhost:8080/callback")
	t.Setenv("OAUTH_GOOGLE_SCOPES", "openid,profile,email")

	config, err := Load()
	if err != nil {
		t.Fatalf("Load() failed: %v", err)
	}

	if config.Auth.Type != "oauth" {
		t.Errorf("Expected Auth.Type to be 'oauth', got '%s'", config.Auth.Type)
	}

	if len(config.OAuth.Providers) == 0 {
		t.Fatal("Expected OAuth providers to be loaded")
	}

	googleProvider, exists := config.OAuth.Providers["google"]
	if !exists {
		t.Fatal("Expected Google OAuth provider to be configured")
	}

	if googleProvider.ClientID != "test-client-id" {
		t.Errorf("Expected ClientID to be 'test-client-id', got '%s'", googleProvider.ClientID)
	}

	if googleProvider.ClientSecret != "test-client-secret" {
		t.Errorf("Expected ClientSecret to be 'test-client-secret', got '%s'", googleProvider.ClientSecret)
	}

	if googleProvider.RedirectURI != "http://localhost:8080/callback" {
		t.Errorf("Expected RedirectURI to be 'http://localhost:8080/callback', got '%s'", googleProvider.RedirectURI)
	}

	expectedScopes := []string{"openid", "profile", "email"}
	if len(googleProvider.Scopes) != len(expectedScopes) {
		t.Errorf("Expected %d scopes, got %d", len(expectedScopes), len(googleProvider.Scopes))
	}
}

func TestLoad_AuthTypeSAML(t *testing.T) {
	// Create temporary certificate and key files
	certFile, err := os.CreateTemp("", "cert-*.pem")
	if err != nil {
		t.Fatalf("Failed to create temp cert file: %v", err)
	}
	defer os.Remove(certFile.Name())
	certFile.Close()

	keyFile, err := os.CreateTemp("", "key-*.pem")
	if err != nil {
		t.Fatalf("Failed to create temp key file: %v", err)
	}
	defer os.Remove(keyFile.Name())
	keyFile.Close()

	// Set environment variables
	t.Setenv("AUTH_TYPE", "saml")
	t.Setenv("SAML_SP_ENTITY_ID", "http://localhost:8080/saml/metadata")
	t.Setenv("SAML_ACS_URL", "http://localhost:8080/saml/acs")
	t.Setenv("SAML_CERTIFICATE_PATH", certFile.Name())
	t.Setenv("SAML_PRIVATE_KEY_PATH", keyFile.Name())
	t.Setenv("SAML_OKTA_METADATA_URL", "https://okta.example.com/metadata")
	t.Setenv("SAML_OKTA_JIT_PROVISIONING", "true")
	t.Setenv("SAML_OKTA_GROUP_SYNC", "true")

	config, err := Load()
	if err != nil {
		t.Fatalf("Load() failed: %v", err)
	}

	if config.Auth.Type != "saml" {
		t.Errorf("Expected Auth.Type to be 'saml', got '%s'", config.Auth.Type)
	}

	if config.SAML.ServiceProviderEntityID != "http://localhost:8080/saml/metadata" {
		t.Errorf("Expected ServiceProviderEntityID to be 'http://localhost:8080/saml/metadata', got '%s'", config.SAML.ServiceProviderEntityID)
	}

	if len(config.SAML.Providers) == 0 {
		t.Fatal("Expected SAML providers to be loaded")
	}

	oktaProvider, exists := config.SAML.Providers["okta"]
	if !exists {
		t.Fatal("Expected Okta SAML provider to be configured")
	}

	if oktaProvider.MetadataURL != "https://okta.example.com/metadata" {
		t.Errorf("Expected MetadataURL to be 'https://okta.example.com/metadata', got '%s'", oktaProvider.MetadataURL)
	}

	if !oktaProvider.JITProvisioning {
		t.Error("Expected JITProvisioning to be true")
	}

	if !oktaProvider.GroupSync {
		t.Error("Expected GroupSync to be true")
	}
}

func TestLoad_InvalidAuthType(t *testing.T) {
	// Set invalid AUTH_TYPE
	t.Setenv("AUTH_TYPE", "invalid")

	config, err := Load()
	if err != nil {
		t.Fatalf("Load() failed: %v", err)
	}

	// Should default to "normal"
	if config.Auth.Type != "normal" {
		t.Errorf("Expected Auth.Type to default to 'normal', got '%s'", config.Auth.Type)
	}
}

func TestLoad_OAuthProviders(t *testing.T) {
	// Set multiple OAuth providers
	t.Setenv("OAUTH_GOOGLE_CLIENT_ID", "google-client-id")
	t.Setenv("OAUTH_GOOGLE_CLIENT_SECRET", "google-secret")
	t.Setenv("OAUTH_GOOGLE_REDIRECT_URI", "http://localhost:8080/callback")

	t.Setenv("OAUTH_MICROSOFT_CLIENT_ID", "microsoft-client-id")
	t.Setenv("OAUTH_MICROSOFT_CLIENT_SECRET", "microsoft-secret")
	t.Setenv("OAUTH_MICROSOFT_REDIRECT_URI", "http://localhost:8080/callback")
	t.Setenv("OAUTH_MICROSOFT_TENANT", "organizations")

	config, err := Load()
	if err != nil {
		t.Fatalf("Load() failed: %v", err)
	}

	if len(config.OAuth.Providers) != 2 {
		t.Errorf("Expected 2 OAuth providers, got %d", len(config.OAuth.Providers))
	}

	if _, exists := config.OAuth.Providers["google"]; !exists {
		t.Error("Expected Google provider to be configured")
	}

	if _, exists := config.OAuth.Providers["microsoft"]; !exists {
		t.Error("Expected Microsoft provider to be configured")
	}
}

func TestLoad_SAMLProviders(t *testing.T) {
	// Set multiple SAML providers
	t.Setenv("SAML_OKTA_METADATA_URL", "https://okta.example.com/metadata")
	t.Setenv("SAML_AUTH0_METADATA_URL", "https://auth0.example.com/metadata")
	t.Setenv("SAML_ADFS_ENTITY_ID", "http://adfs.example.com/services/trust")

	config, err := Load()
	if err != nil {
		t.Fatalf("Load() failed: %v", err)
	}

	if len(config.SAML.Providers) != 3 {
		t.Errorf("Expected 3 SAML providers, got %d", len(config.SAML.Providers))
	}

	if _, exists := config.SAML.Providers["okta"]; !exists {
		t.Error("Expected Okta provider to be configured")
	}

	if _, exists := config.SAML.Providers["auth0"]; !exists {
		t.Error("Expected Auth0 provider to be configured")
	}

	if _, exists := config.SAML.Providers["adfs"]; !exists {
		t.Error("Expected ADFS provider to be configured")
	}
}

func TestValidate_Success(t *testing.T) {
	config := &Config{
		Auth: AuthConfig{
			Type: "normal",
		},
	}

	err := config.Validate()
	if err != nil {
		t.Errorf("Validate() failed for valid config: %v", err)
	}
}

func TestValidate_MissingOAuthProvider(t *testing.T) {
	config := &Config{
		Auth: AuthConfig{
			Type: "oauth",
		},
		OAuth: OAuthConfig{
			Providers: make(map[string]OAuthProviderConfig),
		},
	}

	err := config.Validate()
	if err == nil {
		t.Error("Expected validation error for oauth without providers")
	}
}

func TestValidate_MissingSAMLCertificate(t *testing.T) {
	config := &Config{
		Auth: AuthConfig{
			Type: "saml",
		},
		SAML: SAMLConfig{
			CertificatePath: "/nonexistent/cert.pem",
			PrivateKeyPath:  "/nonexistent/key.pem",
		},
	}

	err := config.Validate()
	if err == nil {
		t.Error("Expected validation error for missing SAML certificate")
	}
}

func TestValidate_InvalidOAuthConfig(t *testing.T) {
	config := &Config{
		Auth: AuthConfig{
			Type: "oauth",
		},
		OAuth: OAuthConfig{
			Providers: map[string]OAuthProviderConfig{
				"google": {
					Type:     "google",
					ClientID: "", // Missing
				},
			},
		},
	}

	err := config.Validate()
	if err == nil {
		t.Error("Expected validation error for OAuth config with missing ClientID")
	}
}
