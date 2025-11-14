package models

import (
	"time"
)

// ParentChildAccount represents a parent-child account relationship
type ParentChildAccount struct {
	ParentUserID     string                 `json:"parent_user_id"`
	ChildUserID      string                 `json:"child_user_id"`
	RelationshipType string                 `json:"relationship_type"` // parent, guardian, administrator
	Permissions      map[string]interface{} `json:"permissions,omitempty"`
	CreatedAt        time.Time              `json:"created_at"`
	UpdatedAt        time.Time              `json:"updated_at"`
}

// NewParentChildAccount creates a new parent-child account relationship
func NewParentChildAccount(parentUserID, childUserID, relationshipType string, permissions map[string]interface{}) *ParentChildAccount {
	now := time.Now()
	return &ParentChildAccount{
		ParentUserID:     parentUserID,
		ChildUserID:      childUserID,
		RelationshipType: relationshipType,
		Permissions:      permissions,
		CreatedAt:        now,
		UpdatedAt:        now,
	}
}

// ParentChildWithUsers represents a parent-child relationship with user details
type ParentChildWithUsers struct {
	ParentUser  *User                  `json:"parent_user"`
	ChildUser   *User                  `json:"child_user"`
	Relationship string                `json:"relationship"`
	Permissions  map[string]interface{} `json:"permissions,omitempty"`
	CreatedAt    time.Time              `json:"created_at"`
}
