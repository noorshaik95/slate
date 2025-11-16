-- Add provider_type column to oauth_providers table
-- This distinguishes between google, microsoft, and custom OAuth providers

-- Add provider_type column with default 'custom'
ALTER TABLE oauth_providers ADD COLUMN IF NOT EXISTS provider_type VARCHAR(50) DEFAULT 'custom';

-- Add CHECK constraint to validate provider_type values
ALTER TABLE oauth_providers ADD CONSTRAINT check_oauth_provider_type 
    CHECK (provider_type IN ('google', 'microsoft', 'custom'));

-- Create index on provider_type for efficient filtering
CREATE INDEX IF NOT EXISTS idx_oauth_providers_provider_type ON oauth_providers(provider_type);

-- Update existing records to set provider_type based on provider column
UPDATE oauth_providers 
SET provider_type = CASE 
    WHEN provider = 'google' THEN 'google'
    WHEN provider = 'microsoft' THEN 'microsoft'
    ELSE 'custom'
END
WHERE provider_type = 'custom';
