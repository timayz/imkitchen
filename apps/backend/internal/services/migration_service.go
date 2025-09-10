package services

import (
	"fmt"
	"log"

	"github.com/imkitchen/backend/internal/models"
	"gorm.io/gorm"
)

// MigrationService handles database schema migrations
type MigrationService struct {
	DB *gorm.DB
}

// NewMigrationService creates a new migration service
func NewMigrationService(db *gorm.DB) *MigrationService {
	return &MigrationService{DB: db}
}

// RunMigrations executes all necessary database migrations
func (ms *MigrationService) RunMigrations() error {
	log.Println("Starting database migrations...")
	
	// Auto-migrate all models
	err := ms.DB.AutoMigrate(
		&models.User{},
		&models.Recipe{},
		&models.MealPlan{},
		&models.MealPlanChangeHistory{},
		&models.UserRecipeFavorite{},
		&models.UserWeeklyPattern{},
	)
	if err != nil {
		return fmt.Errorf("failed to run auto-migrations: %w", err)
	}

	// Create unique constraint for user_recipe_favorites (user_id, recipe_id)
	err = ms.createUniqueConstraints()
	if err != nil {
		return fmt.Errorf("failed to create unique constraints: %w", err)
	}

	// Create indexes for better performance
	err = ms.createIndexes()
	if err != nil {
		return fmt.Errorf("failed to create indexes: %w", err)
	}

	log.Println("Database migrations completed successfully")
	return nil
}

// createUniqueConstraints creates necessary unique constraints
func (ms *MigrationService) createUniqueConstraints() error {
	// Create unique constraint on user_recipe_favorites (user_id, recipe_id)
	sql := `
		DO $$ BEGIN
			IF NOT EXISTS (
				SELECT 1 FROM information_schema.table_constraints 
				WHERE constraint_name = 'unique_user_recipe_favorite'
			) THEN
				ALTER TABLE user_recipe_favorites 
				ADD CONSTRAINT unique_user_recipe_favorite UNIQUE (user_id, recipe_id);
			END IF;
		END $$;
	`
	err := ms.DB.Exec(sql).Error
	if err != nil {
		return fmt.Errorf("failed to create unique constraint on user_recipe_favorites: %w", err)
	}

	return nil
}

// createIndexes creates necessary database indexes for performance
func (ms *MigrationService) createIndexes() error {
	indexes := []string{
		"CREATE INDEX IF NOT EXISTS idx_user_recipe_favorites_user_id ON user_recipe_favorites(user_id);",
		"CREATE INDEX IF NOT EXISTS idx_user_recipe_favorites_recipe_id ON user_recipe_favorites(recipe_id);",
		"CREATE INDEX IF NOT EXISTS idx_user_recipe_favorites_favorited_at ON user_recipe_favorites(favorited_at);",
		"CREATE INDEX IF NOT EXISTS idx_user_weekly_patterns_user_id ON user_weekly_patterns(user_id);",
		"CREATE INDEX IF NOT EXISTS idx_user_weekly_patterns_day_of_week ON user_weekly_patterns(day_of_week);",
		"CREATE INDEX IF NOT EXISTS idx_user_weekly_patterns_weekend ON user_weekly_patterns(is_weekend_pattern);",
		"CREATE INDEX IF NOT EXISTS idx_meal_plan_change_history_meal_plan_id ON meal_plan_change_history(meal_plan_id);",
		"CREATE INDEX IF NOT EXISTS idx_meal_plan_change_history_user_id ON meal_plan_change_history(user_id);",
		"CREATE INDEX IF NOT EXISTS idx_meal_plan_change_history_change_type ON meal_plan_change_history(change_type);",
		"CREATE INDEX IF NOT EXISTS idx_meal_plan_change_history_created_at ON meal_plan_change_history(created_at DESC);",
	}

	for _, indexSQL := range indexes {
		if err := ms.DB.Exec(indexSQL).Error; err != nil {
			return fmt.Errorf("failed to create index: %s, error: %w", indexSQL, err)
		}
	}

	log.Println("Database indexes created successfully")
	return nil
}

// DropTables drops all preference-related tables (for rollback testing)
func (ms *MigrationService) DropTables() error {
	log.Println("Dropping preference tables for rollback testing...")
	
	// Drop tables in reverse order of dependencies
	tables := []string{
		"user_recipe_favorites",
		"user_weekly_patterns",
	}
	
	for _, table := range tables {
		sql := fmt.Sprintf("DROP TABLE IF EXISTS %s CASCADE;", table)
		if err := ms.DB.Exec(sql).Error; err != nil {
			return fmt.Errorf("failed to drop table %s: %w", table, err)
		}
		log.Printf("Dropped table: %s", table)
	}
	
	return nil
}

// ValidateSchema validates that all required tables and constraints exist
func (ms *MigrationService) ValidateSchema() error {
	// Check if tables exist
	tables := []string{
		"user_recipe_favorites",
		"user_weekly_patterns",
	}
	
	for _, table := range tables {
		var exists bool
		query := `
			SELECT EXISTS (
				SELECT 1 FROM information_schema.tables 
				WHERE table_name = ? AND table_schema = 'public'
			);
		`
		if err := ms.DB.Raw(query, table).Scan(&exists).Error; err != nil {
			return fmt.Errorf("failed to check table %s existence: %w", table, err)
		}
		
		if !exists {
			return fmt.Errorf("required table %s does not exist", table)
		}
	}
	
	log.Println("Schema validation passed")
	return nil
}