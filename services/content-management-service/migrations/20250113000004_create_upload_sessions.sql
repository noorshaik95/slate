-- Create upload_sessions table
CREATE TABLE upload_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    total_size BIGINT NOT NULL,
    chunk_size INTEGER NOT NULL,
    total_chunks INTEGER NOT NULL,
    uploaded_chunks INTEGER NOT NULL DEFAULT 0,
    storage_key VARCHAR(500) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'in_progress',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP
);

-- Create index for user lookups
CREATE INDEX idx_upload_sessions_user ON upload_sessions(user_id);

-- Create index for status filtering
CREATE INDEX idx_upload_sessions_status ON upload_sessions(status);

-- Create index for expiration cleanup
CREATE INDEX idx_upload_sessions_expires ON upload_sessions(expires_at);
