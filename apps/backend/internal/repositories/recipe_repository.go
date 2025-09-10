package repositories

import (
	"context"
	"time"
	
	"github.com/google/uuid"
)

// RecipeRepository handles recipe data operations
type RecipeRepository interface {
	Create(ctx context.Context, recipe *Recipe) error
	GetByID(ctx context.Context, id uuid.UUID) (*Recipe, error)
	GetByUserID(ctx context.Context, userID uuid.UUID, filters *RecipeFilters) ([]*Recipe, error)
	Update(ctx context.Context, id uuid.UUID, updates *RecipeUpdates) error
	Delete(ctx context.Context, id uuid.UUID) error
	Search(ctx context.Context, query *RecipeSearchQuery) ([]*Recipe, error)
	GetByExternalSource(ctx context.Context, source, externalID string) (*Recipe, error)
	GetCommunityRecipeByID(ctx context.Context, id uuid.UUID) (*Recipe, error)
}

// Recipe represents a recipe
type Recipe struct {
	ID               uuid.UUID              `json:"id" db:"id"`
	UserID           uuid.UUID              `json:"user_id" db:"user_id"`
	Title            string                 `json:"title" db:"title"`
	Description      string                 `json:"description" db:"description"`
	Ingredients      []RecipeIngredient     `json:"ingredients" db:"ingredients"`
	Instructions     []RecipeInstruction    `json:"instructions" db:"instructions"`
	PrepTime         int                    `json:"prep_time" db:"prep_time"`
	CookTime         int                    `json:"cook_time" db:"cook_time"`
	Servings         int                    `json:"servings" db:"servings"`
	Cuisine          string                 `json:"cuisine" db:"cuisine"`
	DietaryLabels    []string              `json:"dietary_labels" db:"dietary_labels"`
	Tags             []string              `json:"tags" db:"tags"`
	ImageURL         *string               `json:"image_url" db:"image_url"`
	SourceURL        *string               `json:"source_url" db:"source_url"`
	ExternalSource   *string               `json:"external_source" db:"external_source"`
	ExternalID       *string               `json:"external_id" db:"external_id"`
	IsPublic         bool                  `json:"is_public" db:"is_public"`
	Rating           float64               `json:"rating" db:"rating"`
	RatingCount      int                   `json:"rating_count" db:"rating_count"`
	CreatedAt        time.Time             `json:"created_at" db:"created_at"`
	UpdatedAt        time.Time             `json:"updated_at" db:"updated_at"`
}

// RecipeIngredient represents a recipe ingredient
type RecipeIngredient struct {
	Name     string  `json:"name"`
	Amount   float64 `json:"amount"`
	Unit     string  `json:"unit"`
	Notes    string  `json:"notes,omitempty"`
}

// RecipeInstruction represents a recipe instruction step
type RecipeInstruction struct {
	StepNumber int    `json:"step_number"`
	Text       string `json:"text"`
	Duration   int    `json:"duration,omitempty"` // in minutes
}

// RecipeFilters represents filters for recipe queries
type RecipeFilters struct {
	Cuisine       *string    `json:"cuisine,omitempty"`
	DietaryLabels []string   `json:"dietary_labels,omitempty"`
	MaxPrepTime   *int       `json:"max_prep_time,omitempty"`
	MaxCookTime   *int       `json:"max_cook_time,omitempty"`
	Tags          []string   `json:"tags,omitempty"`
	IsPublic      *bool      `json:"is_public,omitempty"`
	Limit         *int       `json:"limit,omitempty"`
	Offset        *int       `json:"offset,omitempty"`
}

// RecipeUpdates represents updates to a recipe
type RecipeUpdates struct {
	Title         *string              `json:"title,omitempty"`
	Description   *string              `json:"description,omitempty"`
	Ingredients   []RecipeIngredient   `json:"ingredients,omitempty"`
	Instructions  []RecipeInstruction  `json:"instructions,omitempty"`
	PrepTime      *int                 `json:"prep_time,omitempty"`
	CookTime      *int                 `json:"cook_time,omitempty"`
	Servings      *int                 `json:"servings,omitempty"`
	Cuisine       *string              `json:"cuisine,omitempty"`
	DietaryLabels []string             `json:"dietary_labels,omitempty"`
	Tags          []string             `json:"tags,omitempty"`
	ImageURL      *string              `json:"image_url,omitempty"`
	IsPublic      *bool                `json:"is_public,omitempty"`
}

// RecipeSearchQuery represents a search query for recipes
type RecipeSearchQuery struct {
	Query         string     `json:"query,omitempty"`
	UserID        *uuid.UUID `json:"user_id,omitempty"`
	Filters       *RecipeFilters `json:"filters,omitempty"`
	SortBy        string     `json:"sort_by,omitempty"`
	SortOrder     string     `json:"sort_order,omitempty"`
}