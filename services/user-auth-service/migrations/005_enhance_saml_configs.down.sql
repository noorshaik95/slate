-- Rollback migration: Remove enhancements from saml_configs table

-- Drop indexes
DROP INDEX IF EXISTS idx_saml_configs_config_key;
DROP INDEX IF EXISTS idx_saml_configs_provider_type;

-- Drop constraints
ALTER TABLE saml_configs DROP CONSTRAINT IF EXISTS unique_saml_config_key;
ALTER TABLE saml_configs DROP CONSTRAINT IF EXISTS check_saml_provider_type;

-- Drop columns
ALTER TABLE saml_configs DROP COLUMN IF EXISTS config_key;
ALTER TABLE saml_configs DROP COLUMN IF EXISTS attribute_mapping;
ALTER TABLE saml_configs DROP COLUMN IF EXISTS group_attribute;
ALTER TABLE saml_configs DROP COLUMN IF EXISTS group_sync;
ALTER TABLE saml_configs DROP COLUMN IF EXISTS jit_provisioning;
ALTER TABLE saml_configs DROP COLUMN IF EXISTS metadata_url;
ALTER TABLE saml_configs DROP COLUMN IF EXISTS provider_type;

-- Re-add organization_id column if needed
ALTER TABLE saml_configs ADD COLUMN IF NOT EXISTS organization_id VARCHAR(36);
