-- Create resources table
CREATE TABLE resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lesson_id UUID NOT NULL REFERENCES lessons(id) ON DELETE RESTRICT,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    content_type VARCHAR(50) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_key VARCHAR(500) NOT NULL,
    manifest_url VARCHAR(500),
    duration_seconds INTEGER,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    downloadable BOOLEAN NOT NULL DEFAULT FALSE,
    copyright_setting VARCHAR(50) NOT NULL DEFAULT 'unrestricted',
    display_order INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(lesson_id, display_order)
);

-- Create index for lesson lookups
CREATE INDEX idx_resources_lesson ON resources(lesson_id);

-- Create index for published status filtering
CREATE INDEX idx_resources_published ON resources(published);

-- Create index for ordering
CREATE INDEX idx_resources_display_order ON resources(lesson_id, display_order);

-- Create index for content type filtering
CREATE INDEX idx_resources_content_type ON resources(content_type);
