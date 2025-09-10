package models

import (
	"database/sql/driver"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/lib/pq"
)

// ShoppingList represents a shopping list generated from meal plans
type ShoppingList struct {
	ID           uuid.UUID  `json:"id" db:"id"`
	UserID       uuid.UUID  `json:"userId" db:"user_id"`
	MealPlanID   *uuid.UUID `json:"mealPlanId,omitempty" db:"meal_plan_id"` // Nullable for manual lists
	Name         string     `json:"name" db:"name"`
	Status       string     `json:"status" db:"status"` // active, completed, archived
	GeneratedAt  time.Time  `json:"generatedAt" db:"generated_at"`
	CompletedAt  *time.Time `json:"completedAt,omitempty" db:"completed_at"`
	CreatedAt    time.Time  `json:"createdAt" db:"created_at"`
	UpdatedAt    time.Time  `json:"updatedAt" db:"updated_at"`
}

// ShoppingItem represents an individual item in a shopping list with recipe source tracking
type ShoppingItem struct {
	ID               uuid.UUID   `json:"id" db:"id"`
	ShoppingListID   uuid.UUID   `json:"shoppingListId" db:"shopping_list_id"`
	IngredientName   string      `json:"ingredientName" db:"ingredient_name"`
	Amount           float64     `json:"amount" db:"amount"`
	Unit             string      `json:"unit" db:"unit"`
	Category         string      `json:"category" db:"category"` // produce, dairy, pantry, protein, other
	IsCompleted      bool        `json:"isCompleted" db:"is_completed"`
	Notes            *string     `json:"notes,omitempty" db:"notes"`
	RecipeSources    UUIDArray   `json:"recipeSources" db:"recipe_sources"` // Which recipes need this ingredient
	EstimatedCost    *float64    `json:"estimatedCost,omitempty" db:"estimated_cost"`
	CompletedAt      *time.Time  `json:"completedAt,omitempty" db:"completed_at"`
	CreatedAt        time.Time   `json:"createdAt" db:"created_at"`
	UpdatedAt        time.Time   `json:"updatedAt" db:"updated_at"`
}

// UUIDArray custom type for PostgreSQL UUID array support
type UUIDArray []uuid.UUID

// Scan implements the sql.Scanner interface for reading from database
func (a *UUIDArray) Scan(value interface{}) error {
	if value == nil {
		*a = UUIDArray{}
		return nil
	}

	arr, ok := value.(pq.StringArray)
	if !ok {
		return fmt.Errorf("cannot scan %T into UUIDArray", value)
	}

	result := make([]uuid.UUID, len(arr))
	for i, str := range arr {
		id, err := uuid.Parse(str)
		if err != nil {
			return err
		}
		result[i] = id
	}
	*a = result
	return nil
}

// Value implements the driver.Valuer interface for writing to database
func (a UUIDArray) Value() (driver.Value, error) {
	if len(a) == 0 {
		return pq.Array([]string{}), nil
	}

	strs := make([]string, len(a))
	for i, id := range a {
		strs[i] = id.String()
	}
	return pq.Array(strs), nil
}

// AggregatedIngredient represents an ingredient during aggregation process
type AggregatedIngredient struct {
	Name          string      `json:"name"`
	Amount        float64     `json:"amount"`
	Unit          string      `json:"unit"`
	Category      string      `json:"category"`
	RecipeSources []uuid.UUID `json:"recipeSources"`
}

// ShoppingListGenerateRequest represents the request to generate a shopping list
type ShoppingListGenerateRequest struct {
	MealPlanID    string `json:"mealPlanId" validate:"required"`
	MergeExisting bool   `json:"mergeExisting"`
}

// ShoppingListResponse represents the full shopping list response with categorized items
type ShoppingListResponse struct {
	ID                string                    `json:"id"`
	UserID            string                    `json:"userId"`
	MealPlanID        *string                   `json:"mealPlanId,omitempty"`
	Name              string                    `json:"name"`
	Status            string                    `json:"status"`
	Categories        map[string][]ShoppingItem `json:"categories"`
	TotalItems        int                       `json:"totalItems"`
	CompletedItems    int                       `json:"completedItems"`
	GeneratedAt       time.Time                 `json:"generatedAt"`
	EstimatedCost     *float64                  `json:"estimatedCost,omitempty"`
}

// ShoppingItemUpdateRequest represents a request to update a shopping item
type ShoppingItemUpdateRequest struct {
	IsCompleted bool    `json:"isCompleted"`
	Notes       *string `json:"notes,omitempty"`
}

// ShoppingListStatus constants
const (
	ShoppingListStatusActive    = "active"
	ShoppingListStatusCompleted = "completed"
	ShoppingListStatusArchived  = "archived"
)

// Category constants for grocery store sections
const (
	CategoryProduce = "produce"
	CategoryDairy   = "dairy"
	CategoryPantry  = "pantry"
	CategoryProtein = "protein"
	CategoryOther   = "other"
)

// GetValidCategories returns all valid category values
func GetValidCategories() []string {
	return []string{
		CategoryProduce,
		CategoryDairy,
		CategoryPantry,
		CategoryProtein,
		CategoryOther,
	}
}

// IsValidCategory checks if a category is valid
func IsValidCategory(category string) bool {
	validCategories := GetValidCategories()
	for _, valid := range validCategories {
		if category == valid {
			return true
		}
	}
	return false
}

// ToResponse converts a ShoppingList and items to a ShoppingListResponse
func (sl *ShoppingList) ToResponse(items []ShoppingItem) *ShoppingListResponse {
	// Organize items by category
	categories := make(map[string][]ShoppingItem)
	totalItems := len(items)
	completedItems := 0

	for _, item := range items {
		if categories[item.Category] == nil {
			categories[item.Category] = []ShoppingItem{}
		}
		categories[item.Category] = append(categories[item.Category], item)

		if item.IsCompleted {
			completedItems++
		}
	}

	var mealPlanID *string
	if sl.MealPlanID != nil {
		mealPlanIDStr := sl.MealPlanID.String()
		mealPlanID = &mealPlanIDStr
	}

	return &ShoppingListResponse{
		ID:             sl.ID.String(),
		UserID:         sl.UserID.String(),
		MealPlanID:     mealPlanID,
		Name:           sl.Name,
		Status:         sl.Status,
		Categories:     categories,
		TotalItems:     totalItems,
		CompletedItems: completedItems,
		GeneratedAt:    sl.GeneratedAt,
		EstimatedCost:  nil, // TODO: Implement cost estimation
	}
}