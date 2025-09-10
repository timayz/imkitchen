// +build ignore

package main

import (
	"log"
	"os"

	"github.com/imkitchen/backend/internal/models"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

func main() {
	// Get database URL from environment
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		dbURL = "postgres://imkitchen_user:imkitchen_password@localhost:5432/imkitchen_dev?sslmode=disable"
	}

	// Connect to database
	db, err := gorm.Open(postgres.Open(dbURL), &gorm.Config{
		Logger: logger.Default.LogMode(logger.Info),
	})
	if err != nil {
		log.Fatalf("Failed to connect to database: %v", err)
	}

	log.Println("Running database migrations...")

	// Auto-migrate all models
	err = db.AutoMigrate(
		&models.User{},
		&models.Recipe{},
		&models.MealPlan{},
		&models.UserRecipeFavorite{},
		&models.UserWeeklyPattern{},
	)
	if err != nil {
		log.Fatalf("Failed to run auto-migrations: %v", err)
	}

	// Create unique constraint for user_recipe_favorites
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
	err = db.Exec(sql).Error
	if err != nil {
		log.Fatalf("Failed to create unique constraint: %v", err)
	}

	// Create indexes
	indexes := []string{
		"CREATE INDEX IF NOT EXISTS idx_user_recipe_favorites_user_id ON user_recipe_favorites(user_id);",
		"CREATE INDEX IF NOT EXISTS idx_user_recipe_favorites_recipe_id ON user_recipe_favorites(recipe_id);",
		"CREATE INDEX IF NOT EXISTS idx_user_weekly_patterns_user_id ON user_weekly_patterns(user_id);",
		"CREATE INDEX IF NOT EXISTS idx_user_weekly_patterns_day_of_week ON user_weekly_patterns(day_of_week);",
	}

	for _, indexSQL := range indexes {
		if err := db.Exec(indexSQL).Error; err != nil {
			log.Printf("Warning: Failed to create index: %s, error: %v", indexSQL, err)
		} else {
			log.Printf("Created index: %s", indexSQL)
		}
	}

	// Validate schema
	var tableCount int64
	db.Raw("SELECT COUNT(*) FROM information_schema.tables WHERE table_name IN ('user_recipe_favorites', 'user_weekly_patterns')").Scan(&tableCount)
	
	if tableCount == 2 {
		log.Println("✅ Migration completed successfully!")
		log.Println("✅ Both user_recipe_favorites and user_weekly_patterns tables created")
	} else {
		log.Printf("⚠️ Warning: Expected 2 tables, found %d", tableCount)
	}

	log.Println("Migration process completed.")
}