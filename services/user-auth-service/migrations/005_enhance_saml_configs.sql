-- Enhance saml_configs table for multi-provider SAML support
-- Remove organization_id as organizations are managed via environment config
-- Add provider-specific fields and configuration options

-- Remove organization_id column if it exists (organizations managed via config, not database)
ALTER TABLE saml_configs DROP COLUMN IF EXISTS organization_id;

-- Add provider_type column to distinguish between different SAML providers
ALTER TABLE saml_configs ADD COLUMN IF NOT EXISTS provider_type VARCHAR(50) DEFAULT 'custom';

-- Add metadata_url for automatic metadata refresh
ALTER TABLE saml_configs ADD COLUMN IF NOT EXISTS metadata_url TEXT;

-- Add JIT (Just-In-Time) provisioning flag
ALTER TABLE saml_configs ADD COLUMN IF NOT EXISTS jit_provisioning BOOLEAN DEFAULT false;

-- Add group sync flag for syncing groups from SAML attributes
ALTER TABLE saml_configs ADD COLUMN IF NOT EXISTS group_sync BOOLEAN DEFAULT false;

-- Add group_attribute to specify which SAML attribute contains groups
ALTER TABLE saml_configs ADD COLUMN IF NOT EXISTS group_attribute VARCHAR(255);

-- Add attribute_mapping for custom attribute mapping (stored as JSONB)
ALTER TABLE saml_configs ADD COLUMN IF NOT EXISTS attribute_mapping JSONB;

-- Add config_key to identify which environment config this belongs to
ALTER TABLE saml_configs ADD COLUMN IF NOT EXISTS config_key VARCHAR(100);

-- Add UNIQUE constraint on config_key
ALTER TABLE saml_configs ADD CONSTRAINT unique_saml_config_key UNIQUE (config_key);

-- Create index on provider_type for filtering
CREATE INDEX IF NOT EXISTS idx_saml_configs_provider_type ON saml_configs(provider_type);

-- Create index on config_key for lookups
CREATE INDEX IF NOT EXISTS idx_saml_configs_config_key ON saml_configs(config_key);

-- Add CHECK constraint for provider_type
ALTER TABLE saml_configs ADD CONSTRAINT check_saml_provider_type 
    CHECK (provider_type IN ('okta', 'auth0', 'adfs', 'shibboleth', 'custom'));
