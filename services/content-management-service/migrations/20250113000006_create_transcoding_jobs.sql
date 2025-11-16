-- Create transcoding_jobs table
CREATE TABLE transcoding_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP
);

-- Create index for status filtering
CREATE INDEX idx_transcoding_status ON transcoding_jobs(status);

-- Create index for resource lookups
CREATE INDEX idx_transcoding_resource ON transcoding_jobs(resource_id);

-- Create index for job processing queries
CREATE INDEX idx_transcoding_pending ON transcoding_jobs(status, created_at) WHERE status = 'pending';
