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

func TestOAuthRepository_CreateOrUpdate_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)
	now := time.Now()

	provider := &models.OAuthProvider{
		ID:             "oauth-123",
		UserID:         "user-123",
		Provider:       "google",
		ProviderUserID: "google-user-123",
		AccessToken:    "access-token",
		RefreshToken:   "refresh-token",
		TokenExpiry:    now.Add(1 * time.Hour),
		CreatedAt:      now,
		UpdatedAt:      now,
	}

	mock.ExpectExec("INSERT INTO oauth_providers").
		WithArgs(provider.ID, provider.UserID, provider.Provider, provider.ProviderUserID,
			provider.AccessToken, provider.RefreshToken, provider.TokenExpiry,
			provider.CreatedAt, provider.UpdatedAt).
		WillReturnResult(sqlmock.NewResult(1, 1))

	err = repo.CreateOrUpdate(context.Background(), provider)
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_CreateOrUpdate_Error(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)
	now := time.Now()

	provider := &models.OAuthProvider{
		ID:             "oauth-123",
		UserID:         "user-123",
		Provider:       "google",
		ProviderUserID: "google-user-123",
		AccessToken:    "access-token",
		RefreshToken:   "refresh-token",
		TokenExpiry:    now.Add(1 * time.Hour),
		CreatedAt:      now,
		UpdatedAt:      now,
	}

	mock.ExpectExec("INSERT INTO oauth_providers").
		WithArgs(provider.ID, provider.UserID, provider.Provider, provider.ProviderUserID,
			provider.AccessToken, provider.RefreshToken, provider.TokenExpiry,
			provider.CreatedAt, provider.UpdatedAt).
		WillReturnError(sql.ErrConnDone)

	err = repo.CreateOrUpdate(context.Background(), provider)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to create/update OAuth provider")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_GetByProviderAndUserID_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)
	now := time.Now()

	expectedProvider := &models.OAuthProvider{
		ID:             "oauth-123",
		UserID:         "user-123",
		Provider:       "google",
		ProviderUserID: "google-user-123",
		AccessToken:    "access-token",
		RefreshToken:   "refresh-token",
		TokenExpiry:    now.Add(1 * time.Hour),
		CreatedAt:      now,
		UpdatedAt:      now,
	}

	rows := sqlmock.NewRows([]string{"id", "user_id", "provider", "provider_user_id",
		"access_token", "refresh_token", "token_expiry", "created_at", "updated_at"}).
		AddRow(expectedProvider.ID, expectedProvider.UserID, expectedProvider.Provider,
			expectedProvider.ProviderUserID, expectedProvider.AccessToken,
			expectedProvider.RefreshToken, expectedProvider.TokenExpiry,
			expectedProvider.CreatedAt, expectedProvider.UpdatedAt)

	mock.ExpectQuery("SELECT (.+) FROM oauth_providers WHERE provider = \\$1 AND provider_user_id = \\$2").
		WithArgs("google", "google-user-123").
		WillReturnRows(rows)

	provider, err := repo.GetByProviderAndUserID(context.Background(), "google", "google-user-123")
	require.NoError(t, err)
	assert.NotNil(t, provider)
	assert.Equal(t, expectedProvider.ID, provider.ID)
	assert.Equal(t, expectedProvider.UserID, provider.UserID)
	assert.Equal(t, expectedProvider.Provider, provider.Provider)
	assert.Equal(t, expectedProvider.ProviderUserID, provider.ProviderUserID)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_GetByProviderAndUserID_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM oauth_providers WHERE provider = \\$1 AND provider_user_id = \\$2").
		WithArgs("google", "nonexistent").
		WillReturnError(sql.ErrNoRows)

	provider, err := repo.GetByProviderAndUserID(context.Background(), "google", "nonexistent")
	assert.Error(t, err)
	assert.Nil(t, provider)
	assert.Contains(t, err.Error(), "OAuth provider not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_GetByProviderAndUserID_DatabaseError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM oauth_providers WHERE provider = \\$1 AND provider_user_id = \\$2").
		WithArgs("google", "google-user-123").
		WillReturnError(sql.ErrConnDone)

	provider, err := repo.GetByProviderAndUserID(context.Background(), "google", "google-user-123")
	assert.Error(t, err)
	assert.Nil(t, provider)
	assert.Contains(t, err.Error(), "failed to get OAuth provider")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_GetByUserID_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)
	now := time.Now()

	rows := sqlmock.NewRows([]string{"id", "user_id", "provider", "provider_user_id",
		"access_token", "refresh_token", "token_expiry", "created_at", "updated_at"}).
		AddRow("oauth-1", "user-123", "google", "google-user-123",
			"access-token-1", "refresh-token-1",
			now.Add(1*time.Hour), now, now).
		AddRow("oauth-2", "user-123", "github", "github-user-123",
			"access-token-2", "refresh-token-2",
			now.Add(1*time.Hour), now, now)

	mock.ExpectQuery("SELECT (.+) FROM oauth_providers WHERE user_id = \\$1 ORDER BY created_at DESC").
		WithArgs("user-123").
		WillReturnRows(rows)

	providers, err := repo.GetByUserID(context.Background(), "user-123")
	require.NoError(t, err)
	assert.Len(t, providers, 2)
	assert.Equal(t, "google", providers[0].Provider)
	assert.Equal(t, "github", providers[1].Provider)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_GetByUserID_Empty(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)

	rows := sqlmock.NewRows([]string{"id", "user_id", "provider", "provider_user_id",
		"access_token", "refresh_token", "token_expiry", "created_at", "updated_at"})

	mock.ExpectQuery("SELECT (.+) FROM oauth_providers WHERE user_id = \\$1 ORDER BY created_at DESC").
		WithArgs("user-123").
		WillReturnRows(rows)

	providers, err := repo.GetByUserID(context.Background(), "user-123")
	require.NoError(t, err)
	assert.Empty(t, providers)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_GetByUserID_DatabaseError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)

	mock.ExpectQuery("SELECT (.+) FROM oauth_providers WHERE user_id = \\$1 ORDER BY created_at DESC").
		WithArgs("user-123").
		WillReturnError(sql.ErrConnDone)

	providers, err := repo.GetByUserID(context.Background(), "user-123")
	assert.Error(t, err)
	assert.Nil(t, providers)
	assert.Contains(t, err.Error(), "failed to list OAuth providers")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_GetByUserID_ScanError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)

	rows := sqlmock.NewRows([]string{"id", "user_id", "provider", "provider_user_id",
		"access_token", "refresh_token", "token_expiry", "created_at", "updated_at"}).
		AddRow("oauth-1", "user-123", "google", "google-user-123",
			"access-token-1", "refresh-token-1",
			"invalid-time", time.Now(), time.Now()) // Invalid time format

	mock.ExpectQuery("SELECT (.+) FROM oauth_providers WHERE user_id = \\$1 ORDER BY created_at DESC").
		WithArgs("user-123").
		WillReturnRows(rows)

	providers, err := repo.GetByUserID(context.Background(), "user-123")
	assert.Error(t, err)
	assert.Nil(t, providers)
	assert.Contains(t, err.Error(), "failed to scan OAuth provider")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_Delete_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)

	mock.ExpectExec("DELETE FROM oauth_providers WHERE id = \\$1").
		WithArgs("oauth-123").
		WillReturnResult(sqlmock.NewResult(0, 1))

	err = repo.Delete(context.Background(), "oauth-123")
	require.NoError(t, err)

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_Delete_NotFound(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)

	mock.ExpectExec("DELETE FROM oauth_providers WHERE id = \\$1").
		WithArgs("nonexistent").
		WillReturnResult(sqlmock.NewResult(0, 0))

	err = repo.Delete(context.Background(), "nonexistent")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "OAuth provider not found")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}

func TestOAuthRepository_Delete_DatabaseError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	repo := NewOAuthRepository(db)

	mock.ExpectExec("DELETE FROM oauth_providers WHERE id = \\$1").
		WithArgs("oauth-123").
		WillReturnError(sql.ErrConnDone)

	err = repo.Delete(context.Background(), "oauth-123")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to delete OAuth provider")

	err = mock.ExpectationsWereMet()
	assert.NoError(t, err)
}
