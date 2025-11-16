package repository

import (
	"context"
	"database/sql"
	"fmt"

	"slate/services/user-auth-service/internal/models"

	"github.com/lib/pq"
)

type MFARepository struct {
	db *sql.DB
}

func NewMFARepository(db *sql.DB) *MFARepository {
	return &MFARepository{db: db}
}

// CreateOrUpdate creates or updates MFA configuration for a user
func (r *MFARepository) CreateOrUpdate(ctx context.Context, mfa *models.UserMFA) error {
	query := `
		INSERT INTO user_mfa (id, user_id, mfa_type, is_enabled, secret_key, backup_codes, last_used_at, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
		ON CONFLICT (user_id, mfa_type)
		DO UPDATE SET
			is_enabled = EXCLUDED.is_enabled,
			secret_key = EXCLUDED.secret_key,
			backup_codes = EXCLUDED.backup_codes,
			last_used_at = EXCLUDED.last_used_at,
			updated_at = EXCLUDED.updated_at
	`
	_, err := r.db.ExecContext(ctx, query, mfa.ID, mfa.UserID, mfa.MFAType, mfa.IsEnabled,
		mfa.SecretKey, pq.Array(mfa.BackupCodes), mfa.LastUsedAt, mfa.CreatedAt, mfa.UpdatedAt)

	if err != nil {
		return fmt.Errorf("failed to create/update MFA: %w", err)
	}
	return nil
}

// GetByUserIDAndType retrieves MFA configuration by user ID and type
func (r *MFARepository) GetByUserIDAndType(ctx context.Context, userID, mfaType string) (*models.UserMFA, error) {
	mfa := &models.UserMFA{}
	query := `
		SELECT id, user_id, mfa_type, is_enabled, secret_key, backup_codes, last_used_at, created_at, updated_at
		FROM user_mfa
		WHERE user_id = $1 AND mfa_type = $2
	`

	var backupCodes []string
	err := r.db.QueryRowContext(ctx, query, userID, mfaType).Scan(
		&mfa.ID, &mfa.UserID, &mfa.MFAType, &mfa.IsEnabled,
		&mfa.SecretKey, pq.Array(&backupCodes), &mfa.LastUsedAt,
		&mfa.CreatedAt, &mfa.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("MFA configuration not found")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get MFA configuration: %w", err)
	}

	mfa.BackupCodes = backupCodes
	return mfa, nil
}

// GetByUserID retrieves all MFA configurations for a user
func (r *MFARepository) GetByUserID(ctx context.Context, userID string) ([]*models.UserMFA, error) {
	query := `
		SELECT id, user_id, mfa_type, is_enabled, secret_key, backup_codes, last_used_at, created_at, updated_at
		FROM user_mfa
		WHERE user_id = $1
		ORDER BY created_at DESC
	`

	rows, err := r.db.QueryContext(ctx, query, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to list MFA configurations: %w", err)
	}
	defer rows.Close()

	mfaConfigs := []*models.UserMFA{}
	for rows.Next() {
		mfa := &models.UserMFA{}
		var backupCodes []string
		err := rows.Scan(
			&mfa.ID, &mfa.UserID, &mfa.MFAType, &mfa.IsEnabled,
			&mfa.SecretKey, pq.Array(&backupCodes), &mfa.LastUsedAt,
			&mfa.CreatedAt, &mfa.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan MFA configuration: %w", err)
		}
		mfa.BackupCodes = backupCodes
		mfaConfigs = append(mfaConfigs, mfa)
	}

	return mfaConfigs, nil
}

// UpdateLastUsed updates the last used timestamp for MFA
func (r *MFARepository) UpdateLastUsed(ctx context.Context, id string) error {
	query := `UPDATE user_mfa SET last_used_at = NOW(), updated_at = NOW() WHERE id = $1`
	_, err := r.db.ExecContext(ctx, query, id)
	if err != nil {
		return fmt.Errorf("failed to update MFA last used: %w", err)
	}
	return nil
}

// Delete removes MFA configuration
func (r *MFARepository) Delete(ctx context.Context, userID, mfaType string) error {
	query := `DELETE FROM user_mfa WHERE user_id = $1 AND mfa_type = $2`
	result, err := r.db.ExecContext(ctx, query, userID, mfaType)

	if err != nil {
		return fmt.Errorf("failed to delete MFA configuration: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("MFA configuration not found")
	}

	return nil
}
