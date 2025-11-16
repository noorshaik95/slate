-- Create saml_metadata_cache table for caching SAML IdP metadata
-- This improves performance by avoiding repeated metadata fetches

CREATE TABLE IF NOT EXISTS saml_metadata_cache (
    id VARCHAR(36) PRIMARY KEY DEFAULT gen_random_uuid(),
    config_key VARCHAR(100) NOT NULL,
    metadata_xml TEXT NOT NULL,
    fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    CONSTRAINT unique_saml_metadata_config_key UNIQUE (config_key)
);

-- Create index on config_key for fast lookups
CREATE INDEX IF NOT EXISTS idx_saml_metadata_cache_config_key ON saml_metadata_cache(config_key);

-- Create index on expires_at for cleanup queries
CREATE INDEX IF NOT EXISTS idx_saml_metadata_cache_expires_at ON saml_metadata_cache(expires_at);

-- Add comment to table
COMMENT ON TABLE saml_metadata_cache IS 'Caches SAML IdP metadata to reduce external requests. config_key matches the SAML provider config key from environment.';
