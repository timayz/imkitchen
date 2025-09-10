package models

import (
	"time"
	"github.com/google/uuid"
	"github.com/lib/pq"
)

// RecipeImportRequest represents the request payload for importing a community recipe
type RecipeImportRequest struct {
	CommunityRecipeID   string                      `json:"communityRecipeId" validate:"required,uuid"`
	Customizations      *RecipeImportCustomizations `json:"customizations,omitempty"`
	PreserveAttribution bool                        `json:"preserveAttribution"`
}

// RecipeImportCustomizations represents optional customizations to apply during import
type RecipeImportCustomizations struct {
	Title             *string `json:"title,omitempty" validate:"omitempty,min=1,max=255"`
	Notes             *string `json:"notes,omitempty" validate:"omitempty,max=1000"`
	ServingAdjustment *int    `json:"servingAdjustment,omitempty" validate:"omitempty,min=1,max=50"`
}

// RecipeImportResponse represents the response after successfully importing a recipe
type RecipeImportResponse struct {
	Success          bool               `json:"success"`
	PersonalRecipeID *string            `json:"personalRecipeId,omitempty"`
	Message          string             `json:"message"`
	Attribution      *ImportAttribution `json:"attribution,omitempty"`
}

// ImportAttribution represents attribution information for imported recipes
type ImportAttribution struct {
	OriginalContributor string                       `json:"originalContributor"`
	ImportDate          time.Time                    `json:"importDate"`
	CommunityMetrics    RecipeImportCommunityMetrics `json:"communityMetrics"`
}

// RecipeImportCommunityMetrics represents metrics from the original community recipe
type RecipeImportCommunityMetrics struct {
	TotalImports  int     `json:"totalImports"`
	AverageRating float64 `json:"averageRating"`
}

// RecipeImport represents a record of importing a community recipe to personal collection
type RecipeImport struct {
	ID                  uuid.UUID  `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	UserID              uuid.UUID  `json:"userId" gorm:"column:user_id;type:uuid;not null;index"`
	PersonalRecipeID    uuid.UUID  `json:"personalRecipeId" gorm:"column:personal_recipe_id;type:uuid;not null"`
	CommunityRecipeID   uuid.UUID  `json:"communityRecipeId" gorm:"column:community_recipe_id;type:uuid;not null"`
	ImportedAt          time.Time  `json:"importedAt" gorm:"column:imported_at;not null"`
	PreserveAttribution bool       `json:"preserveAttribution" gorm:"column:preserve_attribution;default:true"`
	OriginalContributor *string    `json:"originalContributor,omitempty" gorm:"column:original_contributor;type:text"`
	ImportDate          *time.Time `json:"importDate,omitempty" gorm:"column:import_date"`
	
	// Relationships
	PersonalRecipe  *Recipe          `json:"personalRecipe,omitempty" gorm:"foreignKey:PersonalRecipeID"`
	CommunityRecipe *CommunityRecipe `json:"communityRecipe,omitempty" gorm:"foreignKey:CommunityRecipeID"`
}

// CommunityRecipe represents a recipe in the community database
type CommunityRecipe struct {
	ID                 uuid.UUID             `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	Title              string                `json:"title" gorm:"size:255;not null" validate:"required,min=1,max=255"`
	Description        *string               `json:"description,omitempty" gorm:"type:text"`
	ImageURL           *string               `json:"imageURL,omitempty" gorm:"column:image_url;type:text"`
	PrepTime           int                   `json:"prepTime" gorm:"column:prep_time;not null" validate:"required,min=0"`
	CookTime           int                   `json:"cookTime" gorm:"column:cook_time;not null" validate:"required,min=0"`
	TotalTime          int                   `json:"totalTime" gorm:"column:total_time;not null"`
	MealType           pq.StringArray        `json:"mealType" gorm:"column:meal_type;type:text[]" validate:"required"`
	Complexity         string                `json:"complexity" gorm:"size:20;not null" validate:"required,oneof=simple moderate complex"`
	CuisineType        *string               `json:"cuisineType,omitempty" gorm:"column:cuisine_type;size:100"`
	Servings           int                   `json:"servings" gorm:"not null" validate:"required,min=1"`
	Ingredients        []RecipeIngredient    `json:"ingredients" gorm:"type:jsonb" validate:"required,dive"`
	Instructions       []RecipeInstruction   `json:"instructions" gorm:"type:jsonb" validate:"required,dive"`
	DietaryLabels      pq.StringArray        `json:"dietaryLabels" gorm:"column:dietary_labels;type:text[]"`
	
	// Community-specific fields
	ContributorID      *uuid.UUID            `json:"contributorId,omitempty" gorm:"column:contributor_id;type:uuid"`
	ContributorName    *string               `json:"contributorName,omitempty" gorm:"column:contributor_name;size:255"`
	AverageRating      float64               `json:"averageRating" gorm:"column:average_rating;default:0"`
	TotalRatings       int                   `json:"totalRatings" gorm:"column:total_ratings;default:0"`
	ImportCount        int                   `json:"importCount" gorm:"column:import_count;default:0"`
	UserTags           pq.StringArray        `json:"userTags" gorm:"column:user_tags;type:text[]"`
	TrendingScore      float64               `json:"trendingScore" gorm:"column:trending_score;default:0"`
	IsPopular          bool                  `json:"isPopular" gorm:"column:is_popular;default:false"`
	IsTrending         bool                  `json:"isTrending" gorm:"column:is_trending;default:false"`
	
	// Metadata
	CreatedAt          time.Time             `json:"createdAt" gorm:"column:created_at"`
	UpdatedAt          time.Time             `json:"updatedAt" gorm:"column:updated_at"`
}

// ImportConflict represents a conflict when trying to import a recipe that already exists
type ImportConflict struct {
	ExistingRecipeID    string              `json:"existingRecipeId"`
	ExistingRecipeTitle string              `json:"existingRecipeTitle"`
	ImportedAt          time.Time           `json:"importedAt"`
	ConflictType        string              `json:"conflictType"` // "duplicate_import", "similar_recipe"
	Resolution          ConflictResolution  `json:"resolution"`
}

// ConflictResolution represents options for resolving import conflicts
type ConflictResolution struct {
	Options     []string `json:"options"`     // ["rename", "merge", "replace", "cancel"]
	Recommended string   `json:"recommended"` // The recommended option
}

// ImportStats represents statistics about a user's import activity
type ImportStats struct {
	TotalImports       int                `json:"totalImports"`
	RecentImports      int                `json:"recentImports"`      // Last 24 hours
	FavoriteCategories []CategoryStat     `json:"favoriteCategories"`
	ImportLimit        int                `json:"importLimit"`        // Per hour
	ImportLimitWindow  string             `json:"importLimitWindow"`  // "1h"
}

// CategoryStat represents statistics for a recipe category
type CategoryStat struct {
	Category string `json:"category"`
	Count    int    `json:"count"`
}

// Table names
func (RecipeImport) TableName() string {
	return "recipe_imports"
}

func (CommunityRecipe) TableName() string {
	return "community_recipes"
}