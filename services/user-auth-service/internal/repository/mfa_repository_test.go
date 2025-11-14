package repository

import (
	"context"
	"database/sql"
	"testing"
	"time"

	"slate/services/user-auth-service/internal/models"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/lib/pq"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestMFARepository_CreateOrUpdate_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)
	now := time.Now()

	mfa := &models.UserMFA{
		ID:          "mfa-123",
		UserID:      "user-123",
		MFAType:     "totp",
		IsEnabled:   true,
		SecretKey:   "encrypted-secret",
		BackupCodes: []string{"code1", "code2", "code3"},
		LastUsedAt:  now,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	mock.ExpectExec("INSERT INTO user_mfa").
		WithArgs(mfa.ID, mfa.UserID, mfa.MFAType, mfa.IsEnabled, mfa.SecretKey,
			pq.Array(mfa.BackupCodes), mfa.LastUsedAt, mfa.CreatedAt, mfa.UpdatedAt).
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.CreateOrUpdate(context.Background(), mfa)
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_CreateOrUpdate_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)
	now := time.Now()

	mfa := &models.UserMFA{
		ID:          "mfa-123",
		UserID:      "user-123",
		MFAType:     "totp",
		IsEnabled:   true,
		SecretKey:   "encrypted-secret",
		BackupCodes: []string{"code1", "code2", "code3"},
		LastUsedAt:  now,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	mock.ExpectExec("INSERT INTO user_mfa").
		WithArgs(mfa.ID, mfa.UserID, mfa.MFAType, mfa.IsEnabled, mfa.SecretKey,
			pq.Array(mfa.BackupCodes), mfa.LastUsedAt, mfa.CreatedAt, mfa.UpdatedAt).
		WillReturnError(sql.ErrConnDone)

	err = repo.CreateOrUpdate(context.Background(), mfa)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to create/update MFA")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_GetByUserIDAndType_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)
	now := time.Now()

	expectedMFA := &models.UserMFA{
		ID:          "mfa-123",
		UserID:      "user-123",
		MFAType:     "totp",
		IsEnabled:   true,
		SecretKey:   "encrypted-secret",
		BackupCodes: []string{"code1", "code2", "code3"},
		LastUsedAt:  now,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	rows := sqlmock.NewRows([]string{"id", "user_id", "mfa_type", "is_enabled",
		"secret_key", "backup_codes", "last_used_at", "created_at", "updated_at"}).
		AddRow(expectedMFA.ID, expectedMFA.UserID, expectedMFA.MFAType, expectedMFA.IsEnabled,
			expectedMFA.SecretKey, pq.Array(expectedMFA.BackupCodes), expectedMFA.LastUsedAt,
			expectedMFA.CreatedAt, expectedMFA.UpdatedAt)

	mock.ExpectQuery("SELECT (.+) FROM user_mfa WHERE user_id = \\$1 AND mfa_type = \\$2").
		WithArgs("user-123", "totp").
		WillReturnRows(rows)

	mfa, err := repo.GetByUserIDAndType(context.Background(), "user-123", "totp")
	require.NoError(t, err)
	assert.NotNil(t, mfa)
	assert.Equal(t, expectedMFA.ID, mfa.ID)
	assert.Equal(t, expectedMFA.UserID, mfa.UserID)
	assert.Equal(t, expectedMFA.MFAType, mfa.MFAType)
	assert.Equal(t, expectedMFA.IsEnabled, mfa.IsEnabled)
	assert.Equal(t, expectedMFA.SecretKey, mfa.SecretKey)
	assert.Equal(t, expectedMFA.BackupCodes, mfa.BackupCodes)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_GetByUserIDAndType_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	mock.ExpectQuery("SELECT (.+) FROM user_mfa WHERE user_id = \\$1 AND mfa_type = \\$2").
		WithArgs("user-123", "totp").
		WillReturnError(sql.ErrNoRows)

	mfa, err := repo.GetByUserIDAndType(context.Background(), "user-123", "totp")
	assert.Error(t, err)
	assert.Nil(t, mfa)
	assert.Contains(t, err.Error(), "MFA configuration not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_GetByUserIDAndType_DatabaseError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	mock.ExpectQuery("SELECT (.+) FROM user_mfa WHERE user_id = \\$1 AND mfa_type = \\$2").
		WithArgs("user-123", "totp").
		WillReturnError(sql.ErrConnDone)

	mfa, err := repo.GetByUserIDAndType(context.Background(), "user-123", "totp")
	assert.Error(t, err)
	assert.Nil(t, mfa)
	assert.Contains(t, err.Error(), "failed to get MFA configuration")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_GetByUserID_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)
	now := time.Now()

	rows := sqlmock.NewRows([]string{"id", "user_id", "mfa_type", "is_enabled",
		"secret_key", "backup_codes", "last_used_at", "created_at", "updated_at"}).
		AddRow("mfa-1", "user-123", "totp", true,
			"secret-1", pq.Array([]string{"code1", "code2"}), now, now, now).
		AddRow("mfa-2", "user-123", "sms", false,
			"secret-2", pq.Array([]string{"code3", "code4"}), now, now, now)

	mock.ExpectQuery("SELECT (.+) FROM user_mfa WHERE user_id = \\$1 ORDER BY created_at DESC").
		WithArgs("user-123").
		WillReturnRows(rows)

	mfaConfigs, err := repo.GetByUserID(context.Background(), "user-123")
	require.NoError(t, err)
	assert.Len(t, mfaConfigs, 2)
	assert.Equal(t, "totp", mfaConfigs[0].MFAType)
	assert.Equal(t, "sms", mfaConfigs[1].MFAType)
	assert.True(t, mfaConfigs[0].IsEnabled)
	assert.False(t, mfaConfigs[1].IsEnabled)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_GetByUserID_Empty(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	rows := sqlmock.NewRows([]string{"id", "user_id", "mfa_type", "is_enabled",
		"secret_key", "backup_codes", "last_used_at", "created_at", "updated_at"})

	mock.ExpectQuery("SELECT (.+) FROM user_mfa WHERE user_id = \\$1 ORDER BY created_at DESC").
		WithArgs("user-123").
		WillReturnRows(rows)

	mfaConfigs, err := repo.GetByUserID(context.Background(), "user-123")
	require.NoError(t, err)
	assert.Empty(t, mfaConfigs)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_GetByUserID_DatabaseError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	mock.ExpectQuery("SELECT (.+) FROM user_mfa WHERE user_id = \\$1 ORDER BY created_at DESC").
		WithArgs("user-123").
		WillReturnError(sql.ErrConnDone)

	mfaConfigs, err := repo.GetByUserID(context.Background(), "user-123")
	assert.Error(t, err)
	assert.Nil(t, mfaConfigs)
	assert.Contains(t, err.Error(), "failed to list MFA configurations")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_GetByUserID_ScanError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	rows := sqlmock.NewRows([]string{"id", "user_id", "mfa_type", "is_enabled",
		"secret_key", "backup_codes", "last_used_at", "created_at", "updated_at"}).
		AddRow("mfa-1", "user-123", "totp", true,
			"secret-1", "invalid-array-format", time.Now(), time.Now(), time.Now())

	mock.ExpectQuery("SELECT (.+) FROM user_mfa WHERE user_id = \\$1 ORDER BY created_at DESC").
		WithArgs("user-123").
		WillReturnRows(rows)

	mfaConfigs, err := repo.GetByUserID(context.Background(), "user-123")
	assert.Error(t, err)
	assert.Nil(t, mfaConfigs)
	assert.Contains(t, err.Error(), "failed to scan MFA configuration")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_UpdateLastUsed_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	mock.ExpectExec("UPDATE user_mfa SET last_used_at = NOW\\(\\), updated_at = NOW\\(\\) WHERE id = \\$1").
		WithArgs("mfa-123").
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.UpdateLastUsed(context.Background(), "mfa-123")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_UpdateLastUsed_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	mock.ExpectExec("UPDATE user_mfa SET last_used_at = NOW\\(\\), updated_at = NOW\\(\\) WHERE id = \\$1").
		WithArgs("mfa-123").
		WillReturnError(sql.ErrConnDone)

	err = repo.UpdateLastUsed(context.Background(), "mfa-123")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to update MFA last used")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_Delete_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	mock.ExpectExec("DELETE FROM user_mfa WHERE user_id = \\$1 AND mfa_type = \\$2").
		WithArgs("user-123", "totp").
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.Delete(context.Background(), "user-123", "totp")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_Delete_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	mock.ExpectExec("DELETE FROM user_mfa WHERE user_id = \\$1 AND mfa_type = \\$2").
		WithArgs("user-123", "nonexistent").
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.Delete(context.Background(), "user-123", "nonexistent")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "MFA configuration not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestMFARepository_Delete_DatabaseError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewMFARepository(db)

	mock.ExpectExec("DELETE FROM user_mfa WHERE user_id = \\$1 AND mfa_type = \\$2").
		WithArgs("user-123", "totp").
		WillReturnError(sql.ErrConnDone)

	err = repo.Delete(context.Background(), "user-123", "totp")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to delete MFA configuration")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}
