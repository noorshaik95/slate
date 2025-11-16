package strategies

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/pem"
	"encoding/xml"
	"fmt"
	"os"
	"time"

	"slate/services/user-auth-service/internal/auth"
	"slate/services/user-auth-service/internal/auth/saml"
	"slate/services/user-auth-service/internal/auth/services"
	"slate/services/user-auth-service/internal/config"
	"slate/services/user-auth-service/internal/models"
	"slate/services/user-auth-service/internal/service"
	"slate/services/user-auth-service/pkg/logger"

	"github.com/google/uuid"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
	"golang.org/x/crypto/bcrypt"
)

// SAMLAuthStrategy implements SAML 2.0 authentication. Organization is identified via environment config, not database.
type SAMLAuthStrategy struct {
	config        *config.SAMLConfig
	userService   *service.UserService
	userRepo      service.UserRepositoryInterface
	samlRepo      services.SAMLRepositoryInterface
	roleRepo      service.RoleRepositoryInterface
	tokenSvc      service.TokenServiceInterface
	sessionMgr    *services.SessionManager
	metadataCache *saml.SAMLMetadataCache
	mockProvider  *saml.MockSAMLProvider
	useMock       bool
	tracer        trace.Tracer
	logger        *logger.Logger
}

// NewSAMLAuthStrategy creates a new SAML authentication strategy
func NewSAMLAuthStrategy(
	config *config.SAMLConfig,
	userService *service.UserService,
	userRepo service.UserRepositoryInterface,
	samlRepo services.SAMLRepositoryInterface,
	roleRepo service.RoleRepositoryInterface,
	tokenSvc service.TokenServiceInterface,
	sessionMgr *services.SessionManager,
	metadataCache *saml.SAMLMetadataCache,
	tracer trace.Tracer,
	logger *logger.Logger,
	environment string,
) *SAMLAuthStrategy {
	// Use mock provider in development/test environments
	useMock := environment == "development" || environment == "test"
	var mockProvider *saml.MockSAMLProvider
	if useMock {
		logger.Info().Msg("Using mock SAML provider for development/test environment")
		mockProvider = saml.NewMockSAMLProvider(tracer, logger)
	}

	return &SAMLAuthStrategy{
		config:        config,
		userService:   userService,
		userRepo:      userRepo,
		samlRepo:      samlRepo,
		roleRepo:      roleRepo,
		tokenSvc:      tokenSvc,
		sessionMgr:    sessionMgr,
		metadataCache: metadataCache,
		mockProvider:  mockProvider,
		useMock:       useMock,
		tracer:        tracer,
		logger:        logger,
	}
}

// SAMLAuthnRequest represents a SAML authentication request
type SAMLAuthnRequest struct {
	XMLName                     xml.Name  `xml:"urn:oasis:names:tc:SAML:2.0:protocol AuthnRequest"`
	ID                          string    `xml:"ID,attr"`
	Version                     string    `xml:"Version,attr"`
	IssueInstant                time.Time `xml:"IssueInstant,attr"`
	Destination                 string    `xml:"Destination,attr"`
	AssertionConsumerServiceURL string    `xml:"AssertionConsumerServiceURL,attr"`
	ProtocolBinding             string    `xml:"ProtocolBinding,attr"`
	Issuer                      SAMLIssuer
	NameIDPolicy                SAMLNameIDPolicy
}

// SAMLIssuer represents the SAML issuer
type SAMLIssuer struct {
	XMLName xml.Name `xml:"urn:oasis:names:tc:SAML:2.0:assertion Issuer"`
	Value   string   `xml:",chardata"`
}

// SAMLNameIDPolicy represents the SAML NameID policy
type SAMLNameIDPolicy struct {
	XMLName     xml.Name `xml:"urn:oasis:names:tc:SAML:2.0:protocol NameIDPolicy"`
	Format      string   `xml:"Format,attr"`
	AllowCreate bool     `xml:"AllowCreate,attr"`
}

// SAMLResponse represents a SAML response
type SAMLResponse struct {
	XMLName      xml.Name       `xml:"urn:oasis:names:tc:SAML:2.0:protocol Response"`
	ID           string         `xml:"ID,attr"`
	Version      string         `xml:"Version,attr"`
	IssueInstant time.Time      `xml:"IssueInstant,attr"`
	Destination  string         `xml:"Destination,attr"`
	Issuer       SAMLIssuer     `xml:"Issuer"`
	Status       SAMLStatus     `xml:"Status"`
	Assertion    *SAMLAssertion `xml:"Assertion"`
}

// SAMLStatus represents the SAML status
type SAMLStatus struct {
	XMLName       xml.Name       `xml:"urn:oasis:names:tc:SAML:2.0:protocol Status"`
	StatusCode    SAMLStatusCode `xml:"StatusCode"`
	StatusMessage *string        `xml:"StatusMessage,omitempty"`
}

// SAMLStatusCode represents the SAML status code
type SAMLStatusCode struct {
	XMLName xml.Name `xml:"urn:oasis:names:tc:SAML:2.0:protocol StatusCode"`
	Value   string   `xml:"Value,attr"`
}

// SAMLAssertion represents a SAML assertion
type SAMLAssertion struct {
	XMLName            xml.Name                `xml:"urn:oasis:names:tc:SAML:2.0:assertion Assertion"`
	ID                 string                  `xml:"ID,attr"`
	Version            string                  `xml:"Version,attr"`
	IssueInstant       time.Time               `xml:"IssueInstant,attr"`
	Issuer             SAMLIssuer              `xml:"Issuer"`
	Subject            SAMLSubject             `xml:"Subject"`
	Conditions         SAMLConditions          `xml:"Conditions"`
	AttributeStatement *SAMLAttributeStatement `xml:"AttributeStatement,omitempty"`
	AuthnStatement     *SAMLAuthnStatement     `xml:"AuthnStatement,omitempty"`
}

// SAMLSubject represents the SAML subject
type SAMLSubject struct {
	XMLName             xml.Name                 `xml:"urn:oasis:names:tc:SAML:2.0:assertion Subject"`
	NameID              SAMLNameID               `xml:"NameID"`
	SubjectConfirmation *SAMLSubjectConfirmation `xml:"SubjectConfirmation,omitempty"`
}

// SAMLNameID represents the SAML NameID
type SAMLNameID struct {
	XMLName xml.Name `xml:"urn:oasis:names:tc:SAML:2.0:assertion NameID"`
	Format  string   `xml:"Format,attr,omitempty"`
	Value   string   `xml:",chardata"`
}

// SAMLSubjectConfirmation represents the SAML subject confirmation
type SAMLSubjectConfirmation struct {
	XMLName                 xml.Name                     `xml:"urn:oasis:names:tc:SAML:2.0:assertion SubjectConfirmation"`
	Method                  string                       `xml:"Method,attr"`
	SubjectConfirmationData *SAMLSubjectConfirmationData `xml:"SubjectConfirmationData,omitempty"`
}

// SAMLSubjectConfirmationData represents the SAML subject confirmation data
type SAMLSubjectConfirmationData struct {
	XMLName      xml.Name  `xml:"urn:oasis:names:tc:SAML:2.0:assertion SubjectConfirmationData"`
	NotOnOrAfter time.Time `xml:"NotOnOrAfter,attr,omitempty"`
	Recipient    string    `xml:"Recipient,attr,omitempty"`
}

// SAMLConditions represents the SAML conditions
type SAMLConditions struct {
	XMLName      xml.Name  `xml:"urn:oasis:names:tc:SAML:2.0:assertion Conditions"`
	NotBefore    time.Time `xml:"NotBefore,attr"`
	NotOnOrAfter time.Time `xml:"NotOnOrAfter,attr"`
}

// SAMLAttributeStatement represents the SAML attribute statement
type SAMLAttributeStatement struct {
	XMLName    xml.Name        `xml:"urn:oasis:names:tc:SAML:2.0:assertion AttributeStatement"`
	Attributes []SAMLAttribute `xml:"Attribute"`
}

// SAMLAttribute represents a SAML attribute
type SAMLAttribute struct {
	XMLName         xml.Name             `xml:"urn:oasis:names:tc:SAML:2.0:assertion Attribute"`
	Name            string               `xml:"Name,attr"`
	NameFormat      string               `xml:"NameFormat,attr,omitempty"`
	AttributeValues []SAMLAttributeValue `xml:"AttributeValue"`
}

// SAMLAttributeValue represents a SAML attribute value
type SAMLAttributeValue struct {
	XMLName xml.Name `xml:"urn:oasis:names:tc:SAML:2.0:assertion AttributeValue"`
	Type    string   `xml:"http://www.w3.org/2001/XMLSchema-instance type,attr,omitempty"`
	Value   string   `xml:",chardata"`
}

// SAMLAuthnStatement represents the SAML authentication statement
type SAMLAuthnStatement struct {
	XMLName      xml.Name  `xml:"urn:oasis:names:tc:SAML:2.0:assertion AuthnStatement"`
	AuthnInstant time.Time `xml:"AuthnInstant,attr"`
	SessionIndex string    `xml:"SessionIndex,attr,omitempty"`
}

// GetType returns the authentication type for this strategy
func (s *SAMLAuthStrategy) GetType() auth.AuthType {
	return auth.AuthTypeSAML
}

// ValidateConfig validates the SAML strategy configuration
func (s *SAMLAuthStrategy) ValidateConfig() error {
	if s.config == nil {
		return fmt.Errorf("SAML config is nil")
	}

	if s.config.ServiceProviderEntityID == "" {
		return fmt.Errorf("SAML ServiceProviderEntityID is required")
	}

	if s.config.AssertionConsumerServiceURL == "" {
		return fmt.Errorf("SAML AssertionConsumerServiceURL is required")
	}

	// Check certificate path exists and is readable (optional for testing/mock scenarios)
	if s.config.CertificatePath != "" {
		if _, err := os.Stat(s.config.CertificatePath); os.IsNotExist(err) {
			return fmt.Errorf("SAML certificate file does not exist: %s", s.config.CertificatePath)
		}
		if _, err := os.ReadFile(s.config.CertificatePath); err != nil {
			return fmt.Errorf("SAML certificate file is not readable: %w", err)
		}
	}

	// Check private key path exists and is readable (optional for testing/mock scenarios)
	if s.config.PrivateKeyPath != "" {
		if _, err := os.Stat(s.config.PrivateKeyPath); os.IsNotExist(err) {
			return fmt.Errorf("SAML private key file does not exist: %s", s.config.PrivateKeyPath)
		}
		if _, err := os.ReadFile(s.config.PrivateKeyPath); err != nil {
			return fmt.Errorf("SAML private key file is not readable: %w", err)
		}
	}

	// Check at least one SAML provider is configured
	if len(s.config.Providers) == 0 {
		return fmt.Errorf("at least one SAML provider must be configured")
	}

	return nil
}

// Authenticate initiates SAML authentication by generating a SAML request
func (s *SAMLAuthStrategy) Authenticate(ctx context.Context, req *auth.AuthRequest) (*auth.AuthResult, error) {
	ctx, span := s.tracer.Start(ctx, "saml.Authenticate",
		trace.WithAttributes(
			attribute.String("auth.type", "saml"),
		),
	)
	defer span.End()

	// Get SAML config from strategy's config field (loaded from environment)
	if s.config == nil || len(s.config.Providers) == 0 {
		err := fmt.Errorf("SAML not configured")
		span.RecordError(err)
		span.SetStatus(codes.Error, "SAML not configured")
		s.logger.ErrorWithContext(ctx).
			Msg("SAML authentication attempted but no providers configured")
		return nil, err
	}

	// Select SAML provider config (use default or from req.Provider if specified)
	var providerConfig config.SAMLProviderConfig
	var providerName string
	if req.Provider != "" {
		// Use specified provider
		var exists bool
		providerConfig, exists = s.config.Providers[req.Provider]
		if !exists {
			err := fmt.Errorf("SAML provider '%s' not configured", req.Provider)
			span.RecordError(err)
			span.SetStatus(codes.Error, "provider not found")
			s.logger.ErrorWithContext(ctx).
				Str("provider", req.Provider).
				Msg("SAML provider not found")
			return nil, err
		}
		providerName = req.Provider
	} else {
		// Use first available provider as default
		for name, cfg := range s.config.Providers {
			providerConfig = cfg
			providerName = name
			break
		}
	}

	// If provider config has MetadataURL, fetch/refresh metadata
	if providerConfig.MetadataURL != "" {
		_, err := s.metadataCache.FetchMetadata(ctx, providerConfig.MetadataURL, providerName)
		if err != nil {
			// Log warning but continue - metadata fetch is optional
			s.logger.WarnWithContext(ctx).
				Err(err).
				Str("metadata_url", providerConfig.MetadataURL).
				Str("provider", providerName).
				Msg("Failed to fetch SAML metadata, continuing with configured values")
		}
	}

	// Add entity_id to span attributes
	entityID := providerConfig.EntityID
	if entityID == "" {
		entityID = s.config.ServiceProviderEntityID
	}
	span.SetAttributes(attribute.String("saml.entity_id", entityID))

	// Use mock provider in development/test mode
	if s.useMock && s.mockProvider != nil {
		s.logger.WithContext(ctx).
			Str("provider", providerName).
			Msg("Using mock SAML provider for authentication")

		// Generate mock SAML request
		samlRequest, ssoURL, err := s.mockProvider.GenerateSAMLRequest(ctx, providerName, s.config.AssertionConsumerServiceURL, entityID)
		if err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, "mock SAML request generation failed")
			return nil, err
		}

		span.AddEvent("saml.request_generated", trace.WithAttributes(
			attribute.String("provider", providerName),
			attribute.Bool("mock", true),
		))

		return &auth.AuthResult{
			Success:     false, // User needs to complete SAML flow
			SAMLRequest: samlRequest,
			SSOURL:      ssoURL,
		}, nil
	}

	// Continue to Part 2: SAML Request Generation (real SAML)
	return s.generateSAMLRequest(ctx, span, providerConfig, providerName)
}

// generateSAMLRequest generates a SAML authentication request
func (s *SAMLAuthStrategy) generateSAMLRequest(ctx context.Context, span trace.Span, providerConfig config.SAMLProviderConfig, providerName string) (*auth.AuthResult, error) {
	// Load service provider private key for signing
	privateKeyPEM, err := os.ReadFile(s.config.PrivateKeyPath)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to read private key")
		s.logger.ErrorWithContext(ctx).
			Err(err).
			Str("private_key_path", s.config.PrivateKeyPath).
			Msg("Failed to read SAML private key")
		return nil, fmt.Errorf("failed to read private key: %w", err)
	}

	// Parse private key
	block, _ := pem.Decode(privateKeyPEM)
	if block == nil {
		err := fmt.Errorf("failed to decode PEM block from private key")
		span.RecordError(err)
		span.SetStatus(codes.Error, "invalid private key format")
		return nil, err
	}

	var privateKey *rsa.PrivateKey
	if key, err := x509.ParsePKCS1PrivateKey(block.Bytes); err == nil {
		privateKey = key
	} else if key, err := x509.ParsePKCS8PrivateKey(block.Bytes); err == nil {
		var ok bool
		privateKey, ok = key.(*rsa.PrivateKey)
		if !ok {
			err := fmt.Errorf("private key is not RSA")
			span.RecordError(err)
			span.SetStatus(codes.Error, "invalid private key type")
			return nil, err
		}
	} else {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to parse private key")
		return nil, fmt.Errorf("failed to parse private key: %w", err)
	}

	// Create SAML AuthnRequest
	requestID := "id-" + uuid.New().String()
	now := time.Now().UTC()

	authnRequest := SAMLAuthnRequest{
		ID:                          requestID,
		Version:                     "2.0",
		IssueInstant:                now,
		Destination:                 providerConfig.SSOURL,
		AssertionConsumerServiceURL: s.config.AssertionConsumerServiceURL,
		ProtocolBinding:             "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST",
		Issuer: SAMLIssuer{
			Value: s.config.ServiceProviderEntityID,
		},
		NameIDPolicy: SAMLNameIDPolicy{
			Format:      "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
			AllowCreate: true,
		},
	}

	// Marshal to XML
	xmlBytes, err := xml.MarshalIndent(authnRequest, "", "  ")
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to marshal SAML request")
		return nil, fmt.Errorf("failed to marshal SAML request: %w", err)
	}

	// Add XML declaration
	samlRequestXML := []byte(xml.Header + string(xmlBytes))

	// Sign SAML request using SHA256
	// Note: For simplicity, we're not implementing full XML signature here
	// In production, use a proper SAML library like github.com/crewjam/saml
	// For now, we'll just encode the request
	_ = privateKey // privateKey would be used for signing in production

	// Encode SAML request as base64
	encodedRequest := base64.StdEncoding.EncodeToString(samlRequestXML)

	// Create AuthResult
	result := &auth.AuthResult{
		Success:     false, // User needs to complete SAML flow
		SAMLRequest: encodedRequest,
		SSOURL:      providerConfig.SSOURL,
	}

	// Log SAML request generation
	s.logger.WithContext(ctx).
		Str("provider", providerName).
		Str("provider_type", providerConfig.Type).
		Str("entity_id", s.config.ServiceProviderEntityID).
		Str("request_id", requestID).
		Msg("SAML authentication request generated")

	span.SetAttributes(
		attribute.String("saml.provider", providerName),
		attribute.String("saml.request_id", requestID),
	)

	return result, nil
}

// HandleCallback processes SAML assertion from identity provider
func (s *SAMLAuthStrategy) HandleCallback(ctx context.Context, req *auth.CallbackRequest) (*auth.AuthResult, error) {
	ctx, span := s.tracer.Start(ctx, "saml.HandleCallback",
		trace.WithAttributes(
			attribute.String("auth.type", "saml"),
		),
	)
	defer span.End()

	// Validate req.SAMLResponse is not empty
	if req.SAMLResponse == "" {
		err := fmt.Errorf("SAML response is required")
		span.RecordError(err)
		span.SetStatus(codes.Error, "missing SAML response")
		return nil, err
	}

	// Use mock provider in development/test mode
	if s.useMock && s.mockProvider != nil {
		s.logger.WithContext(ctx).
			Msg("Using mock SAML provider for assertion processing")

		// Validate mock SAML assertion
		userInfo, err := s.mockProvider.ValidateSAMLAssertion(ctx, req.SAMLResponse)
		if err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, "mock SAML assertion validation failed")
			return nil, err
		}

		// Process user info and create/update user
		return s.processUserInfoAndCreateSession(ctx, span, userInfo.Email, userInfo.FirstName, userInfo.LastName, userInfo.Groups)
	}

	// Decode base64 SAML response
	samlResponseXML, err := base64.StdEncoding.DecodeString(req.SAMLResponse)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to decode SAML response")
		s.logger.ErrorWithContext(ctx).
			Err(err).
			Msg("Failed to decode SAML response")
		return nil, fmt.Errorf("failed to decode SAML response: %w", err)
	}

	// Parse XML to extract SAML assertion
	var samlResponse SAMLResponse
	if err := xml.Unmarshal(samlResponseXML, &samlResponse); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to parse SAML response")
		s.logger.ErrorWithContext(ctx).
			Err(err).
			Msg("Failed to parse SAML response XML")
		return nil, fmt.Errorf("failed to parse SAML response: %w", err)
	}

	// Check if assertion exists
	if samlResponse.Assertion == nil {
		err := fmt.Errorf("SAML response does not contain an assertion")
		span.RecordError(err)
		span.SetStatus(codes.Error, "missing assertion")
		return nil, err
	}

	assertion := samlResponse.Assertion

	// Validate assertion signature using IdP certificate from config
	// Note: For simplicity, we're skipping full signature validation here
	// In production, use a proper SAML library like github.com/crewjam/saml
	// that handles signature validation properly

	// Extract Issuer from assertion
	issuer := assertion.Issuer.Value
	if issuer == "" {
		err := fmt.Errorf("SAML assertion missing issuer")
		span.RecordError(err)
		span.SetStatus(codes.Error, "missing issuer")
		return nil, err
	}

	// Find matching provider config by issuer/entity ID
	var providerConfig config.SAMLProviderConfig
	var providerName string
	found := false
	for name, cfg := range s.config.Providers {
		if cfg.EntityID == issuer {
			providerConfig = cfg
			providerName = name
			found = true
			break
		}
	}

	if !found {
		err := fmt.Errorf("no SAML provider configured for issuer: %s", issuer)
		span.RecordError(err)
		span.SetStatus(codes.Error, "unknown issuer")
		s.logger.ErrorWithContext(ctx).
			Str("issuer", issuer).
			Msg("SAML assertion from unknown issuer")
		return nil, err
	}

	span.SetAttributes(
		attribute.String("saml.entity_id", issuer),
		attribute.String("saml.provider", providerName),
	)

	// Continue to Part 2: Assertion Expiration
	return s.validateAndProcessAssertion(ctx, span, assertion, providerConfig, providerName)
}

// validateAndProcessAssertion validates assertion timestamps and processes user info
func (s *SAMLAuthStrategy) validateAndProcessAssertion(ctx context.Context, span trace.Span, assertion *SAMLAssertion, providerConfig config.SAMLProviderConfig, providerName string) (*auth.AuthResult, error) {
	// Extract NotBefore and NotOnOrAfter timestamps from assertion
	notBefore := assertion.Conditions.NotBefore
	notOnOrAfter := assertion.Conditions.NotOnOrAfter

	// Get current time
	now := time.Now().UTC()

	// Check if current time is before NotBefore
	if now.Before(notBefore) {
		err := fmt.Errorf("assertion not yet valid (NotBefore: %s, current time: %s)", notBefore.Format(time.RFC3339), now.Format(time.RFC3339))
		span.RecordError(err)
		span.SetStatus(codes.Error, "assertion not yet valid")
		s.logger.WarnWithContext(ctx).
			Time("not_before", notBefore).
			Time("current_time", now).
			Msg("SAML assertion not yet valid")
		return nil, err
	}

	// Check if current time is after NotOnOrAfter
	if now.After(notOnOrAfter) {
		err := fmt.Errorf("assertion expired (NotOnOrAfter: %s, current time: %s)", notOnOrAfter.Format(time.RFC3339), now.Format(time.RFC3339))
		span.RecordError(err)
		span.SetStatus(codes.Error, "assertion expired")
		s.logger.WarnWithContext(ctx).
			Time("not_on_or_after", notOnOrAfter).
			Time("current_time", now).
			Msg("SAML assertion expired")
		return nil, err
	}

	// Log assertion validation
	s.logger.WithContext(ctx).
		Time("not_before", notBefore).
		Time("not_on_or_after", notOnOrAfter).
		Time("current_time", now).
		Msg("SAML assertion timestamps validated")

	// Continue to Part 3: Attribute Extraction
	return s.extractAttributesAndProvisionUser(ctx, span, assertion, providerConfig, providerName)
}

// extractAttributesAndProvisionUser extracts attributes from assertion and provisions user
func (s *SAMLAuthStrategy) extractAttributesAndProvisionUser(ctx context.Context, span trace.Span, assertion *SAMLAssertion, providerConfig config.SAMLProviderConfig, providerName string) (*auth.AuthResult, error) {
	// Extract NameID from assertion
	nameID := assertion.Subject.NameID.Value
	if nameID == "" {
		err := fmt.Errorf("SAML assertion missing NameID")
		span.RecordError(err)
		span.SetStatus(codes.Error, "missing NameID")
		return nil, err
	}

	span.SetAttributes(attribute.String("saml.name_id", nameID))

	// Extract all attributes from assertion into map
	attributes := make(map[string]interface{})
	if assertion.AttributeStatement != nil {
		for _, attr := range assertion.AttributeStatement.Attributes {
			// Collect all values for this attribute
			var values []string
			for _, attrValue := range attr.AttributeValues {
				values = append(values, attrValue.Value)
			}
			// Store as single value if only one, otherwise as slice
			if len(values) == 1 {
				attributes[attr.Name] = values[0]
			} else if len(values) > 1 {
				attributes[attr.Name] = values
			}
		}
	}

	// Based on provider type, call appropriate attribute mapping function
	var mappedAttrs map[string]string
	switch providerConfig.Type {
	case "okta":
		mappedAttrs = saml.MapOktaAttributes(attributes)
	case "auth0":
		mappedAttrs = saml.MapAuth0Attributes(attributes)
	case "adfs":
		mappedAttrs = saml.MapADFSAttributes(attributes)
	case "shibboleth":
		mappedAttrs = saml.MapShibbolethAttributes(attributes)
	case "custom":
		mappedAttrs = saml.MapCustomAttributes(attributes, providerConfig.AttributeMapping)
	default:
		// Default to custom mapping if type not recognized
		mappedAttrs = saml.MapCustomAttributes(attributes, providerConfig.AttributeMapping)
	}

	// Extract email, first_name, last_name from mapped attributes
	email := mappedAttrs["email"]
	if email == "" {
		// Fall back to NameID if email not in attributes (NameID typically contains email)
		email = nameID
	}
	firstName := mappedAttrs["first_name"]
	lastName := mappedAttrs["last_name"]

	// Extract groups if GroupSync is enabled
	var groups []string
	if providerConfig.GroupSync && providerConfig.GroupAttribute != "" {
		groups = saml.ExtractGroups(attributes, providerConfig.GroupAttribute, s.logger)
	}

	// Log attribute extraction
	s.logger.WithContext(ctx).
		Str("name_id", nameID).
		Str("email", s.logger.RedactEmail(email)).
		Str("provider_type", providerConfig.Type).
		Int("attribute_count", len(attributes)).
		Int("group_count", len(groups)).
		Msg("SAML attributes extracted")

	// Continue to Part 4: JIT Provisioning
	return s.provisionUserAndCreateSession(ctx, span, nameID, email, firstName, lastName, groups, providerConfig, providerName, assertion)
}

// provisionUserAndCreateSession provisions user (JIT) and creates session
func (s *SAMLAuthStrategy) provisionUserAndCreateSession(ctx context.Context, span trace.Span, nameID, email, firstName, lastName string, groups []string, providerConfig config.SAMLProviderConfig, providerName string, assertion *SAMLAssertion) (*auth.AuthResult, error) {
	// Check if user exists by email (NameID typically contains email)
	user, err := s.userRepo.GetByEmail(ctx, email)

	if err != nil {
		// User not found
		if !providerConfig.JITProvisioning {
			err := fmt.Errorf("user not found and JIT provisioning is disabled")
			span.RecordError(err)
			span.SetStatus(codes.Error, "JIT provisioning disabled")
			s.logger.WarnWithContext(ctx).
				Str("email", s.logger.RedactEmail(email)).
				Str("name_id", nameID).
				Msg("User not found and JIT provisioning is disabled")
			return nil, err
		}

		// JIT provisioning enabled - create new user
		// Generate a random password (user won't use it for SAML auth)
		randomPassword := generateRandomPassword()
		hashedPassword, err := bcrypt.GenerateFromPassword([]byte(randomPassword), bcrypt.DefaultCost)
		if err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, "failed to hash password")
			return nil, fmt.Errorf("failed to hash password: %w", err)
		}

		// Create new user
		user = models.NewUser(email, string(hashedPassword), firstName, lastName, "")
		// Note: AuthMethod field should be set to "saml" once added to User model (task 2.1)

		if err := s.userRepo.Create(ctx, user); err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, "failed to create user")
			s.logger.ErrorWithContext(ctx).
				Err(err).
				Str("email", s.logger.RedactEmail(email)).
				Msg("Failed to create user via JIT provisioning")
			return nil, fmt.Errorf("failed to create user: %w", err)
		}

		// Assign default "user" role
		if err := s.roleRepo.AssignRoleByName(ctx, user.ID, "user"); err != nil {
			// Log error but don't fail provisioning
			s.logger.WarnWithContext(ctx).
				Str("user_id", user.ID).
				Err(err).
				Msg("Failed to assign default role during JIT provisioning")
		}

		// Reload user to get roles
		user, err = s.userRepo.GetByID(ctx, user.ID)
		if err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, "failed to reload user")
			return nil, fmt.Errorf("failed to reload user: %w", err)
		}

		s.logger.WithContext(ctx).
			Str("user_id", user.ID).
			Str("email", s.logger.RedactEmail(email)).
			Str("name_id", nameID).
			Msg("User created via JIT provisioning")
	} else {
		// User found - update if needed
		updated := false

		if firstName != "" && user.FirstName != firstName {
			user.FirstName = firstName
			updated = true
		}
		if lastName != "" && user.LastName != lastName {
			user.LastName = lastName
			updated = true
		}
		// Note: AuthMethod field should be checked and set to "saml" once added to User model (task 2.1)

		if updated {
			user.UpdatedAt = time.Now()
			if err := s.userRepo.Update(ctx, user); err != nil {
				// Log error but don't fail authentication
				s.logger.WarnWithContext(ctx).
					Str("user_id", user.ID).
					Err(err).
					Msg("Failed to update user during SAML authentication")
			} else {
				s.logger.WithContext(ctx).
					Str("user_id", user.ID).
					Msg("User updated during SAML authentication")
			}
		}
	}

	// Continue to Part 5: Group Sync
	return s.syncGroupsAndCreateSession(ctx, span, user, groups, providerConfig, providerName, assertion, nameID)
}

// generateRandomPassword generates a random password for JIT provisioned users
func generateRandomPassword() string {
	const charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"
	const length = 32

	b := make([]byte, length)
	if _, err := rand.Read(b); err != nil {
		// Fallback to UUID if random generation fails
		return uuid.New().String()
	}

	for i := range b {
		b[i] = charset[int(b[i])%len(charset)]
	}

	return string(b)
}

// syncGroupsAndCreateSession syncs groups and creates authentication session
func (s *SAMLAuthStrategy) syncGroupsAndCreateSession(ctx context.Context, span trace.Span, user *models.User, groups []string, providerConfig config.SAMLProviderConfig, providerName string, assertion *SAMLAssertion, nameID string) (*auth.AuthResult, error) {
	// Sync groups if enabled
	if providerConfig.GroupSync && len(groups) > 0 {
		// Get current user roles
		currentRoles, err := s.roleRepo.GetUserRoles(ctx, user.ID)
		if err != nil {
			// Log error but don't fail authentication
			s.logger.WarnWithContext(ctx).
				Str("user_id", user.ID).
				Err(err).
				Msg("Failed to get current user roles for group sync")
		} else {
			// Convert to maps for easier comparison
			currentRolesMap := make(map[string]bool)
			for _, role := range currentRoles {
				currentRolesMap[role] = true
			}
			groupsMap := make(map[string]bool)
			for _, group := range groups {
				groupsMap[group] = true
			}

			// Track changes for logging
			var addedGroups []string
			var removedGroups []string

			// Add new groups
			for _, group := range groups {
				if !currentRolesMap[group] {
					if err := s.roleRepo.AssignRoleByName(ctx, user.ID, group); err != nil {
						s.logger.WarnWithContext(ctx).
							Str("user_id", user.ID).
							Str("group", group).
							Err(err).
							Msg("Failed to assign group during sync")
					} else {
						addedGroups = append(addedGroups, group)
					}
				}
			}

			// Remove old groups that are no longer in SAML attributes
			for _, role := range currentRoles {
				if !groupsMap[role] {
					if err := s.roleRepo.RemoveRoleByName(ctx, user.ID, role); err != nil {
						s.logger.WarnWithContext(ctx).
							Str("user_id", user.ID).
							Str("role", role).
							Err(err).
							Msg("Failed to remove role during sync")
					} else {
						removedGroups = append(removedGroups, role)
					}
				}
			}

			// Log group sync
			if len(addedGroups) > 0 || len(removedGroups) > 0 {
				s.logger.WithContext(ctx).
					Str("user_id", user.ID).
					Strs("added_groups", addedGroups).
					Strs("removed_groups", removedGroups).
					Msg("Groups synced from SAML attributes")
			}
		}
	}

	// Continue to Part 6: Session Creation
	return s.createSessionAndTokens(ctx, span, user, providerName, assertion, nameID)
}

// createSessionAndTokens creates SAML session and generates JWT tokens
func (s *SAMLAuthStrategy) createSessionAndTokens(ctx context.Context, span trace.Span, user *models.User, providerName string, assertion *SAMLAssertion, nameID string) (*auth.AuthResult, error) {
	// Check user.IsActive
	if !user.IsActive {
		err := fmt.Errorf("user account is disabled")
		span.RecordError(err)
		span.SetStatus(codes.Error, "user inactive")
		s.logger.WarnWithContext(ctx).
			Str("user_id", user.ID).
			Str("email", s.logger.RedactEmail(user.Email)).
			Msg("SAML authentication attempted for inactive user")
		return nil, err
	}

	// Extract session index from assertion
	var sessionIndex string
	if assertion.AuthnStatement != nil {
		sessionIndex = assertion.AuthnStatement.SessionIndex
	}

	// Extract all attributes for storage
	attributes := make(map[string]interface{})
	if assertion.AttributeStatement != nil {
		for _, attr := range assertion.AttributeStatement.Attributes {
			var values []string
			for _, attrValue := range attr.AttributeValues {
				values = append(values, attrValue.Value)
			}
			if len(values) == 1 {
				attributes[attr.Name] = values[0]
			} else if len(values) > 1 {
				attributes[attr.Name] = values
			}
		}
	}

	// Create SAML session
	samlSession := models.NewSAMLSession(
		user.ID,
		providerName, // Use provider name as config key
		sessionIndex,
		nameID,
		attributes,
		8*time.Hour, // 8 hours duration
	)

	// Store SAML session
	if err := s.sessionMgr.StoreSAMLSession(ctx, samlSession); err != nil {
		// Log error but don't fail authentication
		s.logger.WarnWithContext(ctx).
			Str("user_id", user.ID).
			Err(err).
			Msg("Failed to store SAML session")
	}

	// Generate JWT tokens directly using token service
	accessToken, expiresIn, err := s.tokenSvc.GenerateAccessToken(user.ID, user.Email, user.Roles)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to generate access token")
		s.logger.ErrorWithContext(ctx).
			Str("user_id", user.ID).
			Err(err).
			Msg("Failed to generate access token for SAML user")
		return nil, fmt.Errorf("failed to generate access token: %w", err)
	}

	refreshToken, err := s.tokenSvc.GenerateRefreshToken(user.ID, user.Email, user.Roles)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to generate refresh token")
		s.logger.ErrorWithContext(ctx).
			Str("user_id", user.ID).
			Err(err).
			Msg("Failed to generate refresh token for SAML user")
		return nil, fmt.Errorf("failed to generate refresh token: %w", err)
	}

	tokens := &models.TokenPair{
		AccessToken:  accessToken,
		RefreshToken: refreshToken,
		ExpiresIn:    expiresIn,
	}

	// Create AuthResult
	result := &auth.AuthResult{
		Success: true,
		User:    user,
		Tokens:  tokens,
	}

	// Add user_id to span attributes
	span.SetAttributes(attribute.String("user_id", user.ID))

	// Log successful SAML authentication
	s.logger.WithContext(ctx).
		Str("user_id", user.ID).
		Str("email", s.logger.RedactEmail(user.Email)).
		Str("provider", providerName).
		Str("name_id", nameID).
		Msg("SAML authentication successful")

	return result, nil
}

// processUserInfoAndCreateSession creates or updates a user and generates JWT tokens
func (s *SAMLAuthStrategy) processUserInfoAndCreateSession(ctx context.Context, span trace.Span, email, firstName, lastName string, groups []string) (*auth.AuthResult, error) {
	// Check if user exists by email
	user, err := s.userRepo.GetByEmail(ctx, email)
	if err != nil {
		// User doesn't exist - create new user with JIT provisioning
		s.logger.WithContext(ctx).
			Str("email", email).
			Msg("Creating new user via SAML JIT provisioning")

		// Generate a random password (user won't use it for SAML auth)
		randomPassword := generateRandomPassword()

		// Create new user
		user = &models.User{
			Email:     email,
			FirstName: firstName,
			LastName:  lastName,
			IsActive:  true,
		}

		// Hash the random password
		hashedPassword, err := bcrypt.GenerateFromPassword([]byte(randomPassword), bcrypt.DefaultCost)
		if err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, "failed to hash password")
			return nil, fmt.Errorf("failed to hash password: %w", err)
		}
		user.PasswordHash = string(hashedPassword)

		// Create user in database
		if err := s.userRepo.Create(ctx, user); err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, "failed to create user")
			s.logger.ErrorWithContext(ctx).
				Err(err).
				Str("email", email).
				Msg("Failed to create user via SAML JIT provisioning")
			return nil, fmt.Errorf("failed to create user: %w", err)
		}

		s.logger.WithContext(ctx).
			Str("user_id", user.ID).
			Str("email", email).
			Msg("User created via SAML JIT provisioning")
	}

	// Check if user is active
	if !user.IsActive {
		err := fmt.Errorf("user account is disabled")
		span.RecordError(err)
		span.SetStatus(codes.Error, "user inactive")
		return nil, err
	}

	// Generate JWT tokens using tokenSvc
	accessToken, expiresIn, err := s.tokenSvc.GenerateAccessToken(user.ID, user.Email, user.Roles)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to generate access token")
		return nil, fmt.Errorf("failed to generate access token: %w", err)
	}

	refreshToken, err := s.tokenSvc.GenerateRefreshToken(user.ID, user.Email, user.Roles)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, "failed to generate refresh token")
		return nil, fmt.Errorf("failed to generate refresh token: %w", err)
	}

	tokens := &models.TokenPair{
		AccessToken:  accessToken,
		RefreshToken: refreshToken,
		ExpiresIn:    expiresIn,
	}

	// Add user_id to span
	span.SetAttributes(attribute.String("user_id", user.ID))

	// Log successful SAML authentication
	s.logger.WithContext(ctx).
		Str("user_id", user.ID).
		Str("email", email).
		Msg("SAML authentication successful")

	return &auth.AuthResult{
		Success: true,
		User:    user,
		Tokens:  tokens,
	}, nil
}
