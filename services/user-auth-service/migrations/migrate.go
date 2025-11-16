package migrations

import (
	"database/sql"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// RunMigrations executes all pending migrations in order with transaction support
func RunMigrations(db *sql.DB, migrationsPath string) error {
	log.Println("Starting migration process...")

	// Create migrations table if it doesn't exist
	log.Println("Creating schema_migrations table if not exists...")
	_, err := db.Exec(`
		CREATE TABLE IF NOT EXISTS schema_migrations (
			version VARCHAR(255) PRIMARY KEY,
			applied_at TIMESTAMP NOT NULL DEFAULT NOW()
		)
	`)
	if err != nil {
		return fmt.Errorf("failed to create migrations table: %w", err)
	}

	// Read migration files
	log.Printf("Reading migration files from: %s\n", migrationsPath)
	files, err := os.ReadDir(migrationsPath)
	if err != nil {
		return fmt.Errorf("failed to read migrations directory: %w", err)
	}

	// Sort migration files (only .sql files, exclude .down.sql)
	var migrationFiles []string
	for _, file := range files {
		if !file.IsDir() && strings.HasSuffix(file.Name(), ".sql") && !strings.HasSuffix(file.Name(), ".down.sql") {
			migrationFiles = append(migrationFiles, file.Name())
		}
	}
	sort.Strings(migrationFiles)

	log.Printf("Found %d migration files to process\n", len(migrationFiles))

	// Apply migrations
	for _, filename := range migrationFiles {
		version := strings.TrimSuffix(filename, ".sql")

		// Check if migration already applied
		var applied bool
		err := db.QueryRow("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = $1)", version).Scan(&applied)
		if err != nil {
			return fmt.Errorf("failed to check migration status: %w", err)
		}

		if applied {
			log.Printf("✓ Migration %s already applied, skipping\n", version)
			continue
		}

		// Execute migration in transaction
		log.Printf("→ Applying migration: %s\n", version)
		if err := applyMigration(db, migrationsPath, filename, version); err != nil {
			log.Printf("✗ Failed to apply migration %s: %v\n", version, err)
			return err
		}

		log.Printf("✓ Successfully applied migration: %s\n", version)
	}

	log.Println("Migration process completed successfully")
	return nil
}

// applyMigration executes a single migration within a transaction
func applyMigration(db *sql.DB, migrationsPath, filename, version string) error {
	// Start transaction
	tx, err := db.Begin()
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	// Ensure rollback on error
	defer func() {
		if err != nil {
			log.Printf("Rolling back migration %s due to error\n", version)
			if rbErr := tx.Rollback(); rbErr != nil {
				log.Printf("Failed to rollback transaction: %v\n", rbErr)
			}
		}
	}()

	// Read migration file
	content, err := os.ReadFile(filepath.Join(migrationsPath, filename))
	if err != nil {
		return fmt.Errorf("failed to read migration file %s: %w", filename, err)
	}

	// Execute migration
	_, err = tx.Exec(string(content))
	if err != nil {
		return fmt.Errorf("failed to execute migration %s: %w", filename, err)
	}

	// Record migration
	_, err = tx.Exec("INSERT INTO schema_migrations (version) VALUES ($1)", version)
	if err != nil {
		return fmt.Errorf("failed to record migration %s: %w", filename, err)
	}

	// Commit transaction
	if err = tx.Commit(); err != nil {
		return fmt.Errorf("failed to commit migration %s: %w", filename, err)
	}

	return nil
}

// RollbackMigration rolls back a specific migration
func RollbackMigration(db *sql.DB, migrationsPath, version string) error {
	log.Printf("Rolling back migration: %s\n", version)

	// Check if migration is applied
	var applied bool
	err := db.QueryRow("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = $1)", version).Scan(&applied)
	if err != nil {
		return fmt.Errorf("failed to check migration status: %w", err)
	}

	if !applied {
		log.Printf("Migration %s is not applied, nothing to rollback\n", version)
		return nil
	}

	// Find down migration file
	downFile := version + ".down.sql"
	downPath := filepath.Join(migrationsPath, downFile)

	// Check if down migration exists
	if _, err := os.Stat(downPath); os.IsNotExist(err) {
		return fmt.Errorf("down migration file not found: %s", downFile)
	}

	// Start transaction
	tx, err := db.Begin()
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	// Ensure rollback on error
	defer func() {
		if err != nil {
			log.Printf("Rolling back transaction due to error\n")
			if rbErr := tx.Rollback(); rbErr != nil {
				log.Printf("Failed to rollback transaction: %v\n", rbErr)
			}
		}
	}()

	// Read down migration file
	content, err := os.ReadFile(downPath)
	if err != nil {
		return fmt.Errorf("failed to read down migration file %s: %w", downFile, err)
	}

	// Execute down migration
	_, err = tx.Exec(string(content))
	if err != nil {
		return fmt.Errorf("failed to execute down migration %s: %w", downFile, err)
	}

	// Remove migration record
	_, err = tx.Exec("DELETE FROM schema_migrations WHERE version = $1", version)
	if err != nil {
		return fmt.Errorf("failed to remove migration record %s: %w", version, err)
	}

	// Commit transaction
	if err = tx.Commit(); err != nil {
		return fmt.Errorf("failed to commit rollback %s: %w", version, err)
	}

	log.Printf("✓ Successfully rolled back migration: %s\n", version)
	return nil
}

// GetAppliedMigrations returns a list of applied migrations
func GetAppliedMigrations(db *sql.DB) ([]string, error) {
	rows, err := db.Query("SELECT version FROM schema_migrations ORDER BY version")
	if err != nil {
		return nil, fmt.Errorf("failed to query applied migrations: %w", err)
	}
	defer rows.Close()

	var migrations []string
	for rows.Next() {
		var version string
		if err := rows.Scan(&version); err != nil {
			return nil, fmt.Errorf("failed to scan migration version: %w", err)
		}
		migrations = append(migrations, version)
	}

	return migrations, nil
}
