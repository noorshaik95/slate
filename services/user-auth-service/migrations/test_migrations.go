//go:build ignore
// +build ignore

package main

import (
	"database/sql"
	"fmt"
	"log"
	"os"

	_ "github.com/lib/pq"
)

func main() {
	if len(os.Args) < 2 {
		log.Fatal("Usage: go run test_migrations.go <command>\nCommands: up, down <version>, status")
	}

	command := os.Args[1]

	// Database connection
	dbHost := getEnv("DB_HOST", "localhost")
	dbPort := getEnv("DB_PORT", "5432")
	dbUser := getEnv("DB_USER", "postgres")
	dbPassword := getEnv("DB_PASSWORD", "postgres")
	dbName := getEnv("DB_NAME", "userauth_test")
	sslMode := getEnv("DB_SSLMODE", "disable")

	connStr := fmt.Sprintf("host=%s port=%s user=%s password=%s dbname=%s sslmode=%s",
		dbHost, dbPort, dbUser, dbPassword, dbName, sslMode)

	log.Printf("Connecting to database: %s@%s:%s/%s\n", dbUser, dbHost, dbPort, dbName)

	db, err := sql.Open("postgres", connStr)
	if err != nil {
		log.Fatalf("Failed to connect to database: %v", err)
	}
	defer db.Close()

	if err := db.Ping(); err != nil {
		log.Fatalf("Failed to ping database: %v", err)
	}

	log.Println("Database connection successful")

	migrationsPath := "."

	switch command {
	case "up":
		if err := runMigrations(db, migrationsPath); err != nil {
			log.Fatalf("Migration failed: %v", err)
		}
		log.Println("All migrations completed successfully")

	case "down":
		if len(os.Args) < 3 {
			log.Fatal("Usage: go run test_migrations.go down <version>")
		}
		version := os.Args[2]
		if err := rollbackMigration(db, migrationsPath, version); err != nil {
			log.Fatalf("Rollback failed: %v", err)
		}

	case "status":
		if err := showStatus(db); err != nil {
			log.Fatalf("Failed to show status: %v", err)
		}

	default:
		log.Fatalf("Unknown command: %s", command)
	}
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

func runMigrations(db *sql.DB, migrationsPath string) error {
	// Import the migrations package functions
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

	// Get list of migration files
	files, err := os.ReadDir(migrationsPath)
	if err != nil {
		return fmt.Errorf("failed to read migrations directory: %w", err)
	}

	var migrationFiles []string
	for _, file := range files {
		name := file.Name()
		// Only include .sql files, exclude .down.sql files
		if !file.IsDir() && len(name) > 4 && name[len(name)-4:] == ".sql" {
			if len(name) < 9 || name[len(name)-9:] != ".down.sql" {
				migrationFiles = append(migrationFiles, name)
			}
		}
	}

	// Sort migrations
	for i := 0; i < len(migrationFiles); i++ {
		for j := i + 1; j < len(migrationFiles); j++ {
			if migrationFiles[i] > migrationFiles[j] {
				migrationFiles[i], migrationFiles[j] = migrationFiles[j], migrationFiles[i]
			}
		}
	}

	log.Printf("Found %d migration files to process\n", len(migrationFiles))

	// Apply migrations
	for _, filename := range migrationFiles {
		version := filename[:len(filename)-4] // Remove .sql

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
	content, err := os.ReadFile(filename)
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

func rollbackMigration(db *sql.DB, migrationsPath, version string) error {
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

	// Check if down migration exists
	if _, err := os.Stat(downFile); os.IsNotExist(err) {
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
	content, err := os.ReadFile(downFile)
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

func showStatus(db *sql.DB) error {
	log.Println("Migration Status:")
	log.Println("================")

	rows, err := db.Query("SELECT version, applied_at FROM schema_migrations ORDER BY version")
	if err != nil {
		return fmt.Errorf("failed to query applied migrations: %w", err)
	}
	defer rows.Close()

	count := 0
	for rows.Next() {
		var version string
		var appliedAt string
		if err := rows.Scan(&version, &appliedAt); err != nil {
			return fmt.Errorf("failed to scan migration version: %w", err)
		}
		log.Printf("✓ %s (applied at: %s)\n", version, appliedAt)
		count++
	}

	log.Printf("\nTotal applied migrations: %d\n", count)
	return nil
}
