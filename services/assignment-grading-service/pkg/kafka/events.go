package kafka

import "time"

// Event types
const (
	EventTypeAssignmentCreated   = "assignment.created"
	EventTypeAssignmentUpdated   = "assignment.updated"
	EventTypeAssignmentDeleted   = "assignment.deleted"
	EventTypeSubmissionCreated   = "assignment.submitted"
	EventTypeGradePublished      = "grade.published"
)

// Event represents a domain event
type Event struct {
	Type        string                 `json:"type"`
	AggregateID string                 `json:"aggregate_id"`
	Timestamp   time.Time              `json:"timestamp"`
	Data        map[string]interface{} `json:"data"`
}

// NewAssignmentCreatedEvent creates an assignment.created event
func NewAssignmentCreatedEvent(assignmentID, courseID, title string) Event {
	return Event{
		Type:        EventTypeAssignmentCreated,
		AggregateID: assignmentID,
		Timestamp:   time.Now(),
		Data: map[string]interface{}{
			"assignment_id": assignmentID,
			"course_id":     courseID,
			"title":         title,
		},
	}
}

// NewAssignmentUpdatedEvent creates an assignment.updated event
func NewAssignmentUpdatedEvent(assignmentID, courseID string) Event {
	return Event{
		Type:        EventTypeAssignmentUpdated,
		AggregateID: assignmentID,
		Timestamp:   time.Now(),
		Data: map[string]interface{}{
			"assignment_id": assignmentID,
			"course_id":     courseID,
		},
	}
}

// NewAssignmentDeletedEvent creates an assignment.deleted event
func NewAssignmentDeletedEvent(assignmentID, courseID string) Event {
	return Event{
		Type:        EventTypeAssignmentDeleted,
		AggregateID: assignmentID,
		Timestamp:   time.Now(),
		Data: map[string]interface{}{
			"assignment_id": assignmentID,
			"course_id":     courseID,
		},
	}
}

// NewSubmissionCreatedEvent creates an assignment.submitted event
func NewSubmissionCreatedEvent(submissionID, assignmentID, studentID string, isLate bool) Event {
	return Event{
		Type:        EventTypeSubmissionCreated,
		AggregateID: submissionID,
		Timestamp:   time.Now(),
		Data: map[string]interface{}{
			"submission_id": submissionID,
			"assignment_id": assignmentID,
			"student_id":    studentID,
			"is_late":       isLate,
		},
	}
}

// NewGradePublishedEvent creates a grade.published event
func NewGradePublishedEvent(gradeID, assignmentID, studentID string, score, adjustedScore float64) Event {
	return Event{
		Type:        EventTypeGradePublished,
		AggregateID: gradeID,
		Timestamp:   time.Now(),
		Data: map[string]interface{}{
			"grade_id":       gradeID,
			"assignment_id":  assignmentID,
			"student_id":     studentID,
			"score":          score,
			"adjusted_score": adjustedScore,
		},
	}
}
