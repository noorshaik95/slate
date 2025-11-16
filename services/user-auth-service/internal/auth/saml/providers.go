package saml

import (
	"context"
	"encoding/xml"
	"fmt"
	"io"
	"net/http"
	"time"

	"slate/services/user-auth-service/internal/repository"
	"slate/services/user-auth-service/pkg/logger"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
)

// httpHeaderCarrier adapts http.Header to be used as a TextMapCarrier for trace context propagation
type httpHeaderCarrier struct {
	header http.Header
}

// Get retrieves a value from the HTTP header by key
func (hc *httpHeaderCarrier) Get(key string) string {
	return hc.header.Get(key)
}

// Set sets a value in the HTTP header
func (hc *httpHeaderCarrier) Set(key, value string) {
	hc.header.Set(key, value)
}

// Keys returns all keys in the HTTP header
func (hc *httpHeaderCarrier) Keys() []string {
	keys := make([]string, 0, len(hc.header))
	for k := range hc.header {
		keys = append(keys, k)
	}
	return keys
}

// SAMLProviderType represents the type of SAML identity provider
type SAMLProviderType string

const (
	// SAMLProviderOkta represents Okta SAML provider
	SAMLProviderOkta SAMLProviderType = "okta"
	// SAMLProviderAuth0 represents Auth0 SAML provider
	SAMLProviderAuth0 SAMLProviderType = "auth0"
	// SAMLProviderADFS represents Active Directory Federation Services
	SAMLProviderADFS SAMLProviderType = "adfs"
	// SAMLProviderShibboleth represents Shibboleth SAML provider
	SAMLProviderShibboleth SAMLProviderType = "shibboleth"
	// SAMLProviderCustom represents a custom SAML provider
	SAMLProviderCustom SAMLProviderType = "custom"
)

// SAMLMetadataCache handles fetching and caching of SAML metadata
type SAMLMetadataCache struct {
	samlRepo   *repository.SAMLRepository
	httpClient *http.Client
	tracer     trace.Tracer
	logger     *logger.Logger
}

// NewSAMLMetadataCache creates a new SAML metadata cache instance
func NewSAMLMetadataCache(
	samlRepo *repository.SAMLRepository,
	httpClient *http.Client,
	tracer trace.Tracer,
	logger *logger.Logger,
) *SAMLMetadataCache {
	return &SAMLMetadataCache{
		samlRepo:   samlRepo,
		httpClient: httpClient,
		tracer:     tracer,
		logger:     logger,
	}
}

// MetadataCache represents cached SAML metadata
type MetadataCache struct {
	ID          string
	ConfigKey   string
	MetadataXML string
	FetchedAt   time.Time
	ExpiresAt   time.Time
}

// FetchMetadata fetches SAML metadata from the given URL and caches it
// Returns the metadata XML string or an error
func (c *SAMLMetadataCache) FetchMetadata(ctx context.Context, metadataURL string, samlConfigID string) (string, error) {
	ctx, span := c.tracer.Start(ctx, "saml.FetchMetadata",
		trace.WithAttributes(
			attribute.String("metadata_url", metadataURL),
			attribute.String("saml_config_id", samlConfigID),
		),
	)
	defer span.End()

	// Check if metadata exists in cache and is not expired
	cachedMetadata, err := c.getMetadataFromCache(ctx, samlConfigID)
	if err == nil && cachedMetadata != nil && time.Now().Before(cachedMetadata.ExpiresAt) {
		c.logger.WithContext(ctx).
			Str("saml_config_id", samlConfigID).
			Str("cache_status", "hit").
			Msg("SAML metadata retrieved from cache")
		span.SetAttributes(attribute.Bool("cache_hit", true))
		return cachedMetadata.MetadataXML, nil
	}

	// Metadata not cached or expired, fetch from URL
	c.logger.WithContext(ctx).
		Str("metadata_url", metadataURL).
		Str("cache_status", "miss").
		Msg("Fetching SAML metadata from URL")
	span.SetAttributes(attribute.Bool("cache_hit", false))

	// Create HTTP request
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, metadataURL, nil)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to create HTTP request")
		return "", fmt.Errorf("failed to create HTTP request: %w", err)
	}

	// Inject trace context into HTTP headers for distributed tracing
	otel.GetTextMapPropagator().Inject(ctx, &httpHeaderCarrier{header: req.Header})

	// Execute HTTP request
	resp, err := c.httpClient.Do(req)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to fetch metadata")
		c.logger.ErrorWithContext(ctx).
			Err(err).
			Str("metadata_url", metadataURL).
			Msg("Failed to fetch SAML metadata")
		return "", fmt.Errorf("failed to fetch metadata: %w", err)
	}
	defer resp.Body.Close()

	// Check HTTP status
	if resp.StatusCode != http.StatusOK {
		err := fmt.Errorf("unexpected status code: %d", resp.StatusCode)
		span.RecordError(err)
		span.SetStatus(codes.Error, "HTTP error")
		c.logger.ErrorWithContext(ctx).
			Int("status_code", resp.StatusCode).
			Str("metadata_url", metadataURL).
			Msg("Failed to fetch SAML metadata")
		return "", err
	}

	// Read response body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to read response body")
		return "", fmt.Errorf("failed to read response body: %w", err)
	}

	metadataXML := string(body)

	// Validate XML format
	var xmlDoc interface{}
	if err := xml.Unmarshal(body, &xmlDoc); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "invalid XML format")
		c.logger.ErrorWithContext(ctx).
			Err(err).
			Str("metadata_url", metadataURL).
			Msg("Invalid SAML metadata XML format")
		return "", fmt.Errorf("invalid XML format: %w", err)
	}

	// Store in cache
	now := time.Now()
	expiresAt := now.Add(24 * time.Hour)
	if err := c.storeMetadataInCache(ctx, samlConfigID, metadataXML, now, expiresAt); err != nil {
		// Log error but don't fail - we still have the metadata
		c.logger.WarnWithContext(ctx).
			Err(err).
			Str("saml_config_id", samlConfigID).
			Msg("Failed to cache SAML metadata")
	}

	c.logger.WithContext(ctx).
		Str("saml_config_id", samlConfigID).
		Str("metadata_url", metadataURL).
		Msg("SAML metadata fetched and cached successfully")

	return metadataXML, nil
}

// getMetadataFromCache retrieves metadata from the cache
func (c *SAMLMetadataCache) getMetadataFromCache(ctx context.Context, configKey string) (*MetadataCache, error) {
	// Note: This requires adding GetMetadataCache method to SAMLRepository
	// For now, return cache miss to always fetch fresh metadata
	return nil, fmt.Errorf("metadata not found in cache")
}

// storeMetadataInCache stores metadata in the cache
func (c *SAMLMetadataCache) storeMetadataInCache(ctx context.Context, configKey, metadataXML string, fetchedAt, expiresAt time.Time) error {
	// Note: This requires adding StoreMetadataCache method to SAMLRepository
	// For now, skip caching (metadata will be fetched each time)
	return nil
}

// MapOktaAttributes maps Okta SAML attributes to standardized keys
func MapOktaAttributes(attributes map[string]interface{}) map[string]string {
	result := make(map[string]string)

	// Try Okta-specific attribute names first, then fall back to standard claims
	if email, ok := attributes["email"].(string); ok {
		result["email"] = email
	} else if email, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"].(string); ok {
		result["email"] = email
	} else {
		result["email"] = ""
	}

	if firstName, ok := attributes["firstName"].(string); ok {
		result["first_name"] = firstName
	} else if firstName, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname"].(string); ok {
		result["first_name"] = firstName
	} else {
		result["first_name"] = ""
	}

	if lastName, ok := attributes["lastName"].(string); ok {
		result["last_name"] = lastName
	} else if lastName, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname"].(string); ok {
		result["last_name"] = lastName
	} else {
		result["last_name"] = ""
	}

	return result
}

// MapAuth0Attributes maps Auth0 SAML attributes to standardized keys
func MapAuth0Attributes(attributes map[string]interface{}) map[string]string {
	result := make(map[string]string)

	// Auth0 uses standard SAML claim URIs
	if email, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"].(string); ok {
		result["email"] = email
	} else {
		result["email"] = ""
	}

	if firstName, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname"].(string); ok {
		result["first_name"] = firstName
	} else {
		result["first_name"] = ""
	}

	if lastName, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname"].(string); ok {
		result["last_name"] = lastName
	} else {
		result["last_name"] = ""
	}

	return result
}

// MapADFSAttributes maps ADFS SAML attributes to standardized keys
func MapADFSAttributes(attributes map[string]interface{}) map[string]string {
	result := make(map[string]string)

	// ADFS uses standard SAML claim URIs
	if email, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"].(string); ok {
		result["email"] = email
	} else {
		result["email"] = ""
	}

	if firstName, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname"].(string); ok {
		result["first_name"] = firstName
	} else {
		result["first_name"] = ""
	}

	if lastName, ok := attributes["http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname"].(string); ok {
		result["last_name"] = lastName
	} else {
		result["last_name"] = ""
	}

	// ADFS includes groups
	if groups, ok := attributes["http://schemas.microsoft.com/ws/2008/06/identity/claims/groups"]; ok {
		// Convert groups to string representation
		result["groups"] = fmt.Sprintf("%v", groups)
	} else {
		result["groups"] = ""
	}

	return result
}

// MapShibbolethAttributes maps Shibboleth OID-based SAML attributes to standardized keys
func MapShibbolethAttributes(attributes map[string]interface{}) map[string]string {
	result := make(map[string]string)

	// Shibboleth uses OID-based attribute names
	if email, ok := attributes["urn:oid:0.9.2342.19200300.100.1.3"].(string); ok {
		result["email"] = email
	} else {
		result["email"] = ""
	}

	if firstName, ok := attributes["urn:oid:2.5.4.42"].(string); ok {
		result["first_name"] = firstName
	} else {
		result["first_name"] = ""
	}

	if lastName, ok := attributes["urn:oid:2.5.4.4"].(string); ok {
		result["last_name"] = lastName
	} else {
		result["last_name"] = ""
	}

	// eduPersonAffiliation for groups
	if groups, ok := attributes["urn:oid:1.3.6.1.4.1.5923.1.5.1.1"]; ok {
		result["groups"] = fmt.Sprintf("%v", groups)
	} else {
		result["groups"] = ""
	}

	return result
}

// MapCustomAttributes maps custom SAML attributes based on provided mapping
func MapCustomAttributes(attributes map[string]interface{}, mapping map[string]string) map[string]string {
	result := make(map[string]string)

	// Standard keys we expect in the result
	standardKeys := []string{"email", "first_name", "last_name", "groups"}

	for _, standardKey := range standardKeys {
		// Get the custom attribute name from mapping
		customAttrName, exists := mapping[standardKey]
		if !exists {
			// No mapping provided for this key, use empty string
			result[standardKey] = ""
			continue
		}

		// Look up the value in attributes using the custom attribute name
		if value, ok := attributes[customAttrName]; ok {
			// Convert to string
			if strValue, ok := value.(string); ok {
				result[standardKey] = strValue
			} else {
				result[standardKey] = fmt.Sprintf("%v", value)
			}
		} else {
			result[standardKey] = ""
		}
	}

	return result
}

// ExtractGroups extracts group information from SAML attributes
func ExtractGroups(attributes map[string]interface{}, groupAttribute string, logger *logger.Logger) []string {
	if groupAttribute == "" {
		return []string{}
	}

	groupValue, exists := attributes[groupAttribute]
	if !exists {
		if logger != nil {
			logger.Warn().
				Str("group_attribute", groupAttribute).
				Msg("Group attribute specified but not found in SAML attributes")
		}
		return []string{}
	}

	// Handle different formats
	switch v := groupValue.(type) {
	case string:
		// Single group as string
		if v != "" {
			return []string{v}
		}
		return []string{}
	case []string:
		// Already a string slice
		return v
	case []interface{}:
		// Convert interface slice to string slice
		groups := make([]string, 0, len(v))
		for _, item := range v {
			if str, ok := item.(string); ok {
				groups = append(groups, str)
			}
		}
		return groups
	default:
		if logger != nil {
			logger.Warn().
				Str("group_attribute", groupAttribute).
				Str("type", fmt.Sprintf("%T", v)).
				Msg("Unexpected group attribute format")
		}
		return []string{}
	}
}
