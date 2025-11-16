-- Add auth_method column to users table for multi-authentication support
-- This column tracks which authentication method was used: normal, oauth, or saml

-- Add auth_method column with default 'normal' for existing users
ALTER TABLE users ADD COLUMN IF NOT EXISTS auth_method VARCHAR(20) DEFAULT 'normal';

-- Add CHECK constraint to ensure only valid auth methods are stored
ALTER TABLE users ADD CONSTRAINT check_auth_method 
    CHECK (auth_method IN ('normal', 'oauth', 'saml'));

-- Create index on auth_method for efficient filtering queries
CREATE INDEX IF NOT EXISTS idx_users_auth_method ON users(auth_method);

-- Update existing users to have 'normal' auth_method if NULL
UPDATE users SET auth_method = 'normal' WHERE auth_method IS NULL;
