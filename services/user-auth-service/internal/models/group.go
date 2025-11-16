package models

import (
	"time"

	"github.com/google/uuid"
)

// UserGroup represents a group of users
type UserGroup struct {
	ID             string    `json:"id"`
	Name           string    `json:"name"`
	Description    string    `json:"description"`
	OrganizationID string    `json:"organization_id,omitempty"`
	IsActive       bool      `json:"is_active"`
	CreatedAt      time.Time `json:"created_at"`
	UpdatedAt      time.Time `json:"updated_at"`
	CreatedBy      string    `json:"created_by"`
}

// NewUserGroup creates a new user group
func NewUserGroup(name, description, organizationID, createdBy string) *UserGroup {
	now := time.Now()
	return &UserGroup{
		ID:             uuid.New().String(),
		Name:           name,
		Description:    description,
		OrganizationID: organizationID,
		IsActive:       true,
		CreatedAt:      now,
		UpdatedAt:      now,
		CreatedBy:      createdBy,
	}
}

// GroupMember represents a user's membership in a group
type GroupMember struct {
	GroupID  string    `json:"group_id"`
	UserID   string    `json:"user_id"`
	Role     string    `json:"role"` // owner, admin, member
	JoinedAt time.Time `json:"joined_at"`
}

// NewGroupMember creates a new group member
func NewGroupMember(groupID, userID, role string) *GroupMember {
	return &GroupMember{
		GroupID:  groupID,
		UserID:   userID,
		Role:     role,
		JoinedAt: time.Now(),
	}
}

// GroupWithMembers represents a group with its members
type GroupWithMembers struct {
	Group   *UserGroup     `json:"group"`
	Members []*GroupMember `json:"members"`
}
