-- Create modules table
CREATE TABLE modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    display_order INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    UNIQUE(course_id, display_order)
);

-- Create index for course lookups
CREATE INDEX idx_modules_course ON modules(course_id);

-- Create index for ordering
CREATE INDEX idx_modules_display_order ON modules(course_id, display_order);
