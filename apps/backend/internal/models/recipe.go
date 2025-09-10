package models

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"github.com/lib/pq"
)

// RecipeIngredient represents a single ingredient in a recipe
type RecipeIngredient struct {
	Name     string `json:"name" validate:"required,min=1,max=255"`
	Amount   float64 `json:"amount" validate:"required,min=0"`
	Unit     string `json:"unit" validate:"required,min=1,max=50"`
	Category string `json:"category" validate:"required,oneof=produce dairy pantry protein other"`
}

// RecipeInstruction represents a single step in recipe preparation
type RecipeInstruction struct {
	StepNumber        int  `json:"stepNumber" validate:"required,min=1"`
	Instruction       string `json:"instruction" validate:"required,min=1"`
	EstimatedMinutes  *int `json:"estimatedMinutes,omitempty" validate:"omitempty,min=0,max=480"`
}

// RecipeNutrition holds nutritional information for a recipe
type RecipeNutrition struct {
	Calories *float64 `json:"calories,omitempty"`
	Protein  *float64 `json:"protein,omitempty"`
	Carbs    *float64 `json:"carbs,omitempty"`
	Fat      *float64 `json:"fat,omitempty"`
	Fiber    *float64 `json:"fiber,omitempty"`
	Sugar    *float64 `json:"sugar,omitempty"`
	Sodium   *float64 `json:"sodium,omitempty"`
}

// Recipe represents a complete recipe record
type Recipe struct {
	ID             uuid.UUID           `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	UserID         uuid.UUID           `json:"userId" gorm:"column:user_id;type:uuid;not null"`
	ExternalID     *string             `json:"externalId,omitempty" gorm:"column:external_id;size:255"`
	ExternalSource *string             `json:"externalSource,omitempty" gorm:"column:external_source;size:50"`
	
	// Basic Recipe Info
	Title       string  `json:"title" gorm:"size:255;not null" validate:"required,min=1,max=255"`
	Description *string `json:"description,omitempty" gorm:"type:text"`
	ImageURL    *string `json:"imageUrl,omitempty" gorm:"column:image_url;type:text"`
	SourceURL   *string `json:"sourceUrl,omitempty" gorm:"column:source_url;type:text"`
	
	// Timing
	PrepTime   int `json:"prepTime" gorm:"column:prep_time;not null" validate:"required,min=0,max=960"`
	CookTime   int `json:"cookTime" gorm:"column:cook_time;not null" validate:"required,min=0,max=960"`
	TotalTime  int `json:"totalTime" gorm:"column:total_time;->"`  // Generated column
	
	// Classification
	MealType   pq.StringArray `json:"mealType" gorm:"column:meal_type;type:varchar(20)[]" validate:"required,dive,oneof=breakfast lunch dinner snack"`
	Complexity string         `json:"complexity" gorm:"size:20" validate:"required,oneof=simple moderate complex"`
	CuisineType *string       `json:"cuisineType,omitempty" gorm:"column:cuisine_type;size:50"`
	
	// Recipe Data
	Servings     int                 `json:"servings" gorm:"default:4" validate:"required,min=1,max=20"`
	Ingredients  json.RawMessage     `json:"ingredients" gorm:"type:jsonb;not null" validate:"required"`
	Instructions json.RawMessage     `json:"instructions" gorm:"type:jsonb;not null" validate:"required"`
	
	// Nutritional Information
	Nutrition      *json.RawMessage `json:"nutrition,omitempty" gorm:"type:jsonb"`
	DietaryLabels  pq.StringArray   `json:"dietaryLabels" gorm:"column:dietary_labels;type:text[];default:'{}'"`
	
	// Visibility
	IsPublic bool `json:"isPublic" gorm:"column:is_public;default:false"`
	
	// Quality Metrics
	AverageRating  float64 `json:"averageRating" gorm:"column:average_rating;type:decimal(3,2);default:0.0"`
	TotalRatings   int     `json:"totalRatings" gorm:"column:total_ratings;default:0"`
	DifficultyScore *int   `json:"difficultyScore,omitempty" gorm:"column:difficulty_score"`
	
	// Metadata
	CreatedAt time.Time  `json:"createdAt" gorm:"column:created_at;default:now()"`
	UpdatedAt time.Time  `json:"updatedAt" gorm:"column:updated_at;default:now()"`
	DeletedAt *time.Time `json:"deletedAt,omitempty" gorm:"column:deleted_at"`
}

// CreateRecipeInput represents input for creating a new recipe
type CreateRecipeInput struct {
	Title         string              `json:"title" validate:"required,min=1,max=255"`
	Description   *string             `json:"description,omitempty"`
	PrepTime      int                 `json:"prepTime" validate:"required,min=0,max=960"`
	CookTime      int                 `json:"cookTime" validate:"required,min=0,max=960"`
	MealType      []string            `json:"mealType" validate:"required,dive,oneof=breakfast lunch dinner snack"`
	Complexity    string              `json:"complexity" validate:"required,oneof=simple moderate complex"`
	CuisineType   *string             `json:"cuisineType,omitempty"`
	Servings      int                 `json:"servings" validate:"required,min=1,max=20"`
	Ingredients   []RecipeIngredient  `json:"ingredients" validate:"required,min=1,dive"`
	Instructions  []RecipeInstruction `json:"instructions" validate:"required,min=1,dive"`
	DietaryLabels []string            `json:"dietaryLabels,omitempty"`
	ImageURL      *string             `json:"imageUrl,omitempty"`
	SourceURL     *string             `json:"sourceUrl,omitempty"`
}

// UpdateRecipeInput represents input for updating an existing recipe
type UpdateRecipeInput struct {
	Title         *string              `json:"title,omitempty" validate:"omitempty,min=1,max=255"`
	Description   *string              `json:"description,omitempty"`
	PrepTime      *int                 `json:"prepTime,omitempty" validate:"omitempty,min=0,max=960"`
	CookTime      *int                 `json:"cookTime,omitempty" validate:"omitempty,min=0,max=960"`
	MealType      []string             `json:"mealType,omitempty" validate:"omitempty,dive,oneof=breakfast lunch dinner snack"`
	Complexity    *string              `json:"complexity,omitempty" validate:"omitempty,oneof=simple moderate complex"`
	CuisineType   *string              `json:"cuisineType,omitempty"`
	Servings      *int                 `json:"servings,omitempty" validate:"omitempty,min=1,max=20"`
	Ingredients   *[]RecipeIngredient  `json:"ingredients,omitempty" validate:"omitempty,min=1,dive"`
	Instructions  *[]RecipeInstruction `json:"instructions,omitempty" validate:"omitempty,min=1,dive"`
	DietaryLabels []string             `json:"dietaryLabels,omitempty"`
	ImageURL      *string              `json:"imageUrl,omitempty"`
	SourceURL     *string              `json:"sourceUrl,omitempty"`
}

// RecipeFilters represents filters for recipe search
type RecipeFilters struct {
	MealType      *string     `form:"mealType"`
	Complexity    *string     `form:"complexity"`
	MaxPrepTime   *int        `form:"maxPrepTime"`
	MaxCookTime   *int        `form:"maxCookTime"`
	MaxTotalTime  *int        `form:"maxTotalTime"`
	CuisineType   *string     `form:"cuisineType"`
	DietaryLabels []string    `form:"dietaryLabels[]"`
	Search        *string     `form:"search"`
	ExcludeIDs    []uuid.UUID `form:"-"` // Not from form, used internally
}

// RecipeSearchParams extends filters with pagination and sorting
type RecipeSearchParams struct {
	RecipeFilters
	Page      int    `form:"page" validate:"min=1"`
	Limit     int    `form:"limit" validate:"min=1,max=100"`
	SortBy    string `form:"sortBy" validate:"oneof=created_at updated_at total_time average_rating"`
	SortOrder string `form:"sortOrder" validate:"oneof=asc desc"`
}

// RecipeSearchResponse represents the response from recipe search
type RecipeSearchResponse struct {
	Recipes    []Recipe `json:"recipes"`
	Total      int64    `json:"total"`
	Page       int      `json:"page"`
	Limit      int      `json:"limit"`
	TotalPages int      `json:"totalPages"`
}

// ImportRecipeInput represents input for importing a recipe from URL
type ImportRecipeInput struct {
	URL            string             `json:"url" validate:"required,url"`
	OverrideFields *CreateRecipeInput `json:"overrideFields,omitempty"`
}

// ImportRecipeResult represents the result of recipe import
type ImportRecipeResult struct {
	Success  bool     `json:"success"`
	Recipe   *Recipe  `json:"recipe,omitempty"`
	Error    *string  `json:"error,omitempty"`
	Warnings []string `json:"warnings,omitempty"`
}

// TableName specifies the table name for GORM
func (Recipe) TableName() string {
	return "recipes"
}