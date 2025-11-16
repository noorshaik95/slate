-- Create lessons table
CREATE TABLE lessons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id UUID NOT NULL REFERENCES modules(id) ON DELETE RESTRICT,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    display_order INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(module_id, display_order)
);

-- Create index for module lookups
CREATE INDEX idx_lessons_module ON lessons(module_id);

-- Create index for ordering
CREATE INDEX idx_lessons_display_order ON lessons(module_id, display_order);
