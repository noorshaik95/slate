package config

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

type Config struct {
	Server        ServerConfig
	Database      DatabaseConfig
	JWT           JWTConfig
	GRPC          GRPCConfig
	Auth          AuthConfig
	OAuth         OAuthConfig
	SAML          SAMLConfig
	Observability ObservabilityConfig
	Environment   string // "development", "production", "test"
}

// AuthConfig contains authentication configuration
type AuthConfig struct {
	Type            string        // "normal", "oauth", "saml"
	SessionDuration time.Duration // Session duration for authenticated users
}

// OAuthConfig contains OAuth provider configurations
type OAuthConfig struct {
	Providers map[string]OAuthProviderConfig // Map of provider name to configuration
}

// OAuthProviderConfig contains configuration for a specific OAuth provider
type OAuthProviderConfig struct {
	Type             string            // Provider type: "google", "microsoft", "custom"
	ClientID         string            // OAuth client ID
	ClientSecret     string            // OAuth client secret
	RedirectURI      string            // OAuth redirect URI
	Scopes           []string          // OAuth scopes to request
	AuthURL          string            // Authorization endpoint URL
	TokenURL         string            // Token endpoint URL
	UserInfoURL      string            // User info endpoint URL
	AttributeMapping map[string]string // Custom attribute mapping for user fields
}

// SAMLConfig contains SAML service provider configuration
type SAMLConfig struct {
	ServiceProviderEntityID     string                        // SP entity ID
	AssertionConsumerServiceURL string                        // ACS URL where assertions are posted
	CertificatePath             string                        // Path to SP certificate
	PrivateKeyPath              string                        // Path to SP private key
	Providers                   map[string]SAMLProviderConfig // Map of provider name to configuration
}

// SAMLProviderConfig contains configuration for a specific SAML provider
type SAMLProviderConfig struct {
	Type             string            // Provider type: "okta", "auth0", "adfs", "shibboleth", "custom"
	MetadataURL      string            // IdP metadata URL for automatic refresh
	EntityID         string            // IdP entity ID
	SSOURL           string            // IdP SSO URL
	Certificate      string            // IdP certificate for signature validation
	JITProvisioning  bool              // Enable just-in-time user provisioning
	GroupSync        bool              // Enable group synchronization
	GroupAttribute   string            // SAML attribute containing groups
	AttributeMapping map[string]string // Custom attribute mapping for user fields
}

// ObservabilityConfig contains observability and tracing configuration
type ObservabilityConfig struct {
	TracingEnabled  bool   // Enable distributed tracing
	TracingEndpoint string // OpenTelemetry collector endpoint
	ServiceName     string // Service name for tracing
}

type ServerConfig struct {
	Host string
	Port int
}

type DatabaseConfig struct {
	Host     string
	Port     int
	User     string
	Password string
	DBName   string
	SSLMode  string
}

type JWTConfig struct {
	SecretKey            string
	AccessTokenDuration  int // in minutes
	RefreshTokenDuration int // in hours
}

type GRPCConfig struct {
	Host string
	Port int
}

func Load() (*Config, error) {
	// Load AUTH_TYPE and validate
	authType := getEnv("AUTH_TYPE", "normal")
	if authType != "normal" && authType != "oauth" && authType != "saml" {
		fmt.Printf("Warning: Invalid AUTH_TYPE '%s', defaulting to 'normal'\n", authType)
		authType = "normal"
	}

	// Parse SESSION_DURATION with default 24h
	sessionDuration := getEnvAsDuration("SESSION_DURATION", 24*time.Hour)

	return &Config{
		Server: ServerConfig{
			Host: getEnv("SERVER_HOST", "0.0.0.0"),
			Port: getEnvAsInt("SERVER_PORT", 8081),
		},
		Database: DatabaseConfig{
			Host:     getEnv("DB_HOST", "localhost"),
			Port:     getEnvAsInt("DB_PORT", 5432),
			User:     getEnv("DB_USER", "postgres"),
			Password: getEnv("DB_PASSWORD", "postgres"),
			DBName:   getEnv("DB_NAME", "userauth"),
			SSLMode:  getEnv("DB_SSLMODE", "disable"),
		},
		JWT: JWTConfig{
			SecretKey:            getEnv("JWT_SECRET", "your-secret-key-change-in-production"),
			AccessTokenDuration:  getEnvAsInt("JWT_ACCESS_TOKEN_DURATION", 15),
			RefreshTokenDuration: getEnvAsInt("JWT_REFRESH_TOKEN_DURATION", 168), // 7 days
		},
		GRPC: GRPCConfig{
			Host: getEnv("GRPC_HOST", "0.0.0.0"),
			Port: getEnvAsInt("GRPC_PORT", 50051),
		},
		Auth: AuthConfig{
			Type:            authType,
			SessionDuration: sessionDuration,
		},
		OAuth: OAuthConfig{
			Providers: loadOAuthProviders(),
		},
		SAML: SAMLConfig{
			ServiceProviderEntityID:     getEnv("SAML_SP_ENTITY_ID", ""),
			AssertionConsumerServiceURL: getEnv("SAML_ACS_URL", ""),
			CertificatePath:             getEnv("SAML_CERTIFICATE_PATH", ""),
			PrivateKeyPath:              getEnv("SAML_PRIVATE_KEY_PATH", ""),
			Providers:                   loadSAMLProviders(),
		},
		Observability: ObservabilityConfig{
			TracingEnabled:  getEnvAsBool("TRACING_ENABLED", false),
			TracingEndpoint: getEnv("TRACING_ENDPOINT", "http://localhost:4317"),
			ServiceName:     getEnv("SERVICE_NAME", "user-auth-service"),
		},
		Environment: getEnv("ENVIRONMENT", "development"),
	}, nil
}

func (c *DatabaseConfig) DSN() string {
	return fmt.Sprintf("host=%s port=%d user=%s password=%s dbname=%s sslmode=%s",
		c.Host, c.Port, c.User, c.Password, c.DBName, c.SSLMode)
}

func (c *GRPCConfig) Address() string {
	return fmt.Sprintf("%s:%d", c.Host, c.Port)
}

func (c *ServerConfig) Address() string {
	return fmt.Sprintf("%s:%d", c.Host, c.Port)
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

func getEnvAsInt(key string, defaultValue int) int {
	if value := os.Getenv(key); value != "" {
		if intValue, err := strconv.Atoi(value); err == nil {
			return intValue
		}
	}
	return defaultValue
}

func getEnvAsDuration(key string, defaultValue time.Duration) time.Duration {
	if value := os.Getenv(key); value != "" {
		if duration, err := time.ParseDuration(value); err == nil {
			return duration
		}
	}
	return defaultValue
}

func getEnvAsBool(key string, defaultValue bool) bool {
	if value := os.Getenv(key); value != "" {
		if boolValue, err := strconv.ParseBool(value); err == nil {
			return boolValue
		}
	}
	return defaultValue
}

// parseCommaSeparated parses a comma-separated string into a slice
func parseCommaSeparated(value string) []string {
	if value == "" {
		return []string{}
	}
	parts := strings.Split(value, ",")
	result := make([]string, 0, len(parts))
	for _, part := range parts {
		if trimmed := strings.TrimSpace(part); trimmed != "" {
			result = append(result, trimmed)
		}
	}
	return result
}

// parseJSONMapping parses a JSON string into a map
func parseJSONMapping(value string) map[string]string {
	if value == "" {
		return make(map[string]string)
	}
	var mapping map[string]string
	if err := json.Unmarshal([]byte(value), &mapping); err != nil {
		fmt.Printf("Warning: Failed to parse JSON mapping: %v\n", err)
		return make(map[string]string)
	}
	return mapping
}

// loadOAuthProviders loads OAuth provider configurations from environment variables
func loadOAuthProviders() map[string]OAuthProviderConfig {
	providers := make(map[string]OAuthProviderConfig)

	// Load Google provider config
	if clientID := getEnv("OAUTH_GOOGLE_CLIENT_ID", ""); clientID != "" {
		providers["google"] = OAuthProviderConfig{
			Type:             "google",
			ClientID:         clientID,
			ClientSecret:     getEnv("OAUTH_GOOGLE_CLIENT_SECRET", ""),
			RedirectURI:      getEnv("OAUTH_GOOGLE_REDIRECT_URI", ""),
			Scopes:           parseCommaSeparated(getEnv("OAUTH_GOOGLE_SCOPES", "openid,profile,email")),
			AuthURL:          getEnv("OAUTH_GOOGLE_AUTH_URL", "https://accounts.google.com/o/oauth2/v2/auth"),
			TokenURL:         getEnv("OAUTH_GOOGLE_TOKEN_URL", "https://oauth2.googleapis.com/token"),
			UserInfoURL:      getEnv("OAUTH_GOOGLE_USERINFO_URL", "https://www.googleapis.com/oauth2/v2/userinfo"),
			AttributeMapping: parseJSONMapping(getEnv("OAUTH_GOOGLE_ATTR_MAPPING", "")),
		}
	}

	// Load Microsoft provider config
	if clientID := getEnv("OAUTH_MICROSOFT_CLIENT_ID", ""); clientID != "" {
		tenant := getEnv("OAUTH_MICROSOFT_TENANT", "common")
		providers["microsoft"] = OAuthProviderConfig{
			Type:             "microsoft",
			ClientID:         clientID,
			ClientSecret:     getEnv("OAUTH_MICROSOFT_CLIENT_SECRET", ""),
			RedirectURI:      getEnv("OAUTH_MICROSOFT_REDIRECT_URI", ""),
			Scopes:           parseCommaSeparated(getEnv("OAUTH_MICROSOFT_SCOPES", "openid,profile,email")),
			AuthURL:          getEnv("OAUTH_MICROSOFT_AUTH_URL", fmt.Sprintf("https://login.microsoftonline.com/%s/oauth2/v2.0/authorize", tenant)),
			TokenURL:         getEnv("OAUTH_MICROSOFT_TOKEN_URL", fmt.Sprintf("https://login.microsoftonline.com/%s/oauth2/v2.0/token", tenant)),
			UserInfoURL:      getEnv("OAUTH_MICROSOFT_USERINFO_URL", "https://graph.microsoft.com/v1.0/me"),
			AttributeMapping: parseJSONMapping(getEnv("OAUTH_MICROSOFT_ATTR_MAPPING", "")),
		}
	}

	// Load Custom provider config
	if clientID := getEnv("OAUTH_CUSTOM_CLIENT_ID", ""); clientID != "" {
		providers["custom"] = OAuthProviderConfig{
			Type:             "custom",
			ClientID:         clientID,
			ClientSecret:     getEnv("OAUTH_CUSTOM_CLIENT_SECRET", ""),
			RedirectURI:      getEnv("OAUTH_CUSTOM_REDIRECT_URI", ""),
			Scopes:           parseCommaSeparated(getEnv("OAUTH_CUSTOM_SCOPES", "")),
			AuthURL:          getEnv("OAUTH_CUSTOM_AUTH_URL", ""),
			TokenURL:         getEnv("OAUTH_CUSTOM_TOKEN_URL", ""),
			UserInfoURL:      getEnv("OAUTH_CUSTOM_USERINFO_URL", ""),
			AttributeMapping: parseJSONMapping(getEnv("OAUTH_CUSTOM_ATTR_MAPPING", "")),
		}
	}

	return providers
}

// loadSAMLProviders loads SAML provider configurations from environment variables
func loadSAMLProviders() map[string]SAMLProviderConfig {
	providers := make(map[string]SAMLProviderConfig)

	// Load Okta provider config (if metadata URL, entity ID, or SSO URL is set)
	metadataURL := getEnv("SAML_OKTA_METADATA_URL", "")
	entityID := getEnv("SAML_OKTA_ENTITY_ID", "")
	ssoURL := getEnv("SAML_OKTA_SSO_URL", "")
	if metadataURL != "" || entityID != "" || ssoURL != "" {
		providers["okta"] = SAMLProviderConfig{
			Type:             "okta",
			MetadataURL:      metadataURL,
			EntityID:         entityID,
			SSOURL:           ssoURL,
			Certificate:      getEnv("SAML_OKTA_CERTIFICATE", ""),
			JITProvisioning:  getEnvAsBool("SAML_OKTA_JIT_PROVISIONING", false),
			GroupSync:        getEnvAsBool("SAML_OKTA_GROUP_SYNC", false),
			GroupAttribute:   getEnv("SAML_OKTA_GROUP_ATTRIBUTE", "groups"),
			AttributeMapping: parseJSONMapping(getEnv("SAML_OKTA_ATTR_MAPPING", "")),
		}
	}

	// Load Auth0 provider config
	if metadataURL := getEnv("SAML_AUTH0_METADATA_URL", ""); metadataURL != "" {
		providers["auth0"] = SAMLProviderConfig{
			Type:             "auth0",
			MetadataURL:      metadataURL,
			EntityID:         getEnv("SAML_AUTH0_ENTITY_ID", ""),
			SSOURL:           getEnv("SAML_AUTH0_SSO_URL", ""),
			Certificate:      getEnv("SAML_AUTH0_CERTIFICATE", ""),
			JITProvisioning:  getEnvAsBool("SAML_AUTH0_JIT_PROVISIONING", false),
			GroupSync:        getEnvAsBool("SAML_AUTH0_GROUP_SYNC", false),
			GroupAttribute:   getEnv("SAML_AUTH0_GROUP_ATTRIBUTE", "groups"),
			AttributeMapping: parseJSONMapping(getEnv("SAML_AUTH0_ATTR_MAPPING", "")),
		}
	}

	// Load ADFS provider config
	if entityID := getEnv("SAML_ADFS_ENTITY_ID", ""); entityID != "" {
		providers["adfs"] = SAMLProviderConfig{
			Type:             "adfs",
			MetadataURL:      getEnv("SAML_ADFS_METADATA_URL", ""),
			EntityID:         entityID,
			SSOURL:           getEnv("SAML_ADFS_SSO_URL", ""),
			Certificate:      getEnv("SAML_ADFS_CERTIFICATE", ""),
			JITProvisioning:  getEnvAsBool("SAML_ADFS_JIT_PROVISIONING", false),
			GroupSync:        getEnvAsBool("SAML_ADFS_GROUP_SYNC", false),
			GroupAttribute:   getEnv("SAML_ADFS_GROUP_ATTRIBUTE", "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups"),
			AttributeMapping: parseJSONMapping(getEnv("SAML_ADFS_ATTR_MAPPING", "")),
		}
	}

	// Load Shibboleth provider config
	if metadataURL := getEnv("SAML_SHIBBOLETH_METADATA_URL", ""); metadataURL != "" {
		providers["shibboleth"] = SAMLProviderConfig{
			Type:             "shibboleth",
			MetadataURL:      metadataURL,
			EntityID:         getEnv("SAML_SHIBBOLETH_ENTITY_ID", ""),
			SSOURL:           getEnv("SAML_SHIBBOLETH_SSO_URL", ""),
			Certificate:      getEnv("SAML_SHIBBOLETH_CERTIFICATE", ""),
			JITProvisioning:  getEnvAsBool("SAML_SHIBBOLETH_JIT_PROVISIONING", false),
			GroupSync:        getEnvAsBool("SAML_SHIBBOLETH_GROUP_SYNC", false),
			GroupAttribute:   getEnv("SAML_SHIBBOLETH_GROUP_ATTRIBUTE", "urn:oid:1.3.6.1.4.1.5923.1.5.1.1"),
			AttributeMapping: parseJSONMapping(getEnv("SAML_SHIBBOLETH_ATTR_MAPPING", "")),
		}
	}

	return providers
}

// Validate validates the configuration
func (c *Config) Validate() error {
	var errors []string

	// Validate AUTH_TYPE
	if c.Auth.Type != "normal" && c.Auth.Type != "oauth" && c.Auth.Type != "saml" {
		errors = append(errors, fmt.Sprintf("invalid AUTH_TYPE: %s (must be 'normal', 'oauth', or 'saml')", c.Auth.Type))
	}

	// If AUTH_TYPE is oauth, validate at least one OAuth provider is configured
	if c.Auth.Type == "oauth" {
		if len(c.OAuth.Providers) == 0 {
			errors = append(errors, "AUTH_TYPE is 'oauth' but no OAuth providers are configured")
		} else {
			// Validate OAuth provider configs
			for name, provider := range c.OAuth.Providers {
				if provider.ClientID == "" {
					errors = append(errors, fmt.Sprintf("OAuth provider '%s' missing ClientID", name))
				}
				if provider.ClientSecret == "" {
					errors = append(errors, fmt.Sprintf("OAuth provider '%s' missing ClientSecret", name))
				}
				if provider.RedirectURI == "" {
					errors = append(errors, fmt.Sprintf("OAuth provider '%s' missing RedirectURI", name))
				}
			}
		}
	}

	// If AUTH_TYPE is saml, validate SAML configuration
	if c.Auth.Type == "saml" {
		if c.SAML.CertificatePath == "" {
			errors = append(errors, "AUTH_TYPE is 'saml' but SAML_CERTIFICATE_PATH is not set")
		} else {
			// Check if certificate file exists
			if _, err := os.Stat(c.SAML.CertificatePath); os.IsNotExist(err) {
				errors = append(errors, fmt.Sprintf("SAML certificate file does not exist: %s", c.SAML.CertificatePath))
			}
		}
		if c.SAML.PrivateKeyPath == "" {
			errors = append(errors, "AUTH_TYPE is 'saml' but SAML_PRIVATE_KEY_PATH is not set")
		} else {
			// Check if private key file exists
			if _, err := os.Stat(c.SAML.PrivateKeyPath); os.IsNotExist(err) {
				errors = append(errors, fmt.Sprintf("SAML private key file does not exist: %s", c.SAML.PrivateKeyPath))
			}
		}

		// Validate SAML provider configs
		for name, provider := range c.SAML.Providers {
			if provider.Type == "okta" || provider.Type == "auth0" || provider.Type == "shibboleth" {
				if provider.MetadataURL == "" && provider.EntityID == "" {
					errors = append(errors, fmt.Sprintf("SAML provider '%s' missing both MetadataURL and EntityID", name))
				}
			} else if provider.Type == "adfs" {
				if provider.EntityID == "" {
					errors = append(errors, fmt.Sprintf("SAML provider '%s' (ADFS) missing EntityID", name))
				}
			} else if provider.Type == "custom" {
				if provider.EntityID == "" {
					errors = append(errors, fmt.Sprintf("SAML provider '%s' (custom) missing EntityID", name))
				}
				if provider.SSOURL == "" {
					errors = append(errors, fmt.Sprintf("SAML provider '%s' (custom) missing SSOURL", name))
				}
			}
		}
	}

	// Return aggregated errors
	if len(errors) > 0 {
		return fmt.Errorf("configuration validation failed:\n  - %s", strings.Join(errors, "\n  - "))
	}

	return nil
}
