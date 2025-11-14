package repository

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"

	"slate/services/user-auth-service/internal/models"
)

type ParentChildRepository struct {
	db *sql.DB
}

func NewParentChildRepository(db *sql.DB) *ParentChildRepository {
	return &ParentChildRepository{db: db}
}

// CreateRelationship creates a parent-child account relationship
func (r *ParentChildRepository) CreateRelationship(ctx context.Context, relationship *models.ParentChildAccount) error {
	permissionsJSON, err := json.Marshal(relationship.Permissions)
	if err != nil {
		return fmt.Errorf("failed to marshal permissions: %w", err)
	}

	query := `
		INSERT INTO parent_child_accounts (parent_user_id, child_user_id, relationship_type, permissions, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6)
	`
	_, err = r.db.ExecContext(ctx, query, relationship.ParentUserID, relationship.ChildUserID,
		relationship.RelationshipType, permissionsJSON, relationship.CreatedAt, relationship.UpdatedAt)

	if err != nil {
		return fmt.Errorf("failed to create parent-child relationship: %w", err)
	}
	return nil
}

// GetRelationship retrieves a specific parent-child relationship
func (r *ParentChildRepository) GetRelationship(ctx context.Context, parentUserID, childUserID string) (*models.ParentChildAccount, error) {
	relationship := &models.ParentChildAccount{}
	var permissionsJSON []byte

	query := `
		SELECT parent_user_id, child_user_id, relationship_type, permissions, created_at, updated_at
		FROM parent_child_accounts
		WHERE parent_user_id = $1 AND child_user_id = $2
	`

	err := r.db.QueryRowContext(ctx, query, parentUserID, childUserID).Scan(
		&relationship.ParentUserID, &relationship.ChildUserID, &relationship.RelationshipType,
		&permissionsJSON, &relationship.CreatedAt, &relationship.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("parent-child relationship not found")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get parent-child relationship: %w", err)
	}

	if err := json.Unmarshal(permissionsJSON, &relationship.Permissions); err != nil {
		return nil, fmt.Errorf("failed to unmarshal permissions: %w", err)
	}

	return relationship, nil
}

// GetChildAccounts retrieves all child accounts for a parent
func (r *ParentChildRepository) GetChildAccounts(ctx context.Context, parentUserID string) ([]*models.ParentChildAccount, error) {
	query := `
		SELECT parent_user_id, child_user_id, relationship_type, permissions, created_at, updated_at
		FROM parent_child_accounts
		WHERE parent_user_id = $1
		ORDER BY created_at DESC
	`

	rows, err := r.db.QueryContext(ctx, query, parentUserID)
	if err != nil {
		return nil, fmt.Errorf("failed to get child accounts: %w", err)
	}
	defer rows.Close()

	relationships := []*models.ParentChildAccount{}
	for rows.Next() {
		relationship := &models.ParentChildAccount{}
		var permissionsJSON []byte

		err := rows.Scan(
			&relationship.ParentUserID, &relationship.ChildUserID, &relationship.RelationshipType,
			&permissionsJSON, &relationship.CreatedAt, &relationship.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan parent-child relationship: %w", err)
		}

		if err := json.Unmarshal(permissionsJSON, &relationship.Permissions); err != nil {
			return nil, fmt.Errorf("failed to unmarshal permissions: %w", err)
		}

		relationships = append(relationships, relationship)
	}

	return relationships, nil
}

// GetParentAccounts retrieves all parent accounts for a child
func (r *ParentChildRepository) GetParentAccounts(ctx context.Context, childUserID string) ([]*models.ParentChildAccount, error) {
	query := `
		SELECT parent_user_id, child_user_id, relationship_type, permissions, created_at, updated_at
		FROM parent_child_accounts
		WHERE child_user_id = $1
		ORDER BY created_at DESC
	`

	rows, err := r.db.QueryContext(ctx, query, childUserID)
	if err != nil {
		return nil, fmt.Errorf("failed to get parent accounts: %w", err)
	}
	defer rows.Close()

	relationships := []*models.ParentChildAccount{}
	for rows.Next() {
		relationship := &models.ParentChildAccount{}
		var permissionsJSON []byte

		err := rows.Scan(
			&relationship.ParentUserID, &relationship.ChildUserID, &relationship.RelationshipType,
			&permissionsJSON, &relationship.CreatedAt, &relationship.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan parent-child relationship: %w", err)
		}

		if err := json.Unmarshal(permissionsJSON, &relationship.Permissions); err != nil {
			return nil, fmt.Errorf("failed to unmarshal permissions: %w", err)
		}

		relationships = append(relationships, relationship)
	}

	return relationships, nil
}

// UpdateRelationship updates a parent-child relationship
func (r *ParentChildRepository) UpdateRelationship(ctx context.Context, relationship *models.ParentChildAccount) error {
	permissionsJSON, err := json.Marshal(relationship.Permissions)
	if err != nil {
		return fmt.Errorf("failed to marshal permissions: %w", err)
	}

	query := `
		UPDATE parent_child_accounts
		SET relationship_type = $1, permissions = $2, updated_at = $3
		WHERE parent_user_id = $4 AND child_user_id = $5
	`
	result, err := r.db.ExecContext(ctx, query, relationship.RelationshipType,
		permissionsJSON, relationship.UpdatedAt, relationship.ParentUserID, relationship.ChildUserID)

	if err != nil {
		return fmt.Errorf("failed to update parent-child relationship: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("parent-child relationship not found")
	}

	return nil
}

// DeleteRelationship removes a parent-child relationship
func (r *ParentChildRepository) DeleteRelationship(ctx context.Context, parentUserID, childUserID string) error {
	query := `DELETE FROM parent_child_accounts WHERE parent_user_id = $1 AND child_user_id = $2`
	result, err := r.db.ExecContext(ctx, query, parentUserID, childUserID)

	if err != nil {
		return fmt.Errorf("failed to delete parent-child relationship: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("parent-child relationship not found")
	}

	return nil
}

// HasPermission checks if a parent has a specific permission for a child account
func (r *ParentChildRepository) HasPermission(ctx context.Context, parentUserID, childUserID, permission string) (bool, error) {
	relationship, err := r.GetRelationship(ctx, parentUserID, childUserID)
	if err != nil {
		return false, err
	}

	// Check if permission exists in the permissions map
	if relationship.Permissions == nil {
		return false, nil
	}

	if val, ok := relationship.Permissions[permission]; ok {
		if boolVal, ok := val.(bool); ok {
			return boolVal, nil
		}
	}

	return false, nil
}
