package models

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewUserGroup(t *testing.T) {
	name := "Test Group"
	description := "Test Description"
	organizationID := "org-123"
	createdBy := "user-123"

	group := NewUserGroup(name, description, organizationID, createdBy)

	assert.NotNil(t, group)
	assert.NotEmpty(t, group.ID)
	assert.Equal(t, name, group.Name)
	assert.Equal(t, description, group.Description)
	assert.Equal(t, organizationID, group.OrganizationID)
	assert.Equal(t, createdBy, group.CreatedBy)
	assert.True(t, group.IsActive) // Should be active by default
	assert.False(t, group.CreatedAt.IsZero())
	assert.False(t, group.UpdatedAt.IsZero())
}

func TestUserGroup_Fields(t *testing.T) {
	group := &UserGroup{
		ID:             "group-123",
		Name:           "Group Name",
		Description:    "Group Description",
		OrganizationID: "org-123",
		IsActive:       true,
	}

	assert.Equal(t, "group-123", group.ID)
	assert.Equal(t, "Group Name", group.Name)
	assert.Equal(t, "Group Description", group.Description)
	assert.Equal(t, "org-123", group.OrganizationID)
	assert.True(t, group.IsActive)
}

func TestNewGroupMember(t *testing.T) {
	groupID := "group-123"
	userID := "user-123"
	role := "member"

	member := NewGroupMember(groupID, userID, role)

	assert.NotNil(t, member)
	assert.Equal(t, groupID, member.GroupID)
	assert.Equal(t, userID, member.UserID)
	assert.Equal(t, role, member.Role)
	assert.False(t, member.JoinedAt.IsZero())
}

func TestGroupMember_Roles(t *testing.T) {
	tests := []struct {
		name string
		role string
	}{
		{"Owner role", "owner"},
		{"Admin role", "admin"},
		{"Member role", "member"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			member := NewGroupMember("group-123", "user-123", tt.role)
			assert.Equal(t, tt.role, member.Role)
		})
	}
}

func TestGroupWithMembers(t *testing.T) {
	group := &UserGroup{
		ID:   "group-123",
		Name: "Test Group",
	}

	members := []*GroupMember{
		{GroupID: "group-123", UserID: "user-1", Role: "owner"},
		{GroupID: "group-123", UserID: "user-2", Role: "member"},
	}

	groupWithMembers := &GroupWithMembers{
		Group:   group,
		Members: members,
	}

	assert.Equal(t, "group-123", groupWithMembers.Group.ID)
	assert.Equal(t, "Test Group", groupWithMembers.Group.Name)
	assert.Len(t, groupWithMembers.Members, 2)
	assert.Equal(t, "owner", groupWithMembers.Members[0].Role)
	assert.Equal(t, "member", groupWithMembers.Members[1].Role)
}
