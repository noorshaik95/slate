-- Rollback migration: Remove provider_type column from oauth_providers table

-- Drop the index
DROP INDEX IF EXISTS idx_oauth_providers_provider_type;

-- Drop the CHECK constraint
ALTER TABLE oauth_providers DROP CONSTRAINT IF EXISTS check_oauth_provider_type;

-- Drop the provider_type column
ALTER TABLE oauth_providers DROP COLUMN IF EXISTS provider_type;
