# Implementation Guide - New Authentication Features

This guide explains how to complete the implementation of the new authentication features that have been added to the user-auth service.

## What's Already Done ✅

### 1. Database Schema
- ✅ Migration `002_enhanced_features.sql` created with all necessary tables
- ✅ Tables: `oauth_providers`, `user_mfa`, `saml_configs`, `saml_sessions`, `user_groups`, `group_members`, `parent_child_accounts`
- ✅ New user fields: `timezone`, `avatar_url`, `bio`, `organization_id`
- ✅ New roles added: `student`, `instructor`, `superadmin`

### 2. Models
- ✅ `internal/models/oauth.go` - OAuth provider models
- ✅ `internal/models/mfa.go` - MFA configuration models
- ✅ `internal/models/saml.go` - SAML configuration models
- ✅ `internal/models/group.go` - User groups models
- ✅ `internal/models/parent_child.go` - Parent-child account models
- ✅ Updated `User` and `Profile` models with new fields

### 3. Repositories
- ✅ `internal/repository/oauth_repository.go` - OAuth CRUD operations
- ✅ `internal/repository/mfa_repository.go` - MFA CRUD operations
- ✅ `internal/repository/saml_repository.go` - SAML CRUD operations
- ✅ `internal/repository/group_repository.go` - Groups CRUD operations
- ✅ `internal/repository/parent_child_repository.go` - Parent-child CRUD operations
- ✅ Updated `UserRepository` to handle new fields

### 4. Utilities
- ✅ `pkg/totp/totp.go` - TOTP/MFA utilities (secret generation, QR codes, validation)

### 5. Proto Definitions
- ✅ `proto/user.proto` updated with all new RPC methods
- ✅ Message definitions for OAuth, MFA, Groups, ParentChild

### 6. API Gateway Configuration
- ✅ Route overrides for all new endpoints
- ✅ Public routes marked (login, register, oauth callback)
- ✅ Auto-discovery enabled for RESTful endpoints

### 7. Documentation
- ✅ `FEATURES.md` - Comprehensive feature documentation
- ✅ Unit tests updated for new fields

## What Needs to Be Done 🔨

### Step 1: Regenerate Proto Code

Run the following command to regenerate the gRPC code from proto definitions:

```bash
cd /home/user/slate/services/user-auth-service
make proto
```

This will generate:
- `api/proto/user.pb.go` - Message definitions
- `api/proto/user_grpc.pb.go` - gRPC service stubs

**Required tools:**
- `protoc` - Protocol Buffer Compiler
- `protoc-gen-go` - Go proto plugin
- `protoc-gen-go-grpc` - Go gRPC plugin

Install if missing:
```bash
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
```

### Step 2: Create Service Methods

Create service layer methods for the new features. Here's the structure:

#### 2.1 OAuth Service Methods (`internal/service/oauth_service.go`)

```go
package service

import (
    "context"
    "slate/services/user-auth-service/internal/models"
    "slate/services/user-auth-service/internal/repository"
)

type OAuthService struct {
    oauthRepo *repository.OAuthRepository
    userRepo  *repository.UserRepository
    tokenSvc  TokenServiceInterface
}

func NewOAuthService(oauthRepo *repository.OAuthRepository, userRepo *repository.UserRepository, tokenSvc TokenServiceInterface) *OAuthService {
    return &OAuthService{
        oauthRepo: oauthRepo,
        userRepo:  userRepo,
        tokenSvc:  tokenSvc,
    }
}

// OAuthCallback handles OAuth provider callbacks
func (s *OAuthService) OAuthCallback(ctx context.Context, provider, code, state string) (*models.User, *models.TokenPair, bool, error) {
    // TODO: Implement OAuth callback logic
    // 1. Exchange code for access token with OAuth provider
    // 2. Get user info from OAuth provider
    // 3. Check if user exists by provider_user_id
    // 4. If exists, link and return user + tokens
    // 5. If not exists, create new user and return
    return nil, nil, false, nil
}

// LinkOAuthProvider links an OAuth provider to an existing user
func (s *OAuthService) LinkOAuthProvider(ctx context.Context, userID, provider, providerUserID, accessToken, refreshToken string, tokenExpiry time.Time) error {
    oauthProvider := models.NewOAuthProvider(userID, provider, providerUserID, accessToken, refreshToken, tokenExpiry)
    return s.oauthRepo.CreateOrUpdate(ctx, oauthProvider)
}

// UnlinkOAuthProvider removes OAuth provider from user
func (s *OAuthService) UnlinkOAuthProvider(ctx context.Context, userID, provider string) error {
    // TODO: Get provider by user ID and provider name, then delete
    return nil
}

// GetOAuthProviders retrieves all OAuth providers for a user
func (s *OAuthService) GetOAuthProviders(ctx context.Context, userID string) ([]*models.OAuthProvider, error) {
    return s.oauthRepo.GetByUserID(ctx, userID)
}
```

#### 2.2 MFA Service Methods (`internal/service/mfa_service.go`)

```go
package service

import (
    "context"
    "slate/services/user-auth-service/internal/models"
    "slate/services/user-auth-service/internal/repository"
    "slate/services/user-auth-service/pkg/totp"
)

type MFAService struct {
    mfaRepo  *repository.MFARepository
    userRepo *repository.UserRepository
    totpMgr  *totp.TOTPManager
}

func NewMFAService(mfaRepo *repository.MFARepository, userRepo *repository.UserRepository, issuer string) *MFAService {
    return &MFAService{
        mfaRepo:  mfaRepo,
        userRepo: userRepo,
        totpMgr:  totp.NewTOTPManager(issuer),
    }
}

// SetupMFA initiates MFA setup for a user
func (s *MFAService) SetupMFA(ctx context.Context, userID, mfaType string) (*models.MFASetupResponse, error) {
    user, err := s.userRepo.GetByID(ctx, userID)
    if err != nil {
        return nil, err
    }

    // Generate TOTP secret and QR code
    secret, qrURL, err := s.totpMgr.GenerateSecret(user.Email)
    if err != nil {
        return nil, err
    }

    // Generate backup codes
    backupCodes, err := s.totpMgr.GenerateBackupCodes(10)
    if err != nil {
        return nil, err
    }

    // Store MFA config (disabled until verified)
    mfa := models.NewUserMFA(userID, mfaType, secret, backupCodes)
    if err := s.mfaRepo.CreateOrUpdate(ctx, mfa); err != nil {
        return nil, err
    }

    return &models.MFASetupResponse{
        Secret:      secret,
        QRCodeURL:   qrURL,
        BackupCodes: backupCodes,
    }, nil
}

// VerifyMFA enables MFA after user verifies initial code
func (s *MFAService) VerifyMFA(ctx context.Context, userID, code string) error {
    mfa, err := s.mfaRepo.GetByUserIDAndType(ctx, userID, "totp")
    if err != nil {
        return err
    }

    // Validate the code
    if !s.totpMgr.ValidateCode(mfa.SecretKey, code) {
        return fmt.Errorf("invalid MFA code")
    }

    // Enable MFA
    mfa.IsEnabled = true
    mfa.LastUsedAt = time.Now()
    return s.mfaRepo.CreateOrUpdate(ctx, mfa)
}

// ValidateMFACode validates an MFA code for login
func (s *MFAService) ValidateMFACode(ctx context.Context, userID, code, mfaType string) (bool, error) {
    mfa, err := s.mfaRepo.GetByUserIDAndType(ctx, userID, mfaType)
    if err != nil {
        return false, err
    }

    if !mfa.IsEnabled {
        return false, fmt.Errorf("MFA not enabled")
    }

    valid := s.totpMgr.ValidateCodeWithWindow(mfa.SecretKey, code, 1)
    if valid {
        _ = s.mfaRepo.UpdateLastUsed(ctx, mfa.ID)
    }

    return valid, nil
}

// DisableMFA disables MFA for a user
func (s *MFAService) DisableMFA(ctx context.Context, userID, mfaType, password string) error {
    // Verify password before disabling MFA
    // TODO: Add password verification
    return s.mfaRepo.Delete(ctx, userID, mfaType)
}

// GetMFAStatus retrieves MFA status for a user
func (s *MFAService) GetMFAStatus(ctx context.Context, userID string) ([]*models.UserMFA, error) {
    return s.mfaRepo.GetByUserID(ctx, userID)
}
```

#### 2.3 Group Service Methods (`internal/service/group_service.go`)

```go
package service

import (
    "context"
    "slate/services/user-auth-service/internal/models"
    "slate/services/user-auth-service/internal/repository"
)

type GroupService struct {
    groupRepo *repository.GroupRepository
}

func NewGroupService(groupRepo *repository.GroupRepository) *GroupService {
    return &GroupService{groupRepo: groupRepo}
}

// CreateGroup creates a new user group
func (s *GroupService) CreateGroup(ctx context.Context, name, description, organizationID, createdBy string) (*models.UserGroup, error) {
    group := models.NewUserGroup(name, description, organizationID, createdBy)
    if err := s.groupRepo.CreateGroup(ctx, group); err != nil {
        return nil, err
    }
    return group, nil
}

// GetGroup retrieves a group by ID
func (s *GroupService) GetGroup(ctx context.Context, groupID string) (*models.UserGroup, error) {
    return s.groupRepo.GetGroupByID(ctx, groupID)
}

// UpdateGroup updates group information
func (s *GroupService) UpdateGroup(ctx context.Context, groupID string, name, description *string, isActive *bool) (*models.UserGroup, error) {
    group, err := s.groupRepo.GetGroupByID(ctx, groupID)
    if err != nil {
        return nil, err
    }

    if name != nil {
        group.Name = *name
    }
    if description != nil {
        group.Description = *description
    }
    if isActive != nil {
        group.IsActive = *isActive
    }

    group.UpdatedAt = time.Now()

    if err := s.groupRepo.UpdateGroup(ctx, group); err != nil {
        return nil, err
    }

    return group, nil
}

// DeleteGroup soft deletes a group
func (s *GroupService) DeleteGroup(ctx context.Context, groupID string) error {
    return s.groupRepo.DeleteGroup(ctx, groupID)
}

// ListGroups retrieves groups with pagination
func (s *GroupService) ListGroups(ctx context.Context, organizationID string, page, pageSize int) ([]*models.UserGroup, int, error) {
    return s.groupRepo.ListGroups(ctx, organizationID, page, pageSize)
}

// AddGroupMember adds a user to a group
func (s *GroupService) AddGroupMember(ctx context.Context, groupID, userID, role string) error {
    member := models.NewGroupMember(groupID, userID, role)
    return s.groupRepo.AddMember(ctx, member)
}

// RemoveGroupMember removes a user from a group
func (s *GroupService) RemoveGroupMember(ctx context.Context, groupID, userID string) error {
    return s.groupRepo.RemoveMember(ctx, groupID, userID)
}

// GetGroupMembers retrieves all members of a group
func (s *GroupService) GetGroupMembers(ctx context.Context, groupID string) ([]*models.GroupMember, error) {
    return s.groupRepo.GetGroupMembers(ctx, groupID)
}

// GetUserGroups retrieves all groups a user belongs to
func (s *GroupService) GetUserGroups(ctx context.Context, userID string) ([]*models.UserGroup, error) {
    return s.groupRepo.GetUserGroups(ctx, userID)
}
```

#### 2.4 Parent-Child Service Methods (`internal/service/parent_child_service.go`)

```go
package service

import (
    "context"
    "slate/services/user-auth-service/internal/models"
    "slate/services/user-auth-service/internal/repository"
)

type ParentChildService struct {
    pcRepo   *repository.ParentChildRepository
    userRepo *repository.UserRepository
}

func NewParentChildService(pcRepo *repository.ParentChildRepository, userRepo *repository.UserRepository) *ParentChildService {
    return &ParentChildService{
        pcRepo:   pcRepo,
        userRepo: userRepo,
    }
}

// CreateParentChildLink creates a parent-child relationship
func (s *ParentChildService) CreateParentChildLink(ctx context.Context, parentUserID, childUserID, relationshipType string, permissions map[string]interface{}) error {
    // Validate both users exist
    _, err := s.userRepo.GetByID(ctx, parentUserID)
    if err != nil {
        return fmt.Errorf("parent user not found: %w", err)
    }

    _, err = s.userRepo.GetByID(ctx, childUserID)
    if err != nil {
        return fmt.Errorf("child user not found: %w", err)
    }

    relationship := models.NewParentChildAccount(parentUserID, childUserID, relationshipType, permissions)
    return s.pcRepo.CreateRelationship(ctx, relationship)
}

// RemoveParentChildLink removes a parent-child relationship
func (s *ParentChildService) RemoveParentChildLink(ctx context.Context, parentUserID, childUserID string) error {
    return s.pcRepo.DeleteRelationship(ctx, parentUserID, childUserID)
}

// GetChildAccounts retrieves all child accounts for a parent
func (s *ParentChildService) GetChildAccounts(ctx context.Context, parentUserID string) ([]*models.ParentChildAccount, error) {
    return s.pcRepo.GetChildAccounts(ctx, parentUserID)
}

// GetParentAccounts retrieves all parent accounts for a child
func (s *ParentChildService) GetParentAccounts(ctx context.Context, childUserID string) ([]*models.ParentChildAccount, error) {
    return s.pcRepo.GetParentAccounts(ctx, childUserID)
}

// UpdateParentChildPermissions updates permissions for a parent-child relationship
func (s *ParentChildService) UpdateParentChildPermissions(ctx context.Context, parentUserID, childUserID string, permissions map[string]interface{}) error {
    relationship, err := s.pcRepo.GetRelationship(ctx, parentUserID, childUserID)
    if err != nil {
        return err
    }

    relationship.Permissions = permissions
    relationship.UpdatedAt = time.Now()

    return s.pcRepo.UpdateRelationship(ctx, relationship)
}
```

### Step 3: Update gRPC Handler

Update `internal/grpc/user_handler.go` to wire up the new service methods. You'll need to implement handler methods for each new RPC:

```go
// Example: Add to user_handler.go

// SetupMFA handles MFA setup requests
func (h *UserHandler) SetupMFA(ctx context.Context, req *pb.SetupMFARequest) (*pb.SetupMFAResponse, error) {
    response, err := h.mfaService.SetupMFA(ctx, req.UserId, req.MfaType)
    if err != nil {
        return nil, err
    }

    return &pb.SetupMFAResponse{
        Secret:      response.Secret,
        QrCodeUrl:   response.QRCodeURL,
        BackupCodes: response.BackupCodes,
    }, nil
}

// Add similar methods for all other RPCs...
```

### Step 4: Wire Up Dependencies

Update `cmd/server/main.go` to initialize the new services:

```go
// Initialize repositories
oauthRepo := repository.NewOAuthRepository(db)
mfaRepo := repository.NewMFARepository(db)
groupRepo := repository.NewGroupRepository(db)
parentChildRepo := repository.NewParentChildRepository(db)

// Initialize services
oauthService := service.NewOAuthService(oauthRepo, userRepo, tokenService)
mfaService := service.NewMFAService(mfaRepo, userRepo, "YourAppName")
groupService := service.NewGroupService(groupRepo)
parentChildService := service.NewParentChildService(parentChildRepo, userRepo)

// Pass to handler
handler := grpc.NewUserHandler(
    userService,
    oauthService,
    mfaService,
    groupService,
    parentChildService,
)
```

### Step 5: Run Database Migration

Apply the database migration:

```bash
cd /home/user/slate/services/user-auth-service
psql -U postgres -d userauth -f migrations/002_enhanced_features.sql
```

Or use your migration tool if configured.

### Step 6: Test the Implementation

1. **Unit Tests**: Create tests for each service method
2. **Integration Tests**: Test gRPC endpoints
3. **End-to-End Tests**: Test through API gateway

Example test:
```bash
# Test OAuth callback
curl -X POST http://localhost:8080/api/oauth/callback \
  -H "Content-Type: application/json" \
  -d '{"provider":"google","code":"auth_code","state":"random_state"}'

# Test MFA setup
curl -X POST http://localhost:8080/api/mfa/setup \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user-123","mfa_type":"totp"}'
```

## API Routes Summary

All routes are automatically exposed through the API gateway:

### Auto-Discovered (RESTful patterns):
- `GET /api/users/:id` - GetUser
- `GET /api/users` - ListUsers
- `POST /api/users` - CreateUser
- `PUT /api/users/:id` - UpdateUser
- `DELETE /api/users/:id` - DeleteUser
- `GET /api/profiles/:id` - GetProfile
- `PUT /api/profiles/:id` - UpdateProfile

### Manual Overrides:
- **Authentication**: `/api/auth/*`
- **OAuth**: `/api/oauth/*`
- **MFA**: `/api/mfa/*`
- **Groups** (auto-discovered): `/api/groups` and `/api/groups/:id`
- **Parent-Child** (requires overrides if non-RESTful)

## Security Notes

1. **OAuth Callback**: Marked as public - validate state parameter
2. **MFA Setup**: Requires authentication
3. **MFA Verification**: Part of login flow - special handling needed
4. **Group Operations**: Require appropriate permissions
5. **Parent-Child Links**: Verify parent has permission to link

## Next Steps

1. Generate proto code: `make proto`
2. Implement service methods (use templates above)
3. Wire up gRPC handlers
4. Run database migration
5. Test each endpoint
6. Deploy and monitor

## Troubleshooting

**Proto generation fails:**
- Ensure `protoc` is installed
- Check Go proto plugins are in PATH
- Verify proto syntax is correct

**Service not starting:**
- Check database connection
- Verify all dependencies are initialized
- Check logs for detailed errors

**Routes not working:**
- Verify gRPC reflection is enabled
- Check gateway logs for route discovery
- Ensure service name matches in config

## References

- Proto definitions: `/home/user/slate/proto/user.proto`
- Feature docs: `/home/user/slate/services/user-auth-service/FEATURES.md`
- Gateway config: `/home/user/slate/config/gateway-config.docker.yaml`
