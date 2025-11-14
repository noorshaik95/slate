-- Add timezone to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS timezone VARCHAR(100) DEFAULT 'UTC';
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(500);
ALTER TABLE users ADD COLUMN IF NOT EXISTS bio TEXT;

-- Create OAuth providers table
CREATE TABLE IF NOT EXISTS oauth_providers (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL, -- google, github, microsoft, etc.
    provider_user_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    token_expiry TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(provider, provider_user_id)
);

-- Create 2FA/MFA table
CREATE TABLE IF NOT EXISTS user_mfa (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mfa_type VARCHAR(20) NOT NULL, -- totp, sms, email
    is_enabled BOOLEAN DEFAULT false,
    secret_key TEXT, -- Encrypted TOTP secret
    backup_codes TEXT[], -- Encrypted backup codes
    last_used_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, mfa_type)
);

-- Create SAML configurations table
CREATE TABLE IF NOT EXISTS saml_configs (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id VARCHAR(36), -- For multi-tenant support
    entity_id VARCHAR(255) NOT NULL UNIQUE,
    sso_url VARCHAR(500) NOT NULL,
    slo_url VARCHAR(500),
    certificate TEXT NOT NULL, -- X.509 certificate
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create SAML sessions table
CREATE TABLE IF NOT EXISTS saml_sessions (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    saml_config_id VARCHAR(36) NOT NULL REFERENCES saml_configs(id) ON DELETE CASCADE,
    session_index VARCHAR(255),
    name_id VARCHAR(255),
    attributes JSONB, -- Store SAML attributes
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL
);

-- Create user groups table
CREATE TABLE IF NOT EXISTS user_groups (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    organization_id VARCHAR(36), -- For multi-tenant support
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by VARCHAR(36) REFERENCES users(id),
    UNIQUE(name, organization_id)
);

-- Create group members table (many-to-many)
CREATE TABLE IF NOT EXISTS group_members (
    group_id VARCHAR(36) NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50), -- owner, admin, member
    joined_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, user_id)
);

-- Create parent-child accounts table
CREATE TABLE IF NOT EXISTS parent_child_accounts (
    parent_user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    child_user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) DEFAULT 'parent', -- parent, guardian, administrator
    permissions JSONB, -- Specific permissions parent has over child account
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (parent_user_id, child_user_id),
    CHECK (parent_user_id != child_user_id)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_oauth_providers_user_id ON oauth_providers(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_providers_provider ON oauth_providers(provider);
CREATE INDEX IF NOT EXISTS idx_user_mfa_user_id ON user_mfa(user_id);
CREATE INDEX IF NOT EXISTS idx_saml_sessions_user_id ON saml_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_saml_sessions_expires_at ON saml_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_groups_organization_id ON user_groups(organization_id);
CREATE INDEX IF NOT EXISTS idx_group_members_user_id ON group_members(user_id);
CREATE INDEX IF NOT EXISTS idx_group_members_group_id ON group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_parent_child_parent ON parent_child_accounts(parent_user_id);
CREATE INDEX IF NOT EXISTS idx_parent_child_child ON parent_child_accounts(child_user_id);

-- Insert new roles (Student, Instructor, Super Admin)
INSERT INTO roles (id, name, description, permissions, created_at, updated_at)
VALUES
    (gen_random_uuid(), 'student', 'Student with course access and assignment submission',
     ARRAY['courses.read', 'courses.enroll', 'assignments.read', 'assignments.submit', 'profile.read', 'profile.update'],
     NOW(), NOW()),
    (gen_random_uuid(), 'instructor', 'Instructor with course management and grading access',
     ARRAY['courses.create', 'courses.read', 'courses.update', 'courses.delete', 'assignments.create', 'assignments.read', 'assignments.update', 'assignments.grade', 'students.read', 'profile.read', 'profile.update'],
     NOW(), NOW()),
    (gen_random_uuid(), 'superadmin', 'Super Administrator with system-level access',
     ARRAY['*'], -- All permissions
     NOW(), NOW())
ON CONFLICT (name) DO NOTHING;

-- Add organization_id to users for multi-tenant support
ALTER TABLE users ADD COLUMN IF NOT EXISTS organization_id VARCHAR(36);
CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id);
