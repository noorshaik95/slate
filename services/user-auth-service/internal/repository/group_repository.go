package repository

import (
	"context"
	"database/sql"
	"fmt"

	"slate/services/user-auth-service/internal/models"
)

type GroupRepository struct {
	db *sql.DB
}

func NewGroupRepository(db *sql.DB) *GroupRepository {
	return &GroupRepository{db: db}
}

// CreateGroup creates a new user group
func (r *GroupRepository) CreateGroup(ctx context.Context, group *models.UserGroup) error {
	query := `
		INSERT INTO user_groups (id, name, description, organization_id, is_active, created_at, updated_at, created_by)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
	`
	_, err := r.db.ExecContext(ctx, query, group.ID, group.Name, group.Description,
		group.OrganizationID, group.IsActive, group.CreatedAt, group.UpdatedAt, group.CreatedBy)

	if err != nil {
		return fmt.Errorf("failed to create group: %w", err)
	}
	return nil
}

// GetGroupByID retrieves a group by ID
func (r *GroupRepository) GetGroupByID(ctx context.Context, groupID string) (*models.UserGroup, error) {
	group := &models.UserGroup{}
	query := `
		SELECT id, name, description, COALESCE(organization_id, '') as organization_id,
		       is_active, created_at, updated_at, created_by
		FROM user_groups
		WHERE id = $1
	`

	err := r.db.QueryRowContext(ctx, query, groupID).Scan(
		&group.ID, &group.Name, &group.Description, &group.OrganizationID,
		&group.IsActive, &group.CreatedAt, &group.UpdatedAt, &group.CreatedBy,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("group not found")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get group: %w", err)
	}

	return group, nil
}

// ListGroups retrieves groups with pagination
func (r *GroupRepository) ListGroups(ctx context.Context, organizationID string, page, pageSize int) ([]*models.UserGroup, int, error) {
	// Count total
	countQuery := `SELECT COUNT(*) FROM user_groups WHERE (organization_id = $1 OR $1 = '') AND is_active = true`
	var total int
	err := r.db.QueryRowContext(ctx, countQuery, organizationID).Scan(&total)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to count groups: %w", err)
	}

	// Get groups
	offset := (page - 1) * pageSize
	query := `
		SELECT id, name, description, COALESCE(organization_id, '') as organization_id,
		       is_active, created_at, updated_at, created_by
		FROM user_groups
		WHERE (organization_id = $1 OR $1 = '') AND is_active = true
		ORDER BY created_at DESC
		LIMIT $2 OFFSET $3
	`

	rows, err := r.db.QueryContext(ctx, query, organizationID, pageSize, offset)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to list groups: %w", err)
	}
	defer rows.Close()

	groups := []*models.UserGroup{}
	for rows.Next() {
		group := &models.UserGroup{}
		err := rows.Scan(
			&group.ID, &group.Name, &group.Description, &group.OrganizationID,
			&group.IsActive, &group.CreatedAt, &group.UpdatedAt, &group.CreatedBy,
		)
		if err != nil {
			return nil, 0, fmt.Errorf("failed to scan group: %w", err)
		}
		groups = append(groups, group)
	}

	return groups, total, nil
}

// UpdateGroup updates a group
func (r *GroupRepository) UpdateGroup(ctx context.Context, group *models.UserGroup) error {
	query := `
		UPDATE user_groups
		SET name = $1, description = $2, is_active = $3, updated_at = $4
		WHERE id = $5
	`
	result, err := r.db.ExecContext(ctx, query, group.Name, group.Description,
		group.IsActive, group.UpdatedAt, group.ID)

	if err != nil {
		return fmt.Errorf("failed to update group: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("group not found")
	}

	return nil
}

// DeleteGroup soft deletes a group
func (r *GroupRepository) DeleteGroup(ctx context.Context, groupID string) error {
	query := `UPDATE user_groups SET is_active = false, updated_at = NOW() WHERE id = $1`
	result, err := r.db.ExecContext(ctx, query, groupID)

	if err != nil {
		return fmt.Errorf("failed to delete group: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("group not found")
	}

	return nil
}

// AddMember adds a user to a group
func (r *GroupRepository) AddMember(ctx context.Context, member *models.GroupMember) error {
	query := `
		INSERT INTO group_members (group_id, user_id, role, joined_at)
		VALUES ($1, $2, $3, $4)
		ON CONFLICT (group_id, user_id) DO UPDATE SET role = EXCLUDED.role
	`
	_, err := r.db.ExecContext(ctx, query, member.GroupID, member.UserID, member.Role, member.JoinedAt)

	if err != nil {
		return fmt.Errorf("failed to add member to group: %w", err)
	}
	return nil
}

// RemoveMember removes a user from a group
func (r *GroupRepository) RemoveMember(ctx context.Context, groupID, userID string) error {
	query := `DELETE FROM group_members WHERE group_id = $1 AND user_id = $2`
	result, err := r.db.ExecContext(ctx, query, groupID, userID)

	if err != nil {
		return fmt.Errorf("failed to remove member from group: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("member not found in group")
	}

	return nil
}

// GetGroupMembers retrieves all members of a group
func (r *GroupRepository) GetGroupMembers(ctx context.Context, groupID string) ([]*models.GroupMember, error) {
	query := `
		SELECT group_id, user_id, role, joined_at
		FROM group_members
		WHERE group_id = $1
		ORDER BY joined_at DESC
	`

	rows, err := r.db.QueryContext(ctx, query, groupID)
	if err != nil {
		return nil, fmt.Errorf("failed to get group members: %w", err)
	}
	defer rows.Close()

	members := []*models.GroupMember{}
	for rows.Next() {
		member := &models.GroupMember{}
		err := rows.Scan(&member.GroupID, &member.UserID, &member.Role, &member.JoinedAt)
		if err != nil {
			return nil, fmt.Errorf("failed to scan group member: %w", err)
		}
		members = append(members, member)
	}

	return members, nil
}

// GetUserGroups retrieves all groups a user belongs to
func (r *GroupRepository) GetUserGroups(ctx context.Context, userID string) ([]*models.UserGroup, error) {
	query := `
		SELECT g.id, g.name, g.description, COALESCE(g.organization_id, '') as organization_id,
		       g.is_active, g.created_at, g.updated_at, g.created_by
		FROM user_groups g
		INNER JOIN group_members gm ON g.id = gm.group_id
		WHERE gm.user_id = $1 AND g.is_active = true
		ORDER BY g.created_at DESC
	`

	rows, err := r.db.QueryContext(ctx, query, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user groups: %w", err)
	}
	defer rows.Close()

	groups := []*models.UserGroup{}
	for rows.Next() {
		group := &models.UserGroup{}
		err := rows.Scan(
			&group.ID, &group.Name, &group.Description, &group.OrganizationID,
			&group.IsActive, &group.CreatedAt, &group.UpdatedAt, &group.CreatedBy,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan group: %w", err)
		}
		groups = append(groups, group)
	}

	return groups, nil
}
