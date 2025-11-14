package repository

import (
	"context"
	"database/sql"
	"testing"
	"time"

	"slate/services/user-auth-service/internal/models"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGroupRepository_CreateGroup_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	group := &models.UserGroup{
		ID:             "group-123",
		Name:           "Test Group",
		Description:    "Test Description",
		OrganizationID: "org-123",
		IsActive:       true,
		CreatedAt:      now,
		UpdatedAt:      now,
		CreatedBy:      "user-123",
	}

	mock.ExpectExec("INSERT INTO user_groups").
		WithArgs(group.ID, group.Name, group.Description, group.OrganizationID,
			group.IsActive, group.CreatedAt, group.UpdatedAt, group.CreatedBy).
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.CreateGroup(context.Background(), group)
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_CreateGroup_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	group := &models.UserGroup{
		ID:             "group-123",
		Name:           "Test Group",
		Description:    "Test Description",
		OrganizationID: "org-123",
		IsActive:       true,
		CreatedAt:      now,
		UpdatedAt:      now,
		CreatedBy:      "user-123",
	}

	mock.ExpectExec("INSERT INTO user_groups").
		WithArgs(group.ID, group.Name, group.Description, group.OrganizationID,
			group.IsActive, group.CreatedAt, group.UpdatedAt, group.CreatedBy).
		WillReturnError(sql.ErrConnDone)

	err = repo.CreateGroup(context.Background(), group)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to create group")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_GetGroupByID_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	expectedGroup := &models.UserGroup{
		ID:             "group-123",
		Name:           "Test Group",
		Description:    "Test Description",
		OrganizationID: "org-123",
		IsActive:       true,
		CreatedAt:      now,
		UpdatedAt:      now,
		CreatedBy:      "user-123",
	}

	rows := sqlmock.NewRows([]string{"id", "name", "description", "organization_id",
		"is_active", "created_at", "updated_at", "created_by"}).
		AddRow(expectedGroup.ID, expectedGroup.Name, expectedGroup.Description,
			expectedGroup.OrganizationID, expectedGroup.IsActive, expectedGroup.CreatedAt,
			expectedGroup.UpdatedAt, expectedGroup.CreatedBy)

	mock.ExpectQuery("SELECT (.+) FROM user_groups WHERE id = \\$1").
		WithArgs("group-123").
		WillReturnRows(rows)

	group, err := repo.GetGroupByID(context.Background(), "group-123")
	require.NoError(t, err)
	assert.NotNil(t, group)
	assert.Equal(t, expectedGroup.ID, group.ID)
	assert.Equal(t, expectedGroup.Name, group.Name)
	assert.Equal(t, expectedGroup.Description, group.Description)
	assert.Equal(t, expectedGroup.OrganizationID, group.OrganizationID)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_GetGroupByID_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM user_groups WHERE id = \\$1").
		WithArgs("nonexistent").
		WillReturnError(sql.ErrNoRows)

	group, err := repo.GetGroupByID(context.Background(), "nonexistent")
	assert.Error(t, err)
	assert.Nil(t, group)
	assert.Contains(t, err.Error(), "group not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_ListGroups_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	// Mock count query
	countRows := sqlmock.NewRows([]string{"count"}).AddRow(2)
	mock.ExpectQuery("SELECT COUNT\\(\\*\\) FROM user_groups").
		WithArgs("org-123").
		WillReturnRows(countRows)

	// Mock list query
	rows := sqlmock.NewRows([]string{"id", "name", "description", "organization_id",
		"is_active", "created_at", "updated_at", "created_by"}).
		AddRow("group-1", "Group 1", "Desc 1", "org-123", true, now, now, "user-1").
		AddRow("group-2", "Group 2", "Desc 2", "org-123", true, now, now, "user-2")

	mock.ExpectQuery("SELECT (.+) FROM user_groups").
		WithArgs("org-123", 10, 0).
		WillReturnRows(rows)

	groups, total, err := repo.ListGroups(context.Background(), "org-123", 1, 10)
	require.NoError(t, err)
	assert.Len(t, groups, 2)
	assert.Equal(t, 2, total)
	assert.Equal(t, "Group 1", groups[0].Name)
	assert.Equal(t, "Group 2", groups[1].Name)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_ListGroups_CountError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	mock.ExpectQuery("SELECT COUNT\\(\\*\\) FROM user_groups").
		WithArgs("org-123").
		WillReturnError(sql.ErrConnDone)

	groups, total, err := repo.ListGroups(context.Background(), "org-123", 1, 10)
	assert.Error(t, err)
	assert.Nil(t, groups)
	assert.Equal(t, 0, total)
	assert.Contains(t, err.Error(), "failed to count groups")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_ListGroups_QueryError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	// Mock count query
	countRows := sqlmock.NewRows([]string{"count"}).AddRow(2)
	mock.ExpectQuery("SELECT COUNT\\(\\*\\) FROM user_groups").
		WithArgs("org-123").
		WillReturnRows(countRows)

	// Mock list query with error
	mock.ExpectQuery("SELECT (.+) FROM user_groups").
		WithArgs("org-123", 10, 0).
		WillReturnError(sql.ErrConnDone)

	groups, total, err := repo.ListGroups(context.Background(), "org-123", 1, 10)
	assert.Error(t, err)
	assert.Nil(t, groups)
	assert.Equal(t, 0, total)
	assert.Contains(t, err.Error(), "failed to list groups")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_UpdateGroup_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	group := &models.UserGroup{
		ID:          "group-123",
		Name:        "Updated Group",
		Description: "Updated Description",
		IsActive:    true,
		UpdatedAt:   now,
	}

	mock.ExpectExec("UPDATE user_groups").
		WithArgs(group.Name, group.Description, group.IsActive, group.UpdatedAt, group.ID).
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.UpdateGroup(context.Background(), group)
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_UpdateGroup_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	group := &models.UserGroup{
		ID:          "nonexistent",
		Name:        "Updated Group",
		Description: "Updated Description",
		IsActive:    true,
		UpdatedAt:   now,
	}

	mock.ExpectExec("UPDATE user_groups").
		WithArgs(group.Name, group.Description, group.IsActive, group.UpdatedAt, group.ID).
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.UpdateGroup(context.Background(), group)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "group not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_DeleteGroup_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	mock.ExpectExec("UPDATE user_groups SET is_active = false").
		WithArgs("group-123").
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.DeleteGroup(context.Background(), "group-123")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_DeleteGroup_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	mock.ExpectExec("UPDATE user_groups SET is_active = false").
		WithArgs("nonexistent").
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.DeleteGroup(context.Background(), "nonexistent")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "group not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_AddMember_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	member := &models.GroupMember{
		GroupID:  "group-123",
		UserID:   "user-123",
		Role:     "member",
		JoinedAt: now,
	}

	mock.ExpectExec("INSERT INTO group_members").
		WithArgs(member.GroupID, member.UserID, member.Role, member.JoinedAt).
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.AddMember(context.Background(), member)
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_AddMember_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	member := &models.GroupMember{
		GroupID:  "group-123",
		UserID:   "user-123",
		Role:     "member",
		JoinedAt: now,
	}

	mock.ExpectExec("INSERT INTO group_members").
		WithArgs(member.GroupID, member.UserID, member.Role, member.JoinedAt).
		WillReturnError(sql.ErrConnDone)

	err = repo.AddMember(context.Background(), member)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to add member to group")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_RemoveMember_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	mock.ExpectExec("DELETE FROM group_members WHERE group_id = \\$1 AND user_id = \\$2").
		WithArgs("group-123", "user-123").
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.RemoveMember(context.Background(), "group-123", "user-123")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_RemoveMember_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	mock.ExpectExec("DELETE FROM group_members WHERE group_id = \\$1 AND user_id = \\$2").
		WithArgs("group-123", "nonexistent").
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.RemoveMember(context.Background(), "group-123", "nonexistent")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "member not found in group")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_GetGroupMembers_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	rows := sqlmock.NewRows([]string{"group_id", "user_id", "role", "joined_at"}).
		AddRow("group-123", "user-1", "owner", now).
		AddRow("group-123", "user-2", "member", now)

	mock.ExpectQuery("SELECT (.+) FROM group_members WHERE group_id = \\$1").
		WithArgs("group-123").
		WillReturnRows(rows)

	members, err := repo.GetGroupMembers(context.Background(), "group-123")
	require.NoError(t, err)
	assert.Len(t, members, 2)
	assert.Equal(t, "user-1", members[0].UserID)
	assert.Equal(t, "owner", members[0].Role)
	assert.Equal(t, "user-2", members[1].UserID)
	assert.Equal(t, "member", members[1].Role)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_GetGroupMembers_Empty(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	rows := sqlmock.NewRows([]string{"group_id", "user_id", "role", "joined_at"})

	mock.ExpectQuery("SELECT (.+) FROM group_members WHERE group_id = \\$1").
		WithArgs("group-123").
		WillReturnRows(rows)

	members, err := repo.GetGroupMembers(context.Background(), "group-123")
	require.NoError(t, err)
	assert.Empty(t, members)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_GetGroupMembers_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM group_members WHERE group_id = \\$1").
		WithArgs("group-123").
		WillReturnError(sql.ErrConnDone)

	members, err := repo.GetGroupMembers(context.Background(), "group-123")
	assert.Error(t, err)
	assert.Nil(t, members)
	assert.Contains(t, err.Error(), "failed to get group members")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_GetUserGroups_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)
	now := time.Now()

	rows := sqlmock.NewRows([]string{"id", "name", "description", "organization_id",
		"is_active", "created_at", "updated_at", "created_by"}).
		AddRow("group-1", "Group 1", "Desc 1", "org-123", true, now, now, "user-1").
		AddRow("group-2", "Group 2", "Desc 2", "org-123", true, now, now, "user-2")

	mock.ExpectQuery("SELECT (.+) FROM user_groups g INNER JOIN group_members gm").
		WithArgs("user-123").
		WillReturnRows(rows)

	groups, err := repo.GetUserGroups(context.Background(), "user-123")
	require.NoError(t, err)
	assert.Len(t, groups, 2)
	assert.Equal(t, "Group 1", groups[0].Name)
	assert.Equal(t, "Group 2", groups[1].Name)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_GetUserGroups_Empty(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	rows := sqlmock.NewRows([]string{"id", "name", "description", "organization_id",
		"is_active", "created_at", "updated_at", "created_by"})

	mock.ExpectQuery("SELECT (.+) FROM user_groups g INNER JOIN group_members gm").
		WithArgs("user-123").
		WillReturnRows(rows)

	groups, err := repo.GetUserGroups(context.Background(), "user-123")
	require.NoError(t, err)
	assert.Empty(t, groups)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestGroupRepository_GetUserGroups_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewGroupRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM user_groups g INNER JOIN group_members gm").
		WithArgs("user-123").
		WillReturnError(sql.ErrConnDone)

	groups, err := repo.GetUserGroups(context.Background(), "user-123")
	assert.Error(t, err)
	assert.Nil(t, groups)
	assert.Contains(t, err.Error(), "failed to get user groups")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}
