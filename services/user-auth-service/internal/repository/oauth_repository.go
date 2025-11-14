package repository

import (
	"context"
	"database/sql"
	"fmt"

	"slate/services/user-auth-service/internal/models"
)

type OAuthRepository struct {
	db *sql.DB
}

func NewOAuthRepository(db *sql.DB) *OAuthRepository {
	return &OAuthRepository{db: db}
}

// CreateOrUpdate creates or updates an OAuth provider for a user
func (r *OAuthRepository) CreateOrUpdate(ctx context.Context, provider *models.OAuthProvider) error {
	query := `
		INSERT INTO oauth_providers (id, user_id, provider, provider_user_id, access_token, refresh_token, token_expiry, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
		ON CONFLICT (provider, provider_user_id)
		DO UPDATE SET
			user_id = EXCLUDED.user_id,
			access_token = EXCLUDED.access_token,
			refresh_token = EXCLUDED.refresh_token,
			token_expiry = EXCLUDED.token_expiry,
			updated_at = EXCLUDED.updated_at
	`
	_, err := r.db.ExecContext(ctx, query, provider.ID, provider.UserID, provider.Provider,
		provider.ProviderUserID, provider.AccessToken, provider.RefreshToken,
		provider.TokenExpiry, provider.CreatedAt, provider.UpdatedAt)

	if err != nil {
		return fmt.Errorf("failed to create/update OAuth provider: %w", err)
	}
	return nil
}

// GetByProviderAndUserID retrieves an OAuth provider by provider name and provider user ID
func (r *OAuthRepository) GetByProviderAndUserID(ctx context.Context, provider, providerUserID string) (*models.OAuthProvider, error) {
	oauthProvider := &models.OAuthProvider{}
	query := `
		SELECT id, user_id, provider, provider_user_id, access_token, refresh_token, token_expiry, created_at, updated_at
		FROM oauth_providers
		WHERE provider = $1 AND provider_user_id = $2
	`

	err := r.db.QueryRowContext(ctx, query, provider, providerUserID).Scan(
		&oauthProvider.ID, &oauthProvider.UserID, &oauthProvider.Provider,
		&oauthProvider.ProviderUserID, &oauthProvider.AccessToken, &oauthProvider.RefreshToken,
		&oauthProvider.TokenExpiry, &oauthProvider.CreatedAt, &oauthProvider.UpdatedAt,
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("OAuth provider not found")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get OAuth provider: %w", err)
	}

	return oauthProvider, nil
}

// GetByUserID retrieves all OAuth providers for a user
func (r *OAuthRepository) GetByUserID(ctx context.Context, userID string) ([]*models.OAuthProvider, error) {
	query := `
		SELECT id, user_id, provider, provider_user_id, access_token, refresh_token, token_expiry, created_at, updated_at
		FROM oauth_providers
		WHERE user_id = $1
		ORDER BY created_at DESC
	`

	rows, err := r.db.QueryContext(ctx, query, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to list OAuth providers: %w", err)
	}
	defer rows.Close()

	providers := []*models.OAuthProvider{}
	for rows.Next() {
		provider := &models.OAuthProvider{}
		err := rows.Scan(
			&provider.ID, &provider.UserID, &provider.Provider,
			&provider.ProviderUserID, &provider.AccessToken, &provider.RefreshToken,
			&provider.TokenExpiry, &provider.CreatedAt, &provider.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan OAuth provider: %w", err)
		}
		providers = append(providers, provider)
	}

	return providers, nil
}

// Delete removes an OAuth provider
func (r *OAuthRepository) Delete(ctx context.Context, id string) error {
	query := `DELETE FROM oauth_providers WHERE id = $1`
	result, err := r.db.ExecContext(ctx, query, id)

	if err != nil {
		return fmt.Errorf("failed to delete OAuth provider: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rows == 0 {
		return fmt.Errorf("OAuth provider not found")
	}

	return nil
}
