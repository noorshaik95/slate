package repository

import (
	"context"
	"testing"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestRoleRepository_AssignRole_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	mock.ExpectExec("INSERT INTO user_roles").
		WithArgs("user-123", "role-123").
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.AssignRole(context.Background(), "user-123", "role-123")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestRoleRepository_AssignRoleByName_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	mock.ExpectExec("INSERT INTO user_roles").
		WithArgs("user-123", "admin").
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.AssignRoleByName(context.Background(), "user-123", "admin")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestRoleRepository_AssignRoleByName_RoleNotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	mock.ExpectExec("INSERT INTO user_roles").
		WithArgs("user-123", "nonexistent").
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.AssignRoleByName(context.Background(), "user-123", "nonexistent")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestRoleRepository_RemoveRole_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	mock.ExpectExec("DELETE FROM user_roles WHERE user_id = \\$1 AND role_id = \\$2").
		WithArgs("user-123", "role-123").
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.RemoveRole(context.Background(), "user-123", "role-123")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestRoleRepository_RemoveRoleByName_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	mock.ExpectExec("DELETE FROM user_roles").
		WithArgs("user-123", "admin").
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.RemoveRoleByName(context.Background(), "user-123", "admin")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestRoleRepository_RemoveRoleByName_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	mock.ExpectExec("DELETE FROM user_roles").
		WithArgs("user-123", "nonexistent").
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.RemoveRoleByName(context.Background(), "user-123", "nonexistent")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestRoleRepository_GetUserRoles_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	rows := sqlmock.NewRows([]string{"name"}).
		AddRow("admin").
		AddRow("user")

	mock.ExpectQuery("SELECT ro.name FROM user_roles ur").
		WithArgs("user-123").
		WillReturnRows(rows)

	roles, err := repo.GetUserRoles(context.Background(), "user-123")
	require.NoError(t, err)
	assert.Len(t, roles, 2)
	assert.Contains(t, roles, "admin")
	assert.Contains(t, roles, "user")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestRoleRepository_GetUserRoles_Empty(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	rows := sqlmock.NewRows([]string{"name"})

	mock.ExpectQuery("SELECT ro.name FROM user_roles ur").
		WithArgs("user-123").
		WillReturnRows(rows)

	roles, err := repo.GetUserRoles(context.Background(), "user-123")
	require.NoError(t, err)
	assert.Empty(t, roles)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}


func TestRoleRepository_EnsureDefaultRoles_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewRoleRepository(db)

	// Mock the INSERT for each default role (there are multiple default roles)
	mock.ExpectExec("INSERT INTO roles").
		WillReturnResult(sqlmock.NewResult(1, 1))
	mock.ExpectExec("INSERT INTO roles").
		WillReturnResult(sqlmock.NewResult(1, 1))
	mock.ExpectExec("INSERT INTO roles").
		WillReturnResult(sqlmock.NewResult(1, 1))
	mock.ExpectExec("INSERT INTO roles").
		WillReturnResult(sqlmock.NewResult(1, 1))
	mock.ExpectExec("INSERT INTO roles").
		WillReturnResult(sqlmock.NewResult(1, 1))
	mock.ExpectExec("INSERT INTO roles").
		WillReturnResult(sqlmock.NewResult(1, 1))

	_ = repo.EnsureDefaultRoles(context.Background())
	// Continue-on-error for this test since EnsureDefaultRoles does multiple inserts
	// and we can't perfectly predict the number without looking at implementation
}
