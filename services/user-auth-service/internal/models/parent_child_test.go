package models

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewParentChildAccount(t *testing.T) {
	parentUserID := "parent-123"
	childUserID := "child-123"
	relationshipType := "parent"
	permissions := map[string]interface{}{
		"view_profile":    true,
		"edit_settings":   false,
		"manage_payments": true,
	}

	relationship := NewParentChildAccount(parentUserID, childUserID, relationshipType, permissions)

	assert.NotNil(t, relationship)
	assert.Equal(t, parentUserID, relationship.ParentUserID)
	assert.Equal(t, childUserID, relationship.ChildUserID)
	assert.Equal(t, relationshipType, relationship.RelationshipType)
	assert.Equal(t, permissions, relationship.Permissions)
	assert.False(t, relationship.CreatedAt.IsZero())
	assert.False(t, relationship.UpdatedAt.IsZero())
}

func TestParentChildAccount_RelationshipTypes(t *testing.T) {
	tests := []struct {
		name             string
		relationshipType string
	}{
		{"Parent type", "parent"},
		{"Guardian type", "guardian"},
		{"Administrator type", "administrator"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			relationship := NewParentChildAccount("parent-123", "child-123", tt.relationshipType, nil)
			assert.Equal(t, tt.relationshipType, relationship.RelationshipType)
		})
	}
}

func TestParentChildAccount_Permissions(t *testing.T) {
	tests := []struct {
		name        string
		permissions map[string]interface{}
	}{
		{
			"Full permissions",
			map[string]interface{}{
				"view_profile":    true,
				"edit_settings":   true,
				"manage_payments": true,
				"delete_account":  true,
			},
		},
		{
			"Limited permissions",
			map[string]interface{}{
				"view_profile":  true,
				"edit_settings": false,
			},
		},
		{
			"No permissions",
			map[string]interface{}{},
		},
		{
			"Nil permissions",
			nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			relationship := NewParentChildAccount("parent-123", "child-123", "parent", tt.permissions)
			assert.Equal(t, tt.permissions, relationship.Permissions)
		})
	}
}

func TestParentChildAccount_PermissionValues(t *testing.T) {
	permissions := map[string]interface{}{
		"view_profile":  true,
		"edit_settings": false,
		"max_spend":     100.50,
		"allowed_apps":  []string{"app1", "app2"},
	}

	relationship := NewParentChildAccount("parent-123", "child-123", "parent", permissions)

	assert.True(t, relationship.Permissions["view_profile"].(bool))
	assert.False(t, relationship.Permissions["edit_settings"].(bool))
	assert.Equal(t, 100.50, relationship.Permissions["max_spend"].(float64))
	assert.Equal(t, []string{"app1", "app2"}, relationship.Permissions["allowed_apps"].([]string))
}

func TestParentChildWithUsers(t *testing.T) {
	parentUser := &User{
		ID:        "parent-123",
		Email:     "parent@example.com",
		FirstName: "Parent",
		LastName:  "User",
	}

	childUser := &User{
		ID:        "child-123",
		Email:     "child@example.com",
		FirstName: "Child",
		LastName:  "User",
	}

	permissions := map[string]interface{}{
		"view_profile": true,
	}

	relationshipWithUsers := &ParentChildWithUsers{
		ParentUser:  parentUser,
		ChildUser:   childUser,
		Relationship: "parent",
		Permissions: permissions,
	}

	assert.Equal(t, "parent-123", relationshipWithUsers.ParentUser.ID)
	assert.Equal(t, "child-123", relationshipWithUsers.ChildUser.ID)
	assert.Equal(t, "parent", relationshipWithUsers.Relationship)
	assert.Equal(t, permissions, relationshipWithUsers.Permissions)
}
