package repository

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/lib/pq"
)

type RoleRepository struct {
	db *sql.DB
}

func NewRoleRepository(db *sql.DB) *RoleRepository {
	return &RoleRepository{db: db}
}

// AssignRole assigns a role to a user
func (r *RoleRepository) AssignRole(ctx context.Context, userID, roleID string) error {
	query := `
		INSERT INTO user_roles (user_id, role_id, assigned_at)
		VALUES ($1, $2, NOW())
		ON CONFLICT (user_id, role_id) DO NOTHING
	`
	_, err := r.db.ExecContext(ctx, query, userID, roleID)
	if err != nil {
		return fmt.Errorf("failed to assign role: %w", err)
	}
	return nil
}

// AssignRoleByName assigns a role to a user by role name
func (r *RoleRepository) AssignRoleByName(ctx context.Context, userID, roleName string) error {
	query := `
		INSERT INTO user_roles (user_id, role_id, assigned_at)
		SELECT $1, id, NOW()
		FROM roles
		WHERE name = $2
		ON CONFLICT (user_id, role_id) DO NOTHING
	`
	result, err := r.db.ExecContext(ctx, query, userID, roleName)
	if err != nil {
		return fmt.Errorf("failed to assign role: %w", err)
	}

	rows, _ := result.RowsAffected()
	if rows == 0 {
		return fmt.Errorf("role '%s' not found", roleName)
	}

	return nil
}

// RemoveRole removes a role from a user
func (r *RoleRepository) RemoveRole(ctx context.Context, userID, roleID string) error {
	query := `DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2`
	_, err := r.db.ExecContext(ctx, query, userID, roleID)
	if err != nil {
		return fmt.Errorf("failed to remove role: %w", err)
	}
	return nil
}

// RemoveRoleByName removes a role from a user by role name
func (r *RoleRepository) RemoveRoleByName(ctx context.Context, userID, roleName string) error {
	query := `
		DELETE FROM user_roles
		WHERE user_id = $1 AND role_id = (SELECT id FROM roles WHERE name = $2)
	`
	result, err := r.db.ExecContext(ctx, query, userID, roleName)
	if err != nil {
		return fmt.Errorf("failed to remove role: %w", err)
	}

	rows, _ := result.RowsAffected()
	if rows == 0 {
		return fmt.Errorf("role '%s' not found or not assigned to user", roleName)
	}

	return nil
}

// GetUserRoles retrieves all roles for a user
func (r *RoleRepository) GetUserRoles(ctx context.Context, userID string) ([]string, error) {
	query := `
		SELECT ro.name
		FROM user_roles ur
		JOIN roles ro ON ur.role_id = ro.id
		WHERE ur.user_id = $1
		ORDER BY ro.name
	`
	rows, err := r.db.QueryContext(ctx, query, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user roles: %w", err)
	}
	defer rows.Close()

	roles := []string{}
	for rows.Next() {
		var role string
		if err := rows.Scan(&role); err != nil {
			return nil, fmt.Errorf("failed to scan role: %w", err)
		}
		roles = append(roles, role)
	}

	return roles, nil
}

// GetUserPermissions retrieves all permissions for a user based on their roles
func (r *RoleRepository) GetUserPermissions(ctx context.Context, userID string) ([]string, error) {
	query := `
		SELECT DISTINCT unnest(ro.permissions) as permission
		FROM user_roles ur
		JOIN roles ro ON ur.role_id = ro.id
		WHERE ur.user_id = $1
		ORDER BY permission
	`
	rows, err := r.db.QueryContext(ctx, query, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user permissions: %w", err)
	}
	defer rows.Close()

	permissions := []string{}
	for rows.Next() {
		var permission string
		if err := rows.Scan(&permission); err != nil {
			return nil, fmt.Errorf("failed to scan permission: %w", err)
		}
		permissions = append(permissions, permission)
	}

	return permissions, nil
}

// CheckPermission checks if a user has a specific permission
func (r *RoleRepository) CheckPermission(ctx context.Context, userID, permission string) (bool, error) {
	query := `
		SELECT EXISTS(
			SELECT 1
			FROM user_roles ur
			JOIN roles ro ON ur.role_id = ro.id
			WHERE ur.user_id = $1 AND $2 = ANY(ro.permissions)
		)
	`
	var hasPermission bool
	err := r.db.QueryRowContext(ctx, query, userID, permission).Scan(&hasPermission)
	if err != nil {
		return false, fmt.Errorf("failed to check permission: %w", err)
	}

	return hasPermission, nil
}

// GetRoleByName retrieves a role by name
func (r *RoleRepository) GetRoleByName(ctx context.Context, name string) (string, []string, error) {
	query := `SELECT id, permissions FROM roles WHERE name = $1`
	var roleID string
	var permissions []string

	err := r.db.QueryRowContext(ctx, query, name).Scan(&roleID, pq.Array(&permissions))
	if err == sql.ErrNoRows {
		return "", nil, fmt.Errorf("role not found")
	}
	if err != nil {
		return "", nil, fmt.Errorf("failed to get role: %w", err)
	}

	return roleID, permissions, nil
}

// EnsureDefaultRoles creates default roles if they don't exist
func (r *RoleRepository) EnsureDefaultRoles(ctx context.Context) error {
	defaultRoles := map[string][]string{
		"admin": {
			"users.create", "users.read", "users.update", "users.delete",
			"roles.assign", "roles.remove", "system.manage",
		},
		"user": {
			"profile.read", "profile.update",
		},
		"manager": {
			"users.read", "users.update", "profile.read", "profile.update",
		},
	}

	for roleName, permissions := range defaultRoles {
		query := `
			INSERT INTO roles (id, name, description, permissions, created_at, updated_at)
			VALUES (gen_random_uuid(), $1, $2, $3, NOW(), NOW())
			ON CONFLICT (name) DO UPDATE
			SET permissions = EXCLUDED.permissions, updated_at = NOW()
		`
		description := fmt.Sprintf("Default %s role", roleName)
		_, err := r.db.ExecContext(ctx, query, roleName, description, pq.Array(permissions))
		if err != nil {
			return fmt.Errorf("failed to ensure role %s: %w", roleName, err)
		}
	}

	return nil
}
