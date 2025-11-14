# User Authentication Service - Enhanced Features

This document outlines all the enhanced features added to the User Authentication Service.

## Table of Contents

1. [Authentication Methods](#authentication-methods)
2. [Role-Based Access Control (RBAC)](#role-based-access-control-rbac)
3. [User Profiles](#user-profiles)
4. [Two-Factor Authentication (2FA/MFA)](#two-factor-authentication-2famfa)
5. [SAML Integration](#saml-integration)
6. [User Groups](#user-groups)
7. [Parent-Child Accounts](#parent-child-accounts)

## Authentication Methods

### 1. Email/Password Authentication
- Standard email and password authentication
- Passwords are hashed using bcrypt
- Password complexity requirements enforced
- Account activation/deactivation support

### 2. OAuth SSO
- Support for multiple OAuth providers (Google, GitHub, Microsoft, etc.)
- OAuth provider linking to existing accounts
- Token refresh and management
- Provider-specific user information extraction

**Database Schema:**
```sql
CREATE TABLE oauth_providers (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    token_expiry TIMESTAMP,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

**Models:**
- `OAuthProvider`: OAuth provider configuration
- `OAuthUserInfo`: User information from OAuth provider

**Repository Methods:**
- `CreateOrUpdate`: Link OAuth provider to user
- `GetByProviderAndUserID`: Retrieve OAuth provider by provider name and user ID
- `GetByUserID`: Get all OAuth providers for a user
- `Delete`: Remove OAuth provider

### 3. JWT Tokens
- Access tokens (short-lived, 15 minutes default)
- Refresh tokens (long-lived, 7 days default)
- Token blacklisting on logout
- Token validation with role and permission claims

## Role-Based Access Control (RBAC)

### Predefined Roles

1. **Student**
   - Permissions: `courses.read`, `courses.enroll`, `assignments.read`, `assignments.submit`, `profile.read`, `profile.update`
   - For learners accessing courses and submitting assignments

2. **Instructor**
   - Permissions: `courses.create`, `courses.read`, `courses.update`, `courses.delete`, `assignments.create`, `assignments.read`, `assignments.update`, `assignments.grade`, `students.read`, `profile.read`, `profile.update`
   - For teachers managing courses and grading

3. **Admin**
   - Permissions: `users.create`, `users.read`, `users.update`, `users.delete`, `roles.assign`, `roles.remove`, `system.manage`
   - For system administrators

4. **Super Admin**
   - Permissions: `*` (all permissions)
   - Highest level of access

5. **Manager**
   - Permissions: `users.read`, `users.update`, `profile.read`, `profile.update`
   - For user management with limited privileges

6. **User** (Default)
   - Permissions: `profile.read`, `profile.update`
   - Basic user permissions

### Permission Checking
- Role-based permission checks
- Hierarchical permission inheritance
- Fine-grained access control

## User Profiles

Enhanced user profiles with the following fields:

- **Basic Information:**
  - First Name
  - Last Name
  - Email
  - Phone

- **Profile Details:**
  - Avatar URL
  - Bio/Description
  - Timezone (default: UTC)

- **Organizational:**
  - Organization ID (for multi-tenant support)

- **Metadata:**
  - Created At
  - Updated At
  - Active Status

**Proto Definition:**
```protobuf
message Profile {
  string user_id = 1;
  string first_name = 2;
  string last_name = 3;
  string email = 4;
  string phone = 5;
  string avatar_url = 6;
  string bio = 7;
  string timezone = 8;
  repeated string roles = 9;
  google.protobuf.Timestamp created_at = 10;
  google.protobuf.Timestamp updated_at = 11;
}
```

## Two-Factor Authentication (2FA/MFA)

### TOTP-Based MFA

- **Setup Process:**
  1. Generate TOTP secret
  2. Display QR code for authenticator app
  3. Generate backup codes
  4. Verify initial code to enable MFA

- **MFA Types Supported:**
  - TOTP (Time-based One-Time Password)
  - SMS (placeholder for future implementation)
  - Email (placeholder for future implementation)

**Database Schema:**
```sql
CREATE TABLE user_mfa (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    mfa_type VARCHAR(20) NOT NULL,
    is_enabled BOOLEAN DEFAULT false,
    secret_key TEXT,
    backup_codes TEXT[],
    last_used_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

**TOTP Manager Features:**
- Secret generation
- QR code URL generation
- Code validation with time window
- Backup code generation

**Usage:**
```go
totpMgr := totp.NewTOTPManager("MyApp")
secret, qrURL, err := totpMgr.GenerateSecret("user@example.com")
valid := totpMgr.ValidateCode(secret, "123456")
backupCodes, err := totpMgr.GenerateBackupCodes(10)
```

## SAML Integration

### Enterprise SSO Support

- **SAML 2.0 Support:**
  - Single Sign-On (SSO)
  - Single Logout (SLO)
  - Organization-specific configurations
  - Session management

**Database Schema:**
```sql
CREATE TABLE saml_configs (
    id VARCHAR(36) PRIMARY KEY,
    organization_id VARCHAR(36),
    entity_id VARCHAR(255) NOT NULL UNIQUE,
    sso_url VARCHAR(500) NOT NULL,
    slo_url VARCHAR(500),
    certificate TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE saml_sessions (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    saml_config_id VARCHAR(36) NOT NULL,
    session_index VARCHAR(255),
    name_id VARCHAR(255),
    attributes JSONB,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL
);
```

**Features:**
- Organization-specific SAML configurations
- X.509 certificate management
- Session tracking with expiration
- SAML attribute mapping

## User Groups

### Group Management

- **Group Features:**
  - Create and manage user groups
  - Assign users to groups
  - Role-based group membership (owner, admin, member)
  - Organization-specific groups

**Database Schema:**
```sql
CREATE TABLE user_groups (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    organization_id VARCHAR(36),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    created_by VARCHAR(36)
);

CREATE TABLE group_members (
    group_id VARCHAR(36) NOT NULL,
    user_id VARCHAR(36) NOT NULL,
    role VARCHAR(50),
    joined_at TIMESTAMP NOT NULL,
    PRIMARY KEY (group_id, user_id)
);
```

**Repository Methods:**
- `CreateGroup`: Create a new group
- `GetGroupByID`: Retrieve group by ID
- `ListGroups`: Paginated group listing
- `UpdateGroup`: Update group information
- `DeleteGroup`: Soft delete a group
- `AddMember`: Add user to group
- `RemoveMember`: Remove user from group
- `GetGroupMembers`: Get all members of a group
- `GetUserGroups`: Get all groups a user belongs to

## Parent-Child Accounts

### Account Relationship Management

- **Features:**
  - Link parent and child accounts
  - Relationship types: parent, guardian, administrator
  - Granular permission control
  - Multi-parent support

**Database Schema:**
```sql
CREATE TABLE parent_child_accounts (
    parent_user_id VARCHAR(36) NOT NULL,
    child_user_id VARCHAR(36) NOT NULL,
    relationship_type VARCHAR(50) DEFAULT 'parent',
    permissions JSONB,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    PRIMARY KEY (parent_user_id, child_user_id)
);
```

**Repository Methods:**
- `CreateRelationship`: Create parent-child relationship
- `GetRelationship`: Get specific relationship
- `GetChildAccounts`: Get all child accounts for a parent
- `GetParentAccounts`: Get all parent accounts for a child
- `UpdateRelationship`: Update relationship permissions
- `DeleteRelationship`: Remove relationship
- `HasPermission`: Check if parent has specific permission for child

**Permission Examples:**
```json
{
  "view_grades": true,
  "manage_enrollment": true,
  "view_attendance": true,
  "receive_notifications": true
}
```

## Migration Instructions

To apply all new features, run the migration:

```bash
# Apply the new migration
cd /home/user/slate/services/user-auth-service
psql -U postgres -d userauth -f migrations/002_enhanced_features.sql
```

## API Usage Examples

### OAuth Login Flow
1. Redirect user to OAuth provider
2. Receive callback with OAuth code
3. Exchange code for tokens
4. Create or link user account
5. Return JWT tokens

### MFA Setup Flow
1. User requests MFA setup
2. Generate TOTP secret and QR code
3. User scans QR code with authenticator app
4. User verifies with code
5. Store encrypted secret and backup codes
6. Enable MFA

### SAML Login Flow
1. User initiates SAML SSO
2. Generate SAML request
3. Redirect to IdP
4. Receive SAML response
5. Validate and parse assertion
6. Create session
7. Return JWT tokens

### Group Management
1. Admin creates group
2. Add members with roles
3. Members can view group information
4. Owners can manage group and members

### Parent Account Management
1. Parent creates relationship with child account
2. Set granular permissions
3. Parent can view/manage based on permissions
4. Child maintains independence with oversight

## Security Considerations

1. **Password Security:**
   - Bcrypt hashing with default cost
   - Complexity requirements enforced
   - Password change invalidates all tokens

2. **Token Security:**
   - Short-lived access tokens
   - Token blacklisting on logout
   - Refresh token rotation

3. **MFA Security:**
   - TOTP secrets encrypted at rest
   - Backup codes hashed
   - Time window validation to prevent replay

4. **SAML Security:**
   - Certificate validation
   - Session expiration
   - Secure attribute mapping

5. **OAuth Security:**
   - Token encryption
   - Provider verification
   - Secure token storage

## Multi-Tenant Support

The service now supports multi-tenant deployments:

- Organization ID field on users
- Organization-specific SAML configs
- Organization-scoped groups
- Isolated data per organization
