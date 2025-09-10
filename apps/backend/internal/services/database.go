package services

import (
	"fmt"
	"log"
	"os"
	"time"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

// DatabaseService provides database connection and operations
type DatabaseService struct {
	DB *gorm.DB
}

// NewDatabaseService creates a new database service with connection
func NewDatabaseService() (*DatabaseService, error) {
	db, err := connectDatabase()
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %w", err)
	}

	return &DatabaseService{DB: db}, nil
}

// connectDatabase establishes connection to PostgreSQL database
func connectDatabase() (*gorm.DB, error) {
	// Get database configuration from environment
	host := getEnvDefault("DB_HOST", "localhost")
	port := getEnvDefault("DB_PORT", "5432")
	user := getEnvDefault("DB_USER", "imkitchen")
	password := getEnvDefault("DB_PASSWORD", "imkitchen")
	dbname := getEnvDefault("DB_NAME", "imkitchen")
	sslmode := getEnvDefault("DB_SSLMODE", "disable")
	timezone := getEnvDefault("DB_TIMEZONE", "UTC")

	// Construct DSN
	dsn := fmt.Sprintf("host=%s port=%s user=%s password=%s dbname=%s sslmode=%s TimeZone=%s",
		host, port, user, password, dbname, sslmode, timezone)

	// Configure GORM logger
	gormLogger := logger.Default
	if getEnvDefault("ENVIRONMENT", "development") == "development" {
		gormLogger = logger.Default.LogMode(logger.Info)
	} else {
		gormLogger = logger.Default.LogMode(logger.Error)
	}

	// Connect to database
	db, err := gorm.Open(postgres.Open(dsn), &gorm.Config{
		Logger: gormLogger,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %w", err)
	}

	// Configure connection pool
	sqlDB, err := db.DB()
	if err != nil {
		return nil, fmt.Errorf("failed to get database instance: %w", err)
	}

	sqlDB.SetMaxIdleConns(10)
	sqlDB.SetMaxOpenConns(100)
	sqlDB.SetConnMaxLifetime(time.Hour)

	// Test the connection
	if err := sqlDB.Ping(); err != nil {
		return nil, fmt.Errorf("failed to ping database: %w", err)
	}

	log.Println("Database connected successfully")
	return db, nil
}

// Close closes the database connection
func (ds *DatabaseService) Close() error {
	sqlDB, err := ds.DB.DB()
	if err != nil {
		return err
	}
	return sqlDB.Close()
}

// Ping tests the database connection
func (ds *DatabaseService) Ping() error {
	sqlDB, err := ds.DB.DB()
	if err != nil {
		return err
	}
	return sqlDB.Ping()
}

// Helper function to get environment variable with default
func getEnvDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}