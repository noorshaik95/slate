package models

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNewUser(t *testing.T) {
	email := "test@example.com"
	passwordHash := "hashed-password"
	firstName := "John"
	lastName := "Doe"
	phone := "+1234567890"

	user := NewUser(email, passwordHash, firstName, lastName, phone)

	assert.NotEmpty(t, user.ID)
	assert.Equal(t, email, user.Email)
	assert.Equal(t, passwordHash, user.PasswordHash)
	assert.Equal(t, firstName, user.FirstName)
	assert.Equal(t, lastName, user.LastName)
	assert.Equal(t, phone, user.Phone)
	assert.True(t, user.IsActive)
	assert.False(t, user.CreatedAt.IsZero())
	assert.False(t, user.UpdatedAt.IsZero())
	assert.WithinDuration(t, time.Now(), user.CreatedAt, time.Second)
	assert.WithinDuration(t, time.Now(), user.UpdatedAt, time.Second)
}

func TestUser_FullName(t *testing.T) {
	user := &User{
		FirstName: "John",
		LastName:  "Doe",
	}

	fullName := user.FullName()

	assert.Equal(t, "John Doe", fullName)
}

func TestUser_HasRole(t *testing.T) {
	tests := []struct {
		name      string
		roles     []string
		checkRole string
		expected  bool
	}{
		{
			name:      "has role",
			roles:     []string{"user", "admin"},
			checkRole: "admin",
			expected:  true,
		},
		{
			name:      "does not have role",
			roles:     []string{"user"},
			checkRole: "admin",
			expected:  false,
		},
		{
			name:      "empty roles",
			roles:     []string{},
			checkRole: "user",
			expected:  false,
		},
		{
			name:      "nil roles",
			roles:     nil,
			checkRole: "user",
			expected:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			user := &User{Roles: tt.roles}
			result := user.HasRole(tt.checkRole)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestUser_IsAdmin(t *testing.T) {
	tests := []struct {
		name     string
		roles    []string
		expected bool
	}{
		{
			name:     "is admin",
			roles:    []string{"user", "admin"},
			expected: true,
		},
		{
			name:     "is not admin",
			roles:    []string{"user", "manager"},
			expected: false,
		},
		{
			name:     "no roles",
			roles:    []string{},
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			user := &User{Roles: tt.roles}
			result := user.IsAdmin()
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestUser_ToProfile(t *testing.T) {
	now := time.Now()
	user := &User{
		ID:        "user-123",
		Email:     "test@example.com",
		FirstName: "John",
		LastName:  "Doe",
		Phone:     "+1234567890",
		Roles:     []string{"user", "admin"},
		CreatedAt: now,
		UpdatedAt: now,
	}

	profile := user.ToProfile()

	assert.Equal(t, user.ID, profile.UserID)
	assert.Equal(t, user.Email, profile.Email)
	assert.Equal(t, user.FirstName, profile.FirstName)
	assert.Equal(t, user.LastName, profile.LastName)
	assert.Equal(t, user.Phone, profile.Phone)
	assert.Equal(t, user.Roles, profile.Roles)
	assert.Equal(t, user.CreatedAt, profile.CreatedAt)
	assert.Equal(t, user.UpdatedAt, profile.UpdatedAt)
	assert.Empty(t, profile.AvatarURL)
	assert.Empty(t, profile.Bio)
}
