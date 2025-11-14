-- Create progress_tracking table
CREATE TABLE progress_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL,
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    completed_at TIMESTAMP,
    last_position_seconds INTEGER,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(student_id, resource_id)
);

-- Create index for student lookups
CREATE INDEX idx_progress_student ON progress_tracking(student_id);

-- Create index for resource lookups
CREATE INDEX idx_progress_resource ON progress_tracking(resource_id);

-- Create index for completion filtering
CREATE INDEX idx_progress_completed ON progress_tracking(completed);

-- Create composite index for student progress queries
CREATE INDEX idx_progress_student_completed ON progress_tracking(student_id, completed);
