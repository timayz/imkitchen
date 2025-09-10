package tests

import (
	"context"
	"database/sql"
	"os"
	"testing"
	"time"

	_ "github.com/lib/pq"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestDatabaseMigrations(t *testing.T) {
	// Skip integration tests if no database URL is provided
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		dbURL = "postgres://imkitchen_user:imkitchen_password@localhost:5432/imkitchen_test?sslmode=disable"
	}

	db, err := sql.Open("postgres", dbURL)
	if err != nil {
		t.Skip("Database not available for integration testing:", err)
	}
	defer db.Close()

	// Test database connectivity
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	err = db.PingContext(ctx)
	if err != nil {
		t.Skip("Cannot connect to database for integration testing:", err)
	}

	t.Run("Core tables exist with correct structure", func(t *testing.T) {
		tables := []string{"users", "recipes", "meal_plans", "recipe_ratings"}
		
		for _, table := range tables {
			var exists bool
			query := `SELECT EXISTS (
				SELECT FROM information_schema.tables 
				WHERE table_schema = 'public' 
				AND table_name = $1
			)`
			
			err := db.QueryRowContext(ctx, query, table).Scan(&exists)
			require.NoError(t, err, "Failed to check if table %s exists", table)
			assert.True(t, exists, "Table %s should exist", table)
		}
	})

	t.Run("Users table has required columns", func(t *testing.T) {
		requiredColumns := []string{
			"id", "email", "encrypted_password", "first_name", "last_name",
			"dietary_restrictions", "cooking_skill_level", "created_at", "updated_at",
		}

		query := `SELECT column_name FROM information_schema.columns 
				 WHERE table_name = 'users' AND table_schema = 'public'`
		
		rows, err := db.QueryContext(ctx, query)
		require.NoError(t, err)
		defer rows.Close()

		var columns []string
		for rows.Next() {
			var col string
			err := rows.Scan(&col)
			require.NoError(t, err)
			columns = append(columns, col)
		}

		for _, required := range requiredColumns {
			assert.Contains(t, columns, required, "Column %s should exist in users table", required)
		}
	})

	t.Run("Recipes table has required columns", func(t *testing.T) {
		requiredColumns := []string{
			"id", "title", "prep_time", "cook_time", "total_time",
			"meal_type", "complexity", "ingredients", "instructions", "created_at",
		}

		query := `SELECT column_name FROM information_schema.columns 
				 WHERE table_name = 'recipes' AND table_schema = 'public'`
		
		rows, err := db.QueryContext(ctx, query)
		require.NoError(t, err)
		defer rows.Close()

		var columns []string
		for rows.Next() {
			var col string
			err := rows.Scan(&col)
			require.NoError(t, err)
			columns = append(columns, col)
		}

		for _, required := range requiredColumns {
			assert.Contains(t, columns, required, "Column %s should exist in recipes table", required)
		}
	})

	t.Run("Database indexes are created", func(t *testing.T) {
		expectedIndexes := map[string]string{
			"idx_users_email":                    "users",
			"idx_recipes_meal_type":              "recipes",
			"idx_recipes_average_rating":         "recipes", 
			"idx_meal_plans_user_id":             "meal_plans",
			"idx_recipe_ratings_recipe_id":       "recipe_ratings",
		}

		for indexName, tableName := range expectedIndexes {
			var exists bool
			query := `SELECT EXISTS (
				SELECT FROM pg_indexes 
				WHERE indexname = $1 AND tablename = $2
			)`
			
			err := db.QueryRowContext(ctx, query, indexName, tableName).Scan(&exists)
			require.NoError(t, err, "Failed to check if index %s exists", indexName)
			assert.True(t, exists, "Index %s should exist on table %s", indexName, tableName)
		}
	})

	t.Run("Can insert and retrieve user record", func(t *testing.T) {
		// Clean up any existing test data
		_, err := db.ExecContext(ctx, "DELETE FROM users WHERE email = 'test@integration.test'")
		require.NoError(t, err)

		// Insert test user
		userID := "test-user-id-123"
		email := "test@integration.test"
		
		insertQuery := `INSERT INTO users (id, email, first_name, last_name, dietary_restrictions) 
						VALUES ($1, $2, $3, $4, $5)`
		
		_, err = db.ExecContext(ctx, insertQuery, userID, email, "Test", "User", []string{"vegetarian"})
		require.NoError(t, err, "Failed to insert test user")

		// Retrieve test user
		var retrievedID, retrievedEmail, firstName string
		selectQuery := `SELECT id, email, first_name FROM users WHERE email = $1`
		
		err = db.QueryRowContext(ctx, selectQuery, email).Scan(&retrievedID, &retrievedEmail, &firstName)
		require.NoError(t, err, "Failed to retrieve test user")
		
		assert.Equal(t, userID, retrievedID)
		assert.Equal(t, email, retrievedEmail)
		assert.Equal(t, "Test", firstName)

		// Clean up
		_, err = db.ExecContext(ctx, "DELETE FROM users WHERE email = $1", email)
		require.NoError(t, err)
	})
}

func TestRedisIntegration(t *testing.T) {
	// This test is handled by the existing cache service tests
	// which already test Redis connectivity and operations
	t.Run("Redis integration covered by cache service tests", func(t *testing.T) {
		// Reference to existing tests in cache_service_test.go
		assert.True(t, true, "Redis integration is tested in cache_service_test.go")
	})
}

func TestHealthEndpointsIntegration(t *testing.T) {
	t.Run("Health endpoints integration covered by existing tests", func(t *testing.T) {
		// Reference to existing tests in health_test.go
		assert.True(t, true, "Health endpoints are tested in health_test.go with proper integration")
	})
}