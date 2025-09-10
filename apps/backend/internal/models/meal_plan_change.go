package models

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
)

// MealPlanChangeHistory represents a record of changes made to meal plans for undo/redo functionality
type MealPlanChangeHistory struct {
	ID           uuid.UUID       `json:"id" gorm:"type:uuid;primary_key;default:gen_random_uuid()"`
	MealPlanID   uuid.UUID       `json:"mealPlanId" gorm:"column:meal_plan_id;type:uuid;not null"`
	UserID       uuid.UUID       `json:"userId" gorm:"column:user_id;type:uuid;not null"`
	ChangeType   string          `json:"changeType" gorm:"column:change_type;not null" validate:"oneof=substitution swap reorder lock unlock"`
	BeforeState  json.RawMessage `json:"beforeState" gorm:"column:before_state;type:jsonb;not null"`
	AfterState   json.RawMessage `json:"afterState" gorm:"column:after_state;type:jsonb;not null"`
	ChangeReason *string         `json:"changeReason,omitempty" gorm:"column:change_reason"`
	CreatedAt    time.Time       `json:"createdAt" gorm:"column:created_at;default:now()"`

	// Foreign key relationships
	MealPlan *MealPlan `json:"mealPlan,omitempty" gorm:"foreignKey:MealPlanID"`
	User     *User     `json:"user,omitempty" gorm:"foreignKey:UserID"`
}

// ChangeState represents the state of a meal or group of meals before/after a change
type ChangeState struct {
	EntryID    *string           `json:"entryId,omitempty"`    // For single meal changes
	Entries    []MealChangeEntry `json:"entries,omitempty"`    // For bulk changes
	MealSlots  []MealSlot        `json:"mealSlots,omitempty"`  // For meal plan structure changes
	Metadata   map[string]interface{} `json:"metadata,omitempty"`   // Additional change context
}

// MealChangeEntry represents a meal entry in change history
type MealChangeEntry struct {
	ID               string     `json:"id"`
	MealPlanID       string     `json:"mealPlanId"`
	RecipeID         string     `json:"recipeId"`
	Date             time.Time  `json:"date"`
	MealType         string     `json:"mealType"`
	IsManualOverride bool       `json:"isManualOverride"`
	IsLocked         bool       `json:"isLocked"`
	Notes            *string    `json:"notes,omitempty"`
}

// UndoRedoOperation represents an undo/redo operation request
type UndoRedoOperation struct {
	MealPlanID  uuid.UUID `json:"mealPlanId" validate:"required"`
	OperationType string  `json:"operationType" validate:"oneof=undo redo"`
}

// ChangeHistoryFilters represents filters for change history queries
type ChangeHistoryFilters struct {
	MealPlanID  *uuid.UUID `form:"mealPlanId"`
	ChangeType  *string    `form:"changeType" validate:"omitempty,oneof=substitution swap reorder lock unlock"`
	StartDate   *time.Time `form:"startDate"`
	EndDate     *time.Time `form:"endDate"`
	Limit       *int       `form:"limit" validate:"omitempty,min=1,max=50"`
}

// ChangeHistoryResponse represents the response for change history requests
type ChangeHistoryResponse struct {
	Changes []MealPlanChangeHistory `json:"changes"`
	CanUndo bool                    `json:"canUndo"`
	CanRedo bool                    `json:"canRedo"`
	Total   int                     `json:"total"`
}

// TableName specifies the table name for GORM
func (MealPlanChangeHistory) TableName() string {
	return "meal_plan_change_history"
}