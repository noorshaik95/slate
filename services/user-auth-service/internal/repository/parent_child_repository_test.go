package repository

import (
	"context"
	"database/sql"
	"encoding/json"
	"testing"
	"time"

	"slate/services/user-auth-service/internal/models"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParentChildRepository_CreateRelationship_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions := map[string]interface{}{
		"view_profile":    true,
		"edit_settings":   false,
		"manage_payments": true,
	}

	relationship := &models.ParentChildAccount{
		ParentUserID:     "parent-123",
		ChildUserID:      "child-123",
		RelationshipType: "parent",
		Permissions:      permissions,
		CreatedAt:        now,
		UpdatedAt:        now,
	}

	permissionsJSON, _ := json.Marshal(permissions)

	mock.ExpectExec("INSERT INTO parent_child_accounts").
		WithArgs(relationship.ParentUserID, relationship.ChildUserID, relationship.RelationshipType,
			permissionsJSON, relationship.CreatedAt, relationship.UpdatedAt).
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.CreateRelationship(context.Background(), relationship)
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_CreateRelationship_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions := map[string]interface{}{
		"view_profile": true,
	}

	relationship := &models.ParentChildAccount{
		ParentUserID:     "parent-123",
		ChildUserID:      "child-123",
		RelationshipType: "parent",
		Permissions:      permissions,
		CreatedAt:        now,
		UpdatedAt:        now,
	}

	permissionsJSON, _ := json.Marshal(permissions)

	mock.ExpectExec("INSERT INTO parent_child_accounts").
		WithArgs(relationship.ParentUserID, relationship.ChildUserID, relationship.RelationshipType,
			permissionsJSON, relationship.CreatedAt, relationship.UpdatedAt).
		WillReturnError(sql.ErrConnDone)

	err = repo.CreateRelationship(context.Background(), relationship)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to create parent-child relationship")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetRelationship_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions := map[string]interface{}{
		"view_profile":    true,
		"edit_settings":   false,
		"manage_payments": true,
	}
	permissionsJSON, _ := json.Marshal(permissions)

	rows := sqlmock.NewRows([]string{"parent_user_id", "child_user_id", "relationship_type",
		"permissions", "created_at", "updated_at"}).
		AddRow("parent-123", "child-123", "parent", permissionsJSON, now, now)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "child-123").
		WillReturnRows(rows)

	relationship, err := repo.GetRelationship(context.Background(), "parent-123", "child-123")
	require.NoError(t, err)
	assert.NotNil(t, relationship)
	assert.Equal(t, "parent-123", relationship.ParentUserID)
	assert.Equal(t, "child-123", relationship.ChildUserID)
	assert.Equal(t, "parent", relationship.RelationshipType)
	assert.Equal(t, permissions, relationship.Permissions)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetRelationship_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "nonexistent").
		WillReturnError(sql.ErrNoRows)

	relationship, err := repo.GetRelationship(context.Background(), "parent-123", "nonexistent")
	assert.Error(t, err)
	assert.Nil(t, relationship)
	assert.Contains(t, err.Error(), "parent-child relationship not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetRelationship_DatabaseError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "child-123").
		WillReturnError(sql.ErrConnDone)

	relationship, err := repo.GetRelationship(context.Background(), "parent-123", "child-123")
	assert.Error(t, err)
	assert.Nil(t, relationship)
	assert.Contains(t, err.Error(), "failed to get parent-child relationship")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetChildAccounts_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions1 := map[string]interface{}{"view_profile": true}
	permissions2 := map[string]interface{}{"edit_settings": true}
	permissionsJSON1, _ := json.Marshal(permissions1)
	permissionsJSON2, _ := json.Marshal(permissions2)

	rows := sqlmock.NewRows([]string{"parent_user_id", "child_user_id", "relationship_type",
		"permissions", "created_at", "updated_at"}).
		AddRow("parent-123", "child-1", "parent", permissionsJSON1, now, now).
		AddRow("parent-123", "child-2", "guardian", permissionsJSON2, now, now)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1").
		WithArgs("parent-123").
		WillReturnRows(rows)

	relationships, err := repo.GetChildAccounts(context.Background(), "parent-123")
	require.NoError(t, err)
	assert.Len(t, relationships, 2)
	assert.Equal(t, "child-1", relationships[0].ChildUserID)
	assert.Equal(t, "parent", relationships[0].RelationshipType)
	assert.Equal(t, "child-2", relationships[1].ChildUserID)
	assert.Equal(t, "guardian", relationships[1].RelationshipType)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetChildAccounts_Empty(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	rows := sqlmock.NewRows([]string{"parent_user_id", "child_user_id", "relationship_type",
		"permissions", "created_at", "updated_at"})

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1").
		WithArgs("parent-123").
		WillReturnRows(rows)

	relationships, err := repo.GetChildAccounts(context.Background(), "parent-123")
	require.NoError(t, err)
	assert.Empty(t, relationships)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetChildAccounts_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1").
		WithArgs("parent-123").
		WillReturnError(sql.ErrConnDone)

	relationships, err := repo.GetChildAccounts(context.Background(), "parent-123")
	assert.Error(t, err)
	assert.Nil(t, relationships)
	assert.Contains(t, err.Error(), "failed to get child accounts")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetParentAccounts_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions1 := map[string]interface{}{"view_profile": true}
	permissions2 := map[string]interface{}{"edit_settings": true}
	permissionsJSON1, _ := json.Marshal(permissions1)
	permissionsJSON2, _ := json.Marshal(permissions2)

	rows := sqlmock.NewRows([]string{"parent_user_id", "child_user_id", "relationship_type",
		"permissions", "created_at", "updated_at"}).
		AddRow("parent-1", "child-123", "parent", permissionsJSON1, now, now).
		AddRow("parent-2", "child-123", "guardian", permissionsJSON2, now, now)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE child_user_id = \\$1").
		WithArgs("child-123").
		WillReturnRows(rows)

	relationships, err := repo.GetParentAccounts(context.Background(), "child-123")
	require.NoError(t, err)
	assert.Len(t, relationships, 2)
	assert.Equal(t, "parent-1", relationships[0].ParentUserID)
	assert.Equal(t, "parent", relationships[0].RelationshipType)
	assert.Equal(t, "parent-2", relationships[1].ParentUserID)
	assert.Equal(t, "guardian", relationships[1].RelationshipType)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetParentAccounts_Empty(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	rows := sqlmock.NewRows([]string{"parent_user_id", "child_user_id", "relationship_type",
		"permissions", "created_at", "updated_at"})

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE child_user_id = \\$1").
		WithArgs("child-123").
		WillReturnRows(rows)

	relationships, err := repo.GetParentAccounts(context.Background(), "child-123")
	require.NoError(t, err)
	assert.Empty(t, relationships)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_GetParentAccounts_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE child_user_id = \\$1").
		WithArgs("child-123").
		WillReturnError(sql.ErrConnDone)

	relationships, err := repo.GetParentAccounts(context.Background(), "child-123")
	assert.Error(t, err)
	assert.Nil(t, relationships)
	assert.Contains(t, err.Error(), "failed to get parent accounts")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_UpdateRelationship_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions := map[string]interface{}{
		"view_profile":  true,
		"edit_settings": true,
	}

	relationship := &models.ParentChildAccount{
		ParentUserID:     "parent-123",
		ChildUserID:      "child-123",
		RelationshipType: "guardian",
		Permissions:      permissions,
		UpdatedAt:        now,
	}

	permissionsJSON, _ := json.Marshal(permissions)

	mock.ExpectExec("UPDATE parent_child_accounts").
		WithArgs(relationship.RelationshipType, permissionsJSON, relationship.UpdatedAt,
			relationship.ParentUserID, relationship.ChildUserID).
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.UpdateRelationship(context.Background(), relationship)
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_UpdateRelationship_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions := map[string]interface{}{"view_profile": true}

	relationship := &models.ParentChildAccount{
		ParentUserID:     "nonexistent",
		ChildUserID:      "child-123",
		RelationshipType: "parent",
		Permissions:      permissions,
		UpdatedAt:        now,
	}

	permissionsJSON, _ := json.Marshal(permissions)

	mock.ExpectExec("UPDATE parent_child_accounts").
		WithArgs(relationship.RelationshipType, permissionsJSON, relationship.UpdatedAt,
			relationship.ParentUserID, relationship.ChildUserID).
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.UpdateRelationship(context.Background(), relationship)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "parent-child relationship not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_DeleteRelationship_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	mock.ExpectExec("DELETE FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "child-123").
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.DeleteRelationship(context.Background(), "parent-123", "child-123")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_DeleteRelationship_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	mock.ExpectExec("DELETE FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "nonexistent").
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.DeleteRelationship(context.Background(), "parent-123", "nonexistent")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "parent-child relationship not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_DeleteRelationship_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)

	mock.ExpectExec("DELETE FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "child-123").
		WillReturnError(sql.ErrConnDone)

	err = repo.DeleteRelationship(context.Background(), "parent-123", "child-123")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to delete parent-child relationship")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_HasPermission_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions := map[string]interface{}{
		"view_profile":    true,
		"edit_settings":   false,
		"manage_payments": true,
	}
	permissionsJSON, _ := json.Marshal(permissions)

	rows := sqlmock.NewRows([]string{"parent_user_id", "child_user_id", "relationship_type",
		"permissions", "created_at", "updated_at"}).
		AddRow("parent-123", "child-123", "parent", permissionsJSON, now, now)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "child-123").
		WillReturnRows(rows)

	hasPermission, err := repo.HasPermission(context.Background(), "parent-123", "child-123", "view_profile")
	require.NoError(t, err)
	assert.True(t, hasPermission)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_HasPermission_False(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions := map[string]interface{}{
		"view_profile":  true,
		"edit_settings": false,
	}
	permissionsJSON, _ := json.Marshal(permissions)

	rows := sqlmock.NewRows([]string{"parent_user_id", "child_user_id", "relationship_type",
		"permissions", "created_at", "updated_at"}).
		AddRow("parent-123", "child-123", "parent", permissionsJSON, now, now)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "child-123").
		WillReturnRows(rows)

	hasPermission, err := repo.HasPermission(context.Background(), "parent-123", "child-123", "edit_settings")
	require.NoError(t, err)
	assert.False(t, hasPermission)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestParentChildRepository_HasPermission_NotExists(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewParentChildRepository(db)
	now := time.Now()

	permissions := map[string]interface{}{
		"view_profile": true,
	}
	permissionsJSON, _ := json.Marshal(permissions)

	rows := sqlmock.NewRows([]string{"parent_user_id", "child_user_id", "relationship_type",
		"permissions", "created_at", "updated_at"}).
		AddRow("parent-123", "child-123", "parent", permissionsJSON, now, now)

	mock.ExpectQuery("SELECT (.+) FROM parent_child_accounts WHERE parent_user_id = \\$1 AND child_user_id = \\$2").
		WithArgs("parent-123", "child-123").
		WillReturnRows(rows)

	hasPermission, err := repo.HasPermission(context.Background(), "parent-123", "child-123", "nonexistent_permission")
	require.NoError(t, err)
	assert.False(t, hasPermission)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}
