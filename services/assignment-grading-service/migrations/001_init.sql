-- Assignment Grading Service Database Schema

-- Assignments table
CREATE TABLE IF NOT EXISTS assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    max_points DECIMAL(10, 2) NOT NULL CHECK (max_points > 0),
    due_date TIMESTAMP NOT NULL,

    -- Late policy configuration
    late_penalty_percent INT DEFAULT 0 CHECK (late_penalty_percent >= 0 AND late_penalty_percent <= 100),
    max_late_days INT DEFAULT 0 CHECK (max_late_days >= 0),

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Submissions table
CREATE TABLE IF NOT EXISTS submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    student_id VARCHAR(255) NOT NULL,
    file_path VARCHAR(1000) NOT NULL,
    submitted_at TIMESTAMP NOT NULL DEFAULT NOW(),
    status VARCHAR(50) NOT NULL DEFAULT 'submitted' CHECK (status IN ('submitted', 'graded', 'returned')),
    is_late BOOLEAN NOT NULL DEFAULT FALSE,
    days_late INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Ensure one submission per student per assignment (replace if resubmitted)
    UNIQUE(assignment_id, student_id)
);

-- Grades table
CREATE TABLE IF NOT EXISTS grades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    student_id VARCHAR(255) NOT NULL,
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    score DECIMAL(10, 2) NOT NULL CHECK (score >= 0),
    adjusted_score DECIMAL(10, 2) NOT NULL CHECK (adjusted_score >= 0),
    feedback TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published')),
    graded_at TIMESTAMP,
    published_at TIMESTAMP,
    graded_by VARCHAR(255) NOT NULL, -- Instructor ID
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Ensure one grade per submission
    UNIQUE(submission_id)
);

-- Indexes for performance

-- Assignment indexes
CREATE INDEX idx_assignments_course_id ON assignments(course_id);
CREATE INDEX idx_assignments_due_date ON assignments(due_date);

-- Submission indexes
CREATE INDEX idx_submissions_assignment_id ON submissions(assignment_id);
CREATE INDEX idx_submissions_student_id ON submissions(student_id);
CREATE INDEX idx_submissions_submitted_at ON submissions(submitted_at);
CREATE INDEX idx_submissions_status ON submissions(status);

-- Grade indexes
CREATE INDEX idx_grades_submission_id ON grades(submission_id);
CREATE INDEX idx_grades_student_id ON grades(student_id);
CREATE INDEX idx_grades_assignment_id ON grades(assignment_id);
CREATE INDEX idx_grades_status ON grades(status);
CREATE INDEX idx_grades_graded_by ON grades(graded_by);

-- Composite indexes for common queries
CREATE INDEX idx_grades_student_assignment ON grades(student_id, assignment_id);
CREATE INDEX idx_submissions_assignment_student ON submissions(assignment_id, student_id);
