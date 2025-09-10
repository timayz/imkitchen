package models

import (
	"time"
	"github.com/google/uuid"
)

// MealPlanPreferences represents user preferences for meal planning
type MealPlanPreferences struct {
	ID                    uuid.UUID `json:"id" db:"id"`
	UserID                uuid.UUID `json:"user_id" db:"user_id"`
	PreferredCuisines     []string  `json:"preferred_cuisines" db:"preferred_cuisines"`
	DietaryRestrictions   []string  `json:"dietary_restrictions" db:"dietary_restrictions"`
	MaxPrepTime           int       `json:"max_prep_time" db:"max_prep_time"` // minutes
	MaxCookTime           int       `json:"max_cook_time" db:"max_cook_time"` // minutes
	ServingSize           int       `json:"serving_size" db:"serving_size"`
	MealsPerDay           int       `json:"meals_per_day" db:"meals_per_day"`
	PlanningDays          int       `json:"planning_days" db:"planning_days"` // days ahead to plan
	AvoidRepeat           int       `json:"avoid_repeat" db:"avoid_repeat"`   // days before repeating a recipe
	BudgetLimit           float64   `json:"budget_limit" db:"budget_limit"`
	PreferredShoppingDays []string  `json:"preferred_shopping_days" db:"preferred_shopping_days"`
	CreatedAt             time.Time `json:"created_at" db:"created_at"`
	UpdatedAt             time.Time `json:"updated_at" db:"updated_at"`
}

// MealPlan represents a user's meal plan
type MealPlan struct {
	ID          uuid.UUID `json:"id" db:"id"`
	UserID      uuid.UUID `json:"user_id" db:"user_id"`
	Name        string    `json:"name" db:"name"`
	StartDate   time.Time `json:"start_date" db:"start_date"`
	EndDate     time.Time `json:"end_date" db:"end_date"`
	IsActive    bool      `json:"is_active" db:"is_active"`
	MealSlots   []MealSlot `json:"meal_slots,omitempty" db:"-"`
	Meals       []byte    `json:"meals,omitempty" db:"meals"` // JSONB field
	Entries     []MealSlot `json:"entries,omitempty" db:"-"`  // For input processing
	CreatedAt   time.Time `json:"created_at" db:"created_at"`
	UpdatedAt   time.Time `json:"updated_at" db:"updated_at"`
}

// MealSlot represents a single meal slot in a meal plan
type MealSlot struct {
	ID               uuid.UUID `json:"id" db:"id"`
	MealPlanID       uuid.UUID `json:"meal_plan_id" db:"meal_plan_id"`
	RecipeID         *uuid.UUID `json:"recipe_id,omitempty" db:"recipe_id"`
	Recipe           interface{} `json:"recipe,omitempty" db:"-"` // Populated when fetching with joins
	Date             time.Time `json:"date" db:"date"`
	MealType         string    `json:"meal_type" db:"meal_type"` // breakfast, lunch, dinner, snack
	IsManualOverride bool      `json:"is_manual_override" db:"is_manual_override"`
	IsLocked         bool      `json:"is_locked" db:"is_locked"`
	IsCompleted      bool      `json:"is_completed" db:"is_completed"`
	Notes            *string   `json:"notes,omitempty" db:"notes"`
	Servings         *int      `json:"servings,omitempty" db:"servings"`
	CreatedAt        time.Time `json:"created_at" db:"created_at"`
	UpdatedAt        time.Time `json:"updated_at" db:"updated_at"`
}

// MealPlanFilters represents filters for querying meal plans
type MealPlanFilters struct {
	UserID    *uuid.UUID `json:"user_id,omitempty"`
	IsActive  *bool      `json:"is_active,omitempty"`
	StartDate *time.Time `json:"start_date,omitempty"`
	EndDate   *time.Time `json:"end_date,omitempty"`
	WeekStart *time.Time `json:"week_start,omitempty"`
	WeekEnd   *time.Time `json:"week_end,omitempty"`
	Status    *string    `json:"status,omitempty"`
	Limit     *int       `json:"limit,omitempty"`
	Offset    *int       `json:"offset,omitempty"`
}

// UpdateMealPlanInput represents input for updating a meal plan
type UpdateMealPlanInput struct {
	Name                 *string     `json:"name,omitempty"`
	StartDate            *time.Time  `json:"start_date,omitempty"`
	EndDate              *time.Time  `json:"end_date,omitempty"`
	IsActive             *bool       `json:"is_active,omitempty"`
	Status               *string     `json:"status,omitempty"`
	CompletionPercentage *float64    `json:"completion_percentage,omitempty"`
	UserFeedback         *string     `json:"user_feedback,omitempty"`
	Meals                []MealSlot  `json:"meals,omitempty"`
}

// UpdateMealSlotInput represents input for updating a meal slot
type UpdateMealSlotInput struct {
	RecipeID         *uuid.UUID `json:"recipe_id,omitempty"`
	IsManualOverride *bool      `json:"is_manual_override,omitempty"`
	IsLocked         *bool      `json:"is_locked,omitempty"`
	Notes            *string    `json:"notes,omitempty"`
	Servings         **int      `json:"servings,omitempty"`
	IsCompleted      *bool      `json:"is_completed,omitempty"`
}

// WeeklyMeals represents meals organized by week with daily breakdown
type WeeklyMeals struct {
	WeekStart time.Time    `json:"week_start"`
	Monday    []MealSlot   `json:"monday"`
	Tuesday   []MealSlot   `json:"tuesday"`
	Wednesday []MealSlot   `json:"wednesday"`
	Thursday  []MealSlot   `json:"thursday"`
	Friday    []MealSlot   `json:"friday"`
	Saturday  []MealSlot   `json:"saturday"`
	Sunday    []MealSlot   `json:"sunday"`
}

// CreateMealPlanInput represents input for creating a new meal plan
type CreateMealPlanInput struct {
	Name      string    `json:"name" validate:"required,min=1,max=255"`
	StartDate time.Time `json:"start_date" validate:"required"`
	EndDate   time.Time `json:"end_date" validate:"required"`
	IsActive  bool      `json:"is_active"`
}

// MealPlanResponse represents the response structure for meal plan operations
type MealPlanResponse struct {
	MealPlan *MealPlan  `json:"meal_plan"`
	Success  bool       `json:"success"`
	Message  string     `json:"message,omitempty"`
	Error    string     `json:"error,omitempty"`
}

// MealEntryUpdateRequest represents a request to update a meal entry
type MealEntryUpdateRequest struct {
	EntryID          string     `json:"entry_id" validate:"required"`
	RecipeID         *uuid.UUID `json:"recipe_id,omitempty"`
	Notes            *string    `json:"notes,omitempty"`
	Servings         *int       `json:"servings,omitempty"`
	IsManualOverride *bool      `json:"is_manual_override,omitempty"`
	IsLocked         *bool      `json:"is_locked,omitempty"`
}

// MealReorderRequest represents a request to reorder meals
type MealReorderRequest struct {
	MealPlanID string            `json:"meal_plan_id" validate:"required"`
	NewOrder   []MealOrderEntry  `json:"new_order" validate:"required"`
}

// MealOrderEntry represents an entry in a meal reorder request
type MealOrderEntry struct {
	EntryID  string    `json:"entry_id" validate:"required"`
	Date     time.Time `json:"date" validate:"required"`
	MealType string    `json:"meal_type" validate:"required,oneof=breakfast lunch dinner snack"`
}

// RecipeCustomizations represents customizations made to a recipe
type RecipeCustomizations struct {
	ID               uuid.UUID              `json:"id" db:"id"`
	RecipeID         uuid.UUID              `json:"recipe_id" db:"recipe_id"`
	UserID           uuid.UUID              `json:"user_id" db:"user_id"`
	CustomTitle      *string                `json:"custom_title,omitempty" db:"custom_title"`
	CustomNotes      *string                `json:"custom_notes,omitempty" db:"custom_notes"`
	ServingAdjustment *float64              `json:"serving_adjustment,omitempty" db:"serving_adjustment"`
	IngredientChanges map[string]interface{} `json:"ingredient_changes,omitempty" db:"ingredient_changes"`
	StepModifications map[string]interface{} `json:"step_modifications,omitempty" db:"step_modifications"`
	Tags             []string               `json:"tags,omitempty" db:"tags"`
	CreatedAt        time.Time              `json:"created_at" db:"created_at"`
	UpdatedAt        time.Time              `json:"updated_at" db:"updated_at"`
}

// MealSlotWithRecipe represents a meal slot with full recipe details
type MealSlotWithRecipe struct {
	MealSlot
	Recipe *Recipe `json:"recipe,omitempty"`
}


// RecipeRating represents a rating for a recipe
type RecipeRating struct {
	ID       uuid.UUID `json:"id" db:"id"`
	RecipeID uuid.UUID `json:"recipe_id" db:"recipe_id"`
	UserID   uuid.UUID `json:"user_id" db:"user_id"`
	Rating   int       `json:"rating" db:"rating"` // 1-5 stars
	Review   *string   `json:"review,omitempty" db:"review"`
	CreatedAt time.Time `json:"created_at" db:"created_at"`
	UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
}

// RecipeAttribution represents recipe attribution information
type RecipeAttribution struct {
	ID                     string                 `json:"id"`
	RecipeID               string                 `json:"recipe_id"`
	OriginalContributorID  string                 `json:"original_contributor_id"`
	OriginalContributor    string                 `json:"original_contributor"`
	ImportDate             time.Time              `json:"import_date"`
	PreserveAttribution    bool                   `json:"preserve_attribution"`
	Customizations         []string               `json:"customizations"`
	CommunityMetrics       CommunityMetrics       `json:"community_metrics"`
	RecipeChain           []RecipeChainLink      `json:"recipe_chain"`
	EngagementStats       *EngagementStats       `json:"engagement_stats,omitempty"`
}

// CommunityMetrics represents community engagement metrics
type CommunityMetrics struct {
	TotalImports      int     `json:"total_imports"`
	AverageRating     float64 `json:"average_rating"`
	TotalRatings      int     `json:"total_ratings"`
	TrendingScore     float64 `json:"trending_score"`
}

// RecipeChainLink represents a link in the recipe attribution chain
type RecipeChainLink struct {
	ContributorID   string    `json:"contributor_id"`
	ContributorName string    `json:"contributor_name"`
	Contribution    string    `json:"contribution"`
	Timestamp       time.Time `json:"timestamp"`
}

// EngagementStats represents engagement statistics
type EngagementStats struct {
	Views     int `json:"views"`
	Likes     int `json:"likes"`
	Saves     int `json:"saves"`
	Shares    int `json:"shares"`
	Comments  int `json:"comments"`
}

// ImportCustomizations represents customizations during import
type ImportCustomizations struct {
	Title            *string  `json:"title,omitempty"`
	Description      *string  `json:"description,omitempty"`
	AddedIngredients []string `json:"added_ingredients,omitempty"`
	RemovedSteps     []int    `json:"removed_steps,omitempty"`
	ModifiedSteps    []string `json:"modified_steps,omitempty"`
	Tags             []string `json:"tags,omitempty"`
}