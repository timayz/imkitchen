package models

import (
	"time"

	"github.com/google/uuid"
)

// UserRecipeFavorite represents the junction table between users and their favorite recipes
type UserRecipeFavorite struct {
	ID               uuid.UUID `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	UserID           uuid.UUID `json:"userId" gorm:"type:uuid;not null;index"`
	RecipeID         uuid.UUID `json:"recipeId" gorm:"type:uuid;not null;index"`
	FavoritedAt      time.Time `json:"favoritedAt" gorm:"column:favorited_at;default:now()"`
	WeightMultiplier float64   `json:"weightMultiplier" gorm:"column:weight_multiplier;type:decimal(3,2);default:1.5"`
	
	// Foreign key relationships
	User   User   `json:"user" gorm:"foreignKey:UserID;constraint:OnDelete:CASCADE"`
	Recipe Recipe `json:"recipe" gorm:"foreignKey:RecipeID;constraint:OnDelete:CASCADE"`
	
	CreatedAt time.Time `json:"createdAt" gorm:"column:created_at;default:now()"`
	UpdatedAt time.Time `json:"updatedAt" gorm:"column:updated_at;default:now()"`
}

// UserWeeklyPattern represents user's cooking patterns for different days of the week
type UserWeeklyPattern struct {
	ID                  uuid.UUID `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	UserID              uuid.UUID `json:"userId" gorm:"type:uuid;not null;index"`
	DayOfWeek           int       `json:"dayOfWeek" gorm:"column:day_of_week;check:day_of_week >= 0 AND day_of_week <= 6"` // 0=Sunday, 6=Saturday
	MaxPrepTime         int       `json:"maxPrepTime" gorm:"column:max_prep_time;default:60"`                              // minutes
	PreferredComplexity string    `json:"preferredComplexity" gorm:"column:preferred_complexity;size:20;default:moderate" validate:"oneof=simple moderate complex"`
	IsWeekendPattern    bool      `json:"isWeekendPattern" gorm:"column:is_weekend_pattern;default:false"`
	
	// Foreign key relationship
	User User `json:"user" gorm:"foreignKey:UserID;constraint:OnDelete:CASCADE"`
	
	CreatedAt time.Time `json:"createdAt" gorm:"column:created_at;default:now()"`
	UpdatedAt time.Time `json:"updatedAt" gorm:"column:updated_at;default:now()"`
}

// CoreUserPreferences represents the simplified preference structure for Story 2.2B1
type CoreUserPreferences struct {
	MaxCookTime           int    `json:"maxCookTime" validate:"min=15,max=180"`                          // 15-180 minutes
	PreferredComplexity   string `json:"preferredComplexity" validate:"oneof=simple moderate complex"` // simple/moderate/complex
}

// TableName specifies the table name for UserRecipeFavorite
func (UserRecipeFavorite) TableName() string {
	return "user_recipe_favorites"
}

// TableName specifies the table name for UserWeeklyPattern  
func (UserWeeklyPattern) TableName() string {
	return "user_weekly_patterns"
}

// RotationResetLog tracks rotation reset events for analytics
type RotationResetLog struct {
	ID                 uuid.UUID `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	UserID             uuid.UUID `json:"userId" gorm:"type:uuid;not null;index"`
	ResetAt            time.Time `json:"resetAt" gorm:"column:reset_at;default:now()"`
	PreservedPatterns  bool      `json:"preservedPatterns" gorm:"column:preserved_patterns;default:true"`
	PreservedFavorites bool      `json:"preservedFavorites" gorm:"column:preserved_favorites;default:true"`
	WeeksCleared       int       `json:"weeksCleared" gorm:"column:weeks_cleared;default:0"`
	
	// Foreign key relationship
	User User `json:"user" gorm:"foreignKey:UserID;constraint:OnDelete:CASCADE"`
	
	CreatedAt time.Time `json:"createdAt" gorm:"column:created_at;default:now()"`
	UpdatedAt time.Time `json:"updatedAt" gorm:"column:updated_at;default:now()"`
}

// RotationResetRequest represents the request structure for rotation reset
type RotationResetRequest struct {
	ConfirmReset      bool `json:"confirmReset" validate:"required" binding:"required"`
	PreservePatterns  bool `json:"preservePatterns" default:"true"`
	PreserveFavorites bool `json:"preserveFavorites" default:"true"`
}

// RotationAnalytics represents comprehensive rotation analytics data
type RotationAnalytics struct {
	UserID             uuid.UUID             `json:"userId"`
	CalculatedAt       time.Time             `json:"calculatedAt"`
	VarietyScore       float64               `json:"varietyScore"`        // 0-100 scale
	WeeksAnalyzed      int                   `json:"weeksAnalyzed"`
	ComplexityDistribution map[string]float64 `json:"complexityDistribution"`
	FavoritesImpact    float64               `json:"favoritesImpact"`
	ComplexityTrends   []ComplexityTrendData `json:"complexityTrends"`
	FavoritesFrequency map[string]int        `json:"favoritesFrequency"`
	WeeklyPatterns     []WeeklyAnalysisData  `json:"weeklyPatterns"`
	RotationEfficiency float64               `json:"rotationEfficiency"` // Algorithm performance
}

// ComplexityTrendData represents complexity trend over time
type ComplexityTrendData struct {
	Week       string             `json:"week"`       // ISO week format
	Complexity map[string]float64 `json:"complexity"` // simple/moderate/complex percentages
}

// WeeklyAnalysisData represents detailed weekly pattern analysis
type WeeklyAnalysisData struct {
	Week            string  `json:"week"`
	PatternAdherence float64 `json:"patternAdherence"` // 0-100 how well patterns were followed
	VarietyScore    float64 `json:"varietyScore"`
	FavoritesRatio  float64 `json:"favoritesRatio"`   // ratio of favorites used
}

// TableName specifies the table name for RotationResetLog
func (RotationResetLog) TableName() string {
	return "rotation_reset_logs"
}