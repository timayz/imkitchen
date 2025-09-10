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

func TestMealEntryUpdateRequest_Validation(t *testing.T) {
	// Test valid meal entry update request
	updateRequest := models.MealEntryUpdateRequest{
		RecipeID:           uuid.New().String(),
		IsLocked:           boolPtr(true),
		ChangeReason:       "User preference change",
		UpdateShoppingList: boolPtr(true),
	}

	// Test JSON marshaling
	jsonData, err := json.Marshal(updateRequest)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "recipeId")
	assert.Contains(t, string(jsonData), "isLocked")
	assert.Contains(t, string(jsonData), "changeReason")
	assert.Contains(t, string(jsonData), "updateShoppingList")
}

func TestMealReorderRequest_Validation(t *testing.T) {
	// Test meal reorder request with multiple changes
	reorderRequest := models.MealReorderRequest{
		Changes: []models.MealEntryReorder{
			{
				EntryID:     uuid.New().String(),
				NewDate:     time.Now().AddDate(0, 0, 1),
				NewMealType: "dinner",
			},
			{
				EntryID:     uuid.New().String(),
				NewDate:     time.Now().AddDate(0, 0, 2),
				NewMealType: "breakfast",
			},
		},
		PreserveLocked: boolPtr(true),
	}

	// Test JSON marshaling
	jsonData, err := json.Marshal(reorderRequest)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "changes")
	assert.Contains(t, string(jsonData), "preserveLocked")
	assert.Contains(t, string(jsonData), "newMealType")
	
	// Verify changes structure
	assert.Len(t, reorderRequest.Changes, 2)
	assert.Equal(t, "dinner", reorderRequest.Changes[0].NewMealType)
	assert.Equal(t, "breakfast", reorderRequest.Changes[1].NewMealType)
}

func TestChangeHistoryFilters_Structure(t *testing.T) {
	// Test change history filters with all fields
	mealPlanID := uuid.New()
	changeType := "substitution"
	limit := 10
	startDate := time.Now().AddDate(0, 0, -7)
	endDate := time.Now()

	filters := models.ChangeHistoryFilters{
		MealPlanID: &mealPlanID,
		ChangeType: &changeType,
		StartDate:  &startDate,
		EndDate:    &endDate,
		Limit:      &limit,
	}

	// Verify all fields are properly set
	assert.Equal(t, mealPlanID, *filters.MealPlanID)
	assert.Equal(t, "substitution", *filters.ChangeType)
	assert.Equal(t, 10, *filters.Limit)
	assert.NotNil(t, filters.StartDate)
	assert.NotNil(t, filters.EndDate)
	assert.True(t, filters.EndDate.After(*filters.StartDate))
}

func TestChangeHistoryResponse_Structure(t *testing.T) {
	// Test change history response with mock data
	changes := []models.MealPlanChangeHistory{
		{
			ID:         uuid.New(),
			MealPlanID: uuid.New(),
			UserID:     uuid.New(),
			ChangeType: "substitution",
			BeforeState: json.RawMessage(`{"entryId": "old-entry", "recipeId": "old-recipe"}`),
			AfterState:  json.RawMessage(`{"entryId": "old-entry", "recipeId": "new-recipe"}`),
			CreatedAt:   time.Now(),
		},
		{
			ID:         uuid.New(),
			MealPlanID: uuid.New(),
			UserID:     uuid.New(),
			ChangeType: "lock",
			BeforeState: json.RawMessage(`{"entryId": "entry-1", "isLocked": false}`),
			AfterState:  json.RawMessage(`{"entryId": "entry-1", "isLocked": true}`),
			CreatedAt:   time.Now().Add(-1 * time.Hour),
		},
	}

	response := models.ChangeHistoryResponse{
		Changes: changes,
		CanUndo: true,
		CanRedo: false,
		Total:   len(changes),
	}

	// Test JSON marshaling
	jsonData, err := json.Marshal(response)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "changes")
	assert.Contains(t, string(jsonData), "canUndo")
	assert.Contains(t, string(jsonData), "canRedo")
	assert.Contains(t, string(jsonData), "total")

	// Verify response structure
	assert.Len(t, response.Changes, 2)
	assert.True(t, response.CanUndo)
	assert.False(t, response.CanRedo)
	assert.Equal(t, 2, response.Total)
}

func TestUndoRedoOperation_Validation(t *testing.T) {
	// Test undo operation
	undoOp := models.UndoRedoOperation{
		MealPlanID:    uuid.New(),
		OperationType: "undo",
	}

	jsonData, err := json.Marshal(undoOp)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "undo")

	// Test redo operation
	redoOp := models.UndoRedoOperation{
		MealPlanID:    uuid.New(),
		OperationType: "redo",
	}

	jsonData, err = json.Marshal(redoOp)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "redo")
}

func TestMealChangeEntry_Structure(t *testing.T) {
	// Test meal change entry for change history
	changeEntry := models.MealChangeEntry{
		ID:               uuid.New().String(),
		MealPlanID:       uuid.New().String(),
		RecipeID:         uuid.New().String(),
		Date:             time.Now(),
		MealType:         "lunch",
		IsManualOverride: true,
		IsLocked:         false,
		Notes:            stringPtr("Changed due to dietary restriction"),
	}

	// Test JSON marshaling
	jsonData, err := json.Marshal(changeEntry)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "mealPlanId")
	assert.Contains(t, string(jsonData), "recipeId")
	assert.Contains(t, string(jsonData), "mealType")
	assert.Contains(t, string(jsonData), "isManualOverride")
	assert.Contains(t, string(jsonData), "isLocked")
	assert.Contains(t, string(jsonData), "notes")

	// Verify field values
	assert.Equal(t, "lunch", changeEntry.MealType)
	assert.True(t, changeEntry.IsManualOverride)
	assert.False(t, changeEntry.IsLocked)
	assert.Equal(t, "Changed due to dietary restriction", *changeEntry.Notes)
}

func TestChangeState_ComplexStructure(t *testing.T) {
	// Test change state with multiple meal slots
	changeState := models.ChangeState{
		EntryID: stringPtr("entry-123"),
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
		MealSlots: []models.MealSlot{
			{
				Day:         "monday",
				MealType:    "dinner",
				Servings:    2,
				IsCompleted: false,
				IsLocked:    false,
			},
		},
		Metadata: map[string]interface{}{
			"operation":  "bulk_reorder",
			"timestamp":  time.Now().Unix(),
			"affected_count": 2,
		},
	}

	// Test JSON marshaling
	jsonData, err := json.Marshal(changeState)
	require.NoError(t, err)
	assert.Contains(t, string(jsonData), "entryId")
	assert.Contains(t, string(jsonData), "entries")
	assert.Contains(t, string(jsonData), "mealSlots")
	assert.Contains(t, string(jsonData), "metadata")

	// Verify structure
	assert.Equal(t, "entry-123", *changeState.EntryID)
	assert.Len(t, changeState.Entries, 2)
	assert.Len(t, changeState.MealSlots, 1)
	assert.Contains(t, changeState.Metadata, "operation")
	assert.Equal(t, "bulk_reorder", changeState.Metadata["operation"])
}

// Helper functions for tests
func boolPtr(b bool) *bool {
	return &b
}

func stringPtr(s string) *string {
	return &s
}