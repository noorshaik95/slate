-- Rollback migration: Drop saml_metadata_cache table

-- Drop indexes
DROP INDEX IF EXISTS idx_saml_metadata_cache_expires_at;
DROP INDEX IF EXISTS idx_saml_metadata_cache_config_key;

-- Drop table
DROP TABLE IF EXISTS saml_metadata_cache;
