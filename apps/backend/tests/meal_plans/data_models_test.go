package meal_plans

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestMealPlanEntry_IsLockedField(t *testing.T) {
	// Test that MealPlanEntry includes isLocked field
	entry := models.MealPlanEntry{
		ID:               uuid.New().String(),
		MealPlanID:       uuid.New().String(),
		RecipeID:         uuid.New().String(),
		Date:             time.Now(),
		MealType:         "dinner",
		IsManualOverride: false,
		IsLocked:         true,
		IsCompleted:      false,
	}

	// Test JSON marshaling includes isLocked
	jsonData, err := json.Marshal(entry)
	require.NoError(t, err)
	
	var unmarshaled map[string]interface{}
	err = json.Unmarshal(jsonData, &unmarshaled)
	require.NoError(t, err)
	
	assert.Contains(t, unmarshaled, "isLocked")
	assert.Equal(t, true, unmarshaled["isLocked"])
}

func TestMealSlot_IsLockedField(t *testing.T) {
	// Test that MealSlot includes isLocked field
	slot := models.MealSlot{
		Day:         "monday",
		MealType:    "breakfast",
		Servings:    2,
		IsCompleted: false,
		IsLocked:    true,
	}

	// Test JSON marshaling includes isLocked
	jsonData, err := json.Marshal(slot)
	require.NoError(t, err)
	
	var unmarshaled map[string]interface{}
	err = json.Unmarshal(jsonData, &unmarshaled)
	require.NoError(t, err)
	
	assert.Contains(t, unmarshaled, "isLocked")
	assert.Equal(t, true, unmarshaled["isLocked"])
}

func TestMealPlanChangeHistory_Validation(t *testing.T) {
	// Test valid change history record
	changeHistory := models.MealPlanChangeHistory{
		ID:           uuid.New(),
		MealPlanID:   uuid.New(),
		UserID:       uuid.New(),
		ChangeType:   "substitution",
		BeforeState:  json.RawMessage(`{"entryId": "test-entry", "recipeId": "old-recipe"}`),
		AfterState:   json.RawMessage(`{"entryId": "test-entry", "recipeId": "new-recipe"}`),
		CreatedAt:    time.Now(),
	}

	// Test JSON marshaling
	jsonData, err := json.Marshal(changeHistory)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "substitution")

	// Test table name
	assert.Equal(t, "meal_plan_change_history", changeHistory.TableName())
}

func TestChangeState_Structure(t *testing.T) {
	// Test ChangeState with single entry
	singleEntryState := models.ChangeState{
		EntryID: stringPtr("test-entry-id"),
		Metadata: map[string]interface{}{
			"operation": "lock",
			"timestamp": time.Now().Unix(),
		},
	}

	jsonData, err := json.Marshal(singleEntryState)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "test-entry-id")

	// Test ChangeState with multiple entries
	multipleEntriesState := models.ChangeState{
		Entries: []models.MealChangeEntry{
			{
				ID:         "entry-1",
				MealPlanID: "plan-1",
				RecipeID:   "recipe-1",
				Date:       time.Now(),
				MealType:   "breakfast",
				IsLocked:   false,
			},
			{
				ID:         "entry-2",
				MealPlanID: "plan-1",
				RecipeID:   "recipe-2",
				Date:       time.Now(),
				MealType:   "lunch",
				IsLocked:   true,
			},
		},
	}

	jsonData, err = json.Marshal(multipleEntriesState)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "entry-1")
	assert.Contains(t, string(jsonData), "entry-2")
}


func TestChangeHistoryFilters_Validation(t *testing.T) {
	// Test change history filters
	mealPlanID := uuid.New()
	changeType := "substitution"
	limit := 10

	filters := models.ChangeHistoryFilters{
		MealPlanID: &mealPlanID,
		ChangeType: &changeType,
		StartDate:  timePtr(time.Now().AddDate(0, 0, -7)),
		EndDate:    timePtr(time.Now()),
		Limit:      &limit,
	}

	// Test that all fields are properly set
	assert.Equal(t, mealPlanID, *filters.MealPlanID)
	assert.Equal(t, "substitution", *filters.ChangeType)
	assert.Equal(t, 10, *filters.Limit)
	assert.NotNil(t, filters.StartDate)
	assert.NotNil(t, filters.EndDate)
}


// Helper functions
func timePtr(t time.Time) *time.Time {
	return &t
}