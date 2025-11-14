-- Create download_tracking table
CREATE TABLE download_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL,
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    downloaded_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create index for student lookups
CREATE INDEX idx_download_student ON download_tracking(student_id);

-- Create index for resource lookups
CREATE INDEX idx_download_resource ON download_tracking(resource_id);

-- Create index for timestamp queries
CREATE INDEX idx_download_timestamp ON download_tracking(downloaded_at);

-- Create composite index for analytics queries
CREATE INDEX idx_download_resource_timestamp ON download_tracking(resource_id, downloaded_at);
