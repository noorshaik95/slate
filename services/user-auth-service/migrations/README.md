# Database Migrations

This directory contains database migrations for the user-auth-service. Migrations are executed in order and tracked in the `schema_migrations` table.

## Migration Files

Migrations are numbered sequentially and follow the naming convention: `NNN_description.sql` for up migrations and `NNN_description.down.sql` for rollback migrations.

### Current Migrations

1. **001_init.sql** - Initial database schema
   - Creates `users`, `roles`, and `user_roles` tables
   - Sets up basic indexes and constraints
   - Inserts default roles (admin, user, moderator)
   - Creates default admin user

2. **002_enhanced_features.sql** - Enhanced features
   - Adds `oauth_providers` table for OAuth authentication
   - Adds `saml_configs` and `saml_sessions` tables for SAML authentication
   - Adds `user_mfa` table for multi-factor authentication
   - Adds `user_groups` and `group_members` tables for group management
   - Adds `parent_child_accounts` table for account relationships
   - Adds additional user profile fields (timezone, avatar_url, bio)

3. **003_add_auth_method_to_users.sql** - Authentication method tracking
   - Adds `auth_method` column to users table (values: 'normal', 'oauth', 'saml')
   - Adds `organization_id` column to users table
   - Creates indexes for efficient querying
   - Adds CHECK constraint to validate auth_method values
   - **Dependencies**: Requires 001_init.sql

4. **004_add_provider_type_to_oauth.sql** - OAuth provider type
   - Adds `provider_type` column to oauth_providers table
   - Distinguishes between google, microsoft, and custom OAuth providers
   - Updates existing records based on provider column
   - Creates index on provider_type
   - **Dependencies**: Requires 002_enhanced_features.sql

5. **005_enhance_saml_configs.sql** - Enhanced SAML configuration
   - Adds `provider_type` column to saml_configs table
   - Adds `metadata_url` for automatic metadata refresh
   - Adds `jit_provisioning` flag for just-in-time user provisioning
   - Adds `group_sync` flag for group synchronization
   - Adds `group_attribute` for SAML group attribute name
   - Adds `attribute_mapping` JSONB column for custom attribute mapping
   - Adds `config_key` for environment-based configuration lookup
   - Creates indexes for efficient querying
   - **Dependencies**: Requires 002_enhanced_features.sql

6. **006_create_saml_metadata_cache.sql** - SAML metadata caching
   - Creates `saml_metadata_cache` table for caching IdP metadata
   - Stores metadata XML with expiration tracking
   - Links to SAML configs via config_key
   - Creates indexes for efficient lookups and cleanup
   - **Dependencies**: Requires 005_enhance_saml_configs.sql

## Running Migrations

### Prerequisites

- PostgreSQL 12 or higher
- Go 1.21 or higher
- Database connection credentials

### Environment Variables

Set the following environment variables before running migrations:

```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=postgres
export DB_PASSWORD=postgres
export DB_NAME=userauth
export DB_SSLMODE=disable
```

### Apply All Pending Migrations

To apply all pending migrations to your database:

```bash
cd services/user-auth-service/migrations
go run test_migrations.go up
```

This will:
1. Create the `schema_migrations` table if it doesn't exist
2. Check which migrations have already been applied
3. Apply pending migrations in order
4. Execute each migration within a transaction
5. Rollback automatically if any migration fails

### Check Migration Status

To see which migrations have been applied:

```bash
cd services/user-auth-service/migrations
go run test_migrations.go status
```

This displays:
- List of applied migrations
- Timestamp when each migration was applied
- Total count of applied migrations

### Rollback a Migration

To rollback a specific migration:

```bash
cd services/user-auth-service/migrations
go run test_migrations.go down <migration_version>
```

Example:
```bash
go run test_migrations.go down 006_create_saml_metadata_cache
```

This will:
1. Check if the migration is currently applied
2. Execute the corresponding `.down.sql` file
3. Remove the migration record from `schema_migrations`
4. Execute within a transaction with automatic rollback on error

**Note**: Rollback migrations must be executed in reverse order of dependencies.

## Using the Migration Runner in Code

You can also use the migration runner programmatically:

```go
import (
    "database/sql"
    "services/user-auth-service/migrations"
    _ "github.com/lib/pq"
)

func main() {
    db, err := sql.Open("postgres", connStr)
    if err != nil {
        log.Fatal(err)
    }
    defer db.Close()

    // Run all pending migrations
    if err := migrations.RunMigrations(db, "./migrations"); err != nil {
        log.Fatalf("Migration failed: %v", err)
    }

    // Rollback a specific migration
    if err := migrations.RollbackMigration(db, "./migrations", "006_create_saml_metadata_cache"); err != nil {
        log.Fatalf("Rollback failed: %v", err)
    }

    // Get list of applied migrations
    applied, err := migrations.GetAppliedMigrations(db)
    if err != nil {
        log.Fatalf("Failed to get migrations: %v", err)
    }
    for _, version := range applied {
        log.Printf("Applied: %s", version)
    }
}
```

## Creating New Migrations

### Naming Convention

Migration files should follow this pattern:
- Up migration: `NNN_description.sql`
- Down migration: `NNN_description.down.sql`

Where:
- `NNN` is a zero-padded sequential number (e.g., 007, 008, 009)
- `description` is a brief, lowercase, underscore-separated description

### Migration File Structure

**Up Migration (NNN_description.sql)**:
```sql
-- Brief description of what this migration does

-- Add your DDL statements here
ALTER TABLE users ADD COLUMN new_field VARCHAR(255);

-- Create indexes
CREATE INDEX idx_users_new_field ON users(new_field);

-- Update existing data if needed
UPDATE users SET new_field = 'default_value' WHERE new_field IS NULL;
```

**Down Migration (NNN_description.down.sql)**:
```sql
-- Rollback for NNN_description

-- Remove what was added in the up migration
DROP INDEX IF EXISTS idx_users_new_field;
ALTER TABLE users DROP COLUMN IF EXISTS new_field;
```

### Best Practices

1. **Always create both up and down migrations** - This allows for safe rollbacks
2. **Keep migrations small and focused** - One logical change per migration
3. **Test migrations on a copy of production data** - Ensure they work with real data
4. **Use transactions** - The migration runner automatically wraps each migration in a transaction
5. **Make migrations idempotent when possible** - Use `IF NOT EXISTS` and `IF EXISTS` clauses
6. **Document dependencies** - Add comments noting which migrations must be applied first
7. **Never modify applied migrations** - Create a new migration to make changes
8. **Test rollbacks** - Ensure down migrations actually reverse the up migration

### Example: Adding a New Column

**007_add_user_preferences.sql**:
```sql
-- Add user preferences column for storing JSON preferences

ALTER TABLE users ADD COLUMN IF NOT EXISTS preferences JSONB DEFAULT '{}';

-- Create GIN index for efficient JSONB queries
CREATE INDEX IF NOT EXISTS idx_users_preferences ON users USING GIN (preferences);

-- Add comment
COMMENT ON COLUMN users.preferences IS 'User preferences stored as JSON';
```

**007_add_user_preferences.down.sql**:
```sql
-- Rollback user preferences column

DROP INDEX IF EXISTS idx_users_preferences;
ALTER TABLE users DROP COLUMN IF EXISTS preferences;
```

## Migration Execution Flow

```
┌─────────────────────────────────────────┐
│  Start Migration Process                │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Create schema_migrations table         │
│  (if not exists)                        │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Read all .sql files from directory     │
│  (exclude .down.sql files)              │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  Sort migration files alphabetically    │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│  For each migration file:               │
│  ┌───────────────────────────────────┐  │
│  │ Check if already applied          │  │
│  └────────┬──────────────────────────┘  │
│           │                              │
│           ▼                              │
│  ┌───────────────────────────────────┐  │
│  │ If applied: Skip                  │  │
│  │ If not: Apply in transaction      │  │
│  └────────┬──────────────────────────┘  │
│           │                              │
│           ▼                              │
│  ┌───────────────────────────────────┐  │
│  │ BEGIN TRANSACTION                 │  │
│  │ Execute SQL                       │  │
│  │ Record in schema_migrations       │  │
│  │ COMMIT                            │  │
│  └────────┬──────────────────────────┘  │
│           │                              │
│           ▼                              │
│  ┌───────────────────────────────────┐  │
│  │ On Error: ROLLBACK                │  │
│  │ Log error and stop                │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## Troubleshooting

### Migration Fails with "already exists" Error

**Problem**: Migration tries to create a table/column that already exists.

**Solution**: 
- Use `IF NOT EXISTS` clause in CREATE statements
- Use `IF EXISTS` clause in DROP statements
- Check if the migration was partially applied

### Migration Fails Midway

**Problem**: Migration fails after some statements executed.

**Solution**:
- The transaction will automatically rollback
- Fix the migration SQL
- Re-run the migration
- If needed, manually clean up any objects created outside transactions (e.g., databases)

### Cannot Rollback Migration

**Problem**: Down migration file is missing or fails.

**Solution**:
- Ensure `.down.sql` file exists for the migration
- Verify the down migration correctly reverses the up migration
- Check for dependencies (e.g., foreign keys preventing drops)

### Migration Order Issues

**Problem**: Migration fails due to missing dependencies.

**Solution**:
- Check migration dependencies in comments
- Ensure migrations are numbered correctly
- Apply migrations in the correct order
- Create a new migration to fix the issue rather than modifying existing ones

### Database Connection Issues

**Problem**: Cannot connect to database.

**Solution**:
- Verify database is running: `docker ps | grep postgres`
- Check environment variables are set correctly
- Test connection: `psql -h $DB_HOST -U $DB_USER -d $DB_NAME`
- Ensure firewall allows connection on port 5432

### Schema Drift

**Problem**: Production database schema differs from migrations.

**Solution**:
- Never modify the database schema manually
- Always use migrations for schema changes
- If manual changes were made, create a migration to align the schema
- Use `go run test_migrations.go status` to check applied migrations

## Testing Migrations

### Test on Clean Database

```bash
# Create test database
docker exec slate-postgres-1 psql -U postgres -c "CREATE DATABASE userauth_test;"

# Run migrations
DB_NAME=userauth_test go run test_migrations.go up

# Verify schema
docker exec slate-postgres-1 psql -U postgres -d userauth_test -c "\dt"
docker exec slate-postgres-1 psql -U postgres -d userauth_test -c "\d users"

# Clean up
docker exec slate-postgres-1 psql -U postgres -c "DROP DATABASE userauth_test;"
```

### Test on Database with Existing Data

```bash
# Create test database with existing schema
docker exec slate-postgres-1 psql -U postgres -c "CREATE DATABASE userauth_existing;"

# Apply base migrations
cat 001_init.sql | docker exec -i slate-postgres-1 psql -U postgres -d userauth_existing
cat 002_enhanced_features.sql | docker exec -i slate-postgres-1 psql -U postgres -d userauth_existing

# Insert test data
docker exec slate-postgres-1 psql -U postgres -d userauth_existing -c "INSERT INTO users (id, email, password_hash, first_name, last_name) VALUES ('test-1', 'test@example.com', 'hash', 'Test', 'User');"

# Run new migrations
DB_NAME=userauth_existing go run test_migrations.go up

# Verify data preserved and new columns have defaults
docker exec slate-postgres-1 psql -U postgres -d userauth_existing -c "SELECT * FROM users;"

# Clean up
docker exec slate-postgres-1 psql -U postgres -c "DROP DATABASE userauth_existing;"
```

### Test Rollback

```bash
# Apply all migrations
DB_NAME=userauth_test go run test_migrations.go up

# Rollback last migration
DB_NAME=userauth_test go run test_migrations.go down 006_create_saml_metadata_cache

# Verify table was dropped
docker exec slate-postgres-1 psql -U postgres -d userauth_test -c "\dt"

# Re-apply migration
DB_NAME=userauth_test go run test_migrations.go up

# Verify table was recreated
docker exec slate-postgres-1 psql -U postgres -d userauth_test -c "\d saml_metadata_cache"
```

## Production Deployment

### Pre-Deployment Checklist

- [ ] All migrations tested on staging database
- [ ] Rollback migrations tested
- [ ] Database backup created
- [ ] Migration execution time estimated
- [ ] Downtime window scheduled (if needed)
- [ ] Rollback plan documented

### Deployment Steps

1. **Backup Database**
   ```bash
   pg_dump -h $DB_HOST -U $DB_USER -d $DB_NAME > backup_$(date +%Y%m%d_%H%M%S).sql
   ```

2. **Apply Migrations**
   ```bash
   cd services/user-auth-service/migrations
   DB_HOST=prod-db.example.com DB_NAME=userauth_prod go run test_migrations.go up
   ```

3. **Verify Success**
   ```bash
   DB_HOST=prod-db.example.com DB_NAME=userauth_prod go run test_migrations.go status
   ```

4. **If Rollback Needed**
   ```bash
   # Rollback migrations in reverse order
   DB_HOST=prod-db.example.com DB_NAME=userauth_prod go run test_migrations.go down 006_create_saml_metadata_cache
   DB_HOST=prod-db.example.com DB_NAME=userauth_prod go run test_migrations.go down 005_enhance_saml_configs
   # ... continue as needed
   ```

### Zero-Downtime Migrations

For large tables or production systems requiring zero downtime:

1. **Add new columns as nullable first**
2. **Deploy application code that writes to both old and new columns**
3. **Backfill data in batches**
4. **Deploy application code that reads from new columns**
5. **Remove old columns in a later migration**

Example:
```sql
-- Migration 1: Add new column (nullable)
ALTER TABLE users ADD COLUMN new_email VARCHAR(255);
CREATE INDEX CONCURRENTLY idx_users_new_email ON users(new_email);

-- Migration 2 (after backfill): Make column NOT NULL
ALTER TABLE users ALTER COLUMN new_email SET NOT NULL;

-- Migration 3 (after code deployment): Remove old column
ALTER TABLE users DROP COLUMN email;
ALTER TABLE users RENAME COLUMN new_email TO email;
```

## Support

For issues or questions about migrations:
1. Check this README for common solutions
2. Review migration comments for dependencies
3. Test on a local database first
4. Contact the development team for assistance
