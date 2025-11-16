-- Rollback migration: Remove auth_method column from users table

-- Drop the index
DROP INDEX IF EXISTS idx_users_auth_method;

-- Drop the CHECK constraint
ALTER TABLE users DROP CONSTRAINT IF EXISTS check_auth_method;

-- Drop the auth_method column
ALTER TABLE users DROP COLUMN IF EXISTS auth_method;
