package tests

import (
	"fmt"
	"os"
	"testing"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

// setupTestDB creates a test database connection
func setupTestDB(t *testing.T) *gorm.DB {
	// Use test database URL or default
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		dbURL = "postgres://imkitchen_user:imkitchen_password@localhost:5432/imkitchen_test?sslmode=disable"
	}

	// Create connection
	db, err := gorm.Open(postgres.Open(dbURL), &gorm.Config{
		Logger: logger.Default.LogMode(logger.Silent), // Silent for tests
	})
	if err != nil {
		t.Skip("Test database not available:", err)
	}

	// Clean up function
	t.Cleanup(func() {
		// Clean test data
		db.Exec("DELETE FROM user_recipe_favorites")
		db.Exec("DELETE FROM user_weekly_patterns")
		db.Exec("DELETE FROM users WHERE email LIKE '%@example.com'")
		db.Exec("DELETE FROM recipes WHERE title LIKE 'Test %' OR title LIKE 'Favorite %'")
		
		sqlDB, _ := db.DB()
		if sqlDB != nil {
			sqlDB.Close()
		}
	})

	return db
}

func TestMigrationService(t *testing.T) {
	db := setupTestDB(t)
	migrationService := services.NewMigrationService(db)

	t.Run("RunMigrations creates all required tables", func(t *testing.T) {
		// Run migrations
		err := migrationService.RunMigrations()
		require.NoError(t, err)

		// Verify tables exist
		err = migrationService.ValidateSchema()
		assert.NoError(t, err)

		// Test table structures by inserting test records
		testUser := models.User{
			Email:                   "test@example.com",
			CookingSkillLevel:       "intermediate",
			PreferredMealComplexity: "moderate",
			MaxCookTime:             60,
		}
		err = db.Create(&testUser).Error
		require.NoError(t, err)

		testRecipe := models.Recipe{
			Title:       "Test Recipe",
			Complexity:  "simple",
			PrepTime:    30,
			Description: "Test description",
		}
		err = db.Create(&testRecipe).Error
		require.NoError(t, err)

		// Test UserRecipeFavorite creation
		favorite := models.UserRecipeFavorite{
			UserID:           testUser.ID,
			RecipeID:         testRecipe.ID,
			WeightMultiplier: 1.5,
		}
		err = db.Create(&favorite).Error
		assert.NoError(t, err)

		// Test UserWeeklyPattern creation
		pattern := models.UserWeeklyPattern{
			UserID:              testUser.ID,
			DayOfWeek:           1, // Monday
			MaxPrepTime:         45,
			PreferredComplexity: "simple",
			IsWeekendPattern:    false,
		}
		err = db.Create(&pattern).Error
		assert.NoError(t, err)

		// Verify unique constraint works
		duplicateFavorite := models.UserRecipeFavorite{
			UserID:           testUser.ID,
			RecipeID:         testRecipe.ID,
			WeightMultiplier: 2.0,
		}
		err = db.Create(&duplicateFavorite).Error
		assert.Error(t, err, "Should fail due to unique constraint")
	})

	t.Run("DropTables removes preference tables", func(t *testing.T) {
		// Ensure tables exist first
		err := migrationService.RunMigrations()
		require.NoError(t, err)

		// Drop tables
		err = migrationService.DropTables()
		assert.NoError(t, err)

		// Verify tables are gone
		var exists bool
		err = db.Raw("SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'user_recipe_favorites')").Scan(&exists).Error
		require.NoError(t, err)
		assert.False(t, exists, "user_recipe_favorites table should be dropped")

		err = db.Raw("SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'user_weekly_patterns')").Scan(&exists).Error
		require.NoError(t, err)
		assert.False(t, exists, "user_weekly_patterns table should be dropped")
	})

	t.Run("ValidateSchema detects missing tables", func(t *testing.T) {
		// Drop tables first
		err := migrationService.DropTables()
		require.NoError(t, err)

		// Validation should fail
		err = migrationService.ValidateSchema()
		assert.Error(t, err, "Should fail when required tables are missing")
	})
}

func TestUserPreferenceModels(t *testing.T) {
	db := setupTestDB(t)
	migrationService := services.NewMigrationService(db)
	
	// Run migrations
	err := migrationService.RunMigrations()
	require.NoError(t, err)

	t.Run("UserRecipeFavorite model validation", func(t *testing.T) {
		// Create test user and recipe
		user := models.User{
			Email:                   "user@example.com",
			CookingSkillLevel:       "beginner",
			PreferredMealComplexity: "simple",
			MaxCookTime:             30,
		}
		err := db.Create(&user).Error
		require.NoError(t, err)

		recipe := models.Recipe{
			Title:       "Favorite Recipe",
			Complexity:  "simple",
			PrepTime:    20,
			Description: "A favorite recipe",
		}
		err = db.Create(&recipe).Error
		require.NoError(t, err)

		// Test favorite creation
		favorite := models.UserRecipeFavorite{
			UserID:           user.ID,
			RecipeID:         recipe.ID,
			WeightMultiplier: 2.0,
		}
		err = db.Create(&favorite).Error
		assert.NoError(t, err)

		// Verify foreign key relationships
		var retrievedFavorite models.UserRecipeFavorite
		err = db.Preload("User").Preload("Recipe").Where("user_id = ? AND recipe_id = ?", user.ID, recipe.ID).First(&retrievedFavorite).Error
		require.NoError(t, err)
		
		assert.Equal(t, user.Email, retrievedFavorite.User.Email)
		assert.Equal(t, recipe.Title, retrievedFavorite.Recipe.Title)
		assert.Equal(t, 2.0, retrievedFavorite.WeightMultiplier)
	})

	t.Run("UserWeeklyPattern model validation", func(t *testing.T) {
		// Create test user
		user := models.User{
			Email:                   "pattern@example.com",
			CookingSkillLevel:       "advanced",
			PreferredMealComplexity: "complex",
			MaxCookTime:             120,
		}
		err := db.Create(&user).Error
		require.NoError(t, err)

		// Test pattern creation
		pattern := models.UserWeeklyPattern{
			UserID:              user.ID,
			DayOfWeek:           6, // Saturday
			MaxPrepTime:         90,
			PreferredComplexity: "complex",
			IsWeekendPattern:    true,
		}
		err = db.Create(&pattern).Error
		assert.NoError(t, err)

		// Verify constraints
		invalidPattern := models.UserWeeklyPattern{
			UserID:              user.ID,
			DayOfWeek:           8, // Invalid day
			MaxPrepTime:         60,
			PreferredComplexity: "moderate",
			IsWeekendPattern:    false,
		}
		err = db.Create(&invalidPattern).Error
		assert.Error(t, err, "Should fail due to day_of_week constraint")

		// Verify foreign key relationship
		var retrievedPattern models.UserWeeklyPattern
		err = db.Preload("User").Where("user_id = ? AND day_of_week = ?", user.ID, 6).First(&retrievedPattern).Error
		require.NoError(t, err)
		
		assert.Equal(t, user.Email, retrievedPattern.User.Email)
		assert.Equal(t, 90, retrievedPattern.MaxPrepTime)
		assert.True(t, retrievedPattern.IsWeekendPattern)
	})
}