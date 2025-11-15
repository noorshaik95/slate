-- Add lesson_id to upload_sessions table
ALTER TABLE upload_sessions
ADD COLUMN lesson_id UUID;

-- Create index for lesson lookups
CREATE INDEX idx_upload_sessions_lesson ON upload_sessions(lesson_id);
