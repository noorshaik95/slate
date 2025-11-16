package database

import (
	"database/sql"
	"os"
	"slate/services/user-auth-service/pkg/logger"
	"strconv"
	"time"

	_ "github.com/lib/pq"
)

type DB struct {
	*sql.DB
	log *logger.Logger
}

func NewPostgresDB(dsn string) (*DB, error) {
	log := logger.NewLogger("info")

	db, err := sql.Open("postgres", dsn)
	if err != nil {
		log.Error().Err(err).Msg("Failed to open database connection")
		return nil, err
	}

	// Test connection
	if err := db.Ping(); err != nil {
		log.Error().Err(err).Msg("Failed to ping database")
		return nil, err
	}

	// Configure connection pool from environment variables
	maxOpenConns := getEnvInt("DB_MAX_OPEN_CONNS", 25, log)
	maxIdleConns := getEnvInt("DB_MAX_IDLE_CONNS", 5, log)
	connMaxLifetime := getEnvDuration("DB_CONN_MAX_LIFETIME", 5*time.Minute, log)
	connMaxIdleTime := getEnvDuration("DB_CONN_MAX_IDLE_TIME", 1*time.Minute, log)

	db.SetMaxOpenConns(maxOpenConns)
	db.SetMaxIdleConns(maxIdleConns)
	db.SetConnMaxLifetime(connMaxLifetime)
	db.SetConnMaxIdleTime(connMaxIdleTime)

	log.Info().
		Int("max_open_conns", maxOpenConns).
		Int("max_idle_conns", maxIdleConns).
		Dur("conn_max_lifetime", connMaxLifetime).
		Dur("conn_max_idle_time", connMaxIdleTime).
		Msg("Database connection pool configured")

	log.Info().Msg("Successfully connected to PostgreSQL database")

	return &DB{DB: db, log: log}, nil
}

func (db *DB) Close() error {
	return db.DB.Close()
}

// getEnvInt reads an integer from environment variable with a default value
func getEnvInt(key string, defaultValue int, log *logger.Logger) int {
	if value := os.Getenv(key); value != "" {
		if intValue, err := strconv.Atoi(value); err == nil {
			return intValue
		}
		log.Warn().
			Str("key", key).
			Int("default_value", defaultValue).
			Str("invalid_value", value).
			Msg("Invalid environment variable value, using default")
	}
	return defaultValue
}

// getEnvDuration reads a duration from environment variable with a default value
// Accepts values like "5m", "30s", "1h", etc.
func getEnvDuration(key string, defaultValue time.Duration, log *logger.Logger) time.Duration {
	if value := os.Getenv(key); value != "" {
		if duration, err := time.ParseDuration(value); err == nil {
			return duration
		}
		log.Warn().
			Str("key", key).
			Dur("default_value", defaultValue).
			Str("invalid_value", value).
			Msg("Invalid duration value, using default")
	}
	return defaultValue
}
