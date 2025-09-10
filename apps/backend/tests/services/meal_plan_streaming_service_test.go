package services

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/imkitchen/backend/internal/models"
)

// Mock services for testing
type MockOptimizedRotationService struct {
	mock.Mock
}

func (m *MockOptimizedRotationService) GenerateOptimizedMealPlan(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) (*models.WeeklyMeals, *RotationConstraintReport, error) {
	args := m.Called(ctx, userID, preferences, patterns)
	return args.Get(0).(*models.WeeklyMeals), args.Get(1).(*RotationConstraintReport), args.Error(2)
}

type MockMealPlanService struct {
	mock.Mock
}

func (m *MockMealPlanService) GenerateMealPlan(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) (*models.WeeklyMeals, error) {
	args := m.Called(ctx, userID, preferences, patterns)
	return args.Get(0).(*models.WeeklyMeals), args.Error(1)
}

func TestStreamingMealPlanService_GenerateWithTimeout_Success(t *testing.T) {
	mockOptimizedService := &MockOptimizedRotationService{}
	mockFallbackService := &MockMealPlanService{}

	service := NewStreamingMealPlanService(mockOptimizedService, mockFallbackService)

	userID := uuid.New()
	preferences := &models.UserPreferences{
		DietaryRestrictions: []string{"vegetarian"},
		FamilySize:          4,
	}

	expectedMealPlan := &models.WeeklyMeals{
		WeekStartDate: time.Now(),
		Meals: map[string][]models.PlannedMeal{
			"monday": {{
				RecipeID: uuid.New(),
				MealType: "breakfast",
			}},
		},
	}

	constraintReport := &RotationConstraintReport{
		TotalRecipesAnalyzed: 10,
		ConstraintsViolated:  0,
	}

	mockOptimizedService.On("GenerateOptimizedMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return(expectedMealPlan, constraintReport, nil)

	result, err := service.GenerateWithTimeout(context.Background(), userID, preferences, 5*time.Second)

	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, expectedMealPlan, result)

	mockOptimizedService.AssertExpectations(t)
}

func TestStreamingMealPlanService_GenerateWithTimeout_FallbackToStandard(t *testing.T) {
	mockOptimizedService := &MockOptimizedRotationService{}
	mockFallbackService := &MockMealPlanService{}

	service := NewStreamingMealPlanService(mockOptimizedService, mockFallbackService)

	userID := uuid.New()
	preferences := &models.UserPreferences{
		DietaryRestrictions: []string{"vegetarian"},
		FamilySize:          4,
	}

	expectedMealPlan := &models.WeeklyMeals{
		WeekStartDate: time.Now(),
		Meals: map[string][]models.PlannedMeal{
			"monday": {{
				RecipeID: uuid.New(),
				MealType: "breakfast",
			}},
		},
	}

	// Mock optimized service failure
	mockOptimizedService.On("GenerateOptimizedMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return((*models.WeeklyMeals)(nil), (*RotationConstraintReport)(nil), assert.AnError)

	// Mock fallback service success
	mockFallbackService.On("GenerateMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return(expectedMealPlan, nil)

	result, err := service.GenerateWithTimeout(context.Background(), userID, preferences, 5*time.Second)

	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, expectedMealPlan, result)

	mockOptimizedService.AssertExpectations(t)
	mockFallbackService.AssertExpectations(t)
}

func TestStreamingMealPlanService_GenerateWithTimeout_TimeoutHandling(t *testing.T) {
	mockOptimizedService := &MockOptimizedRotationService{}
	mockFallbackService := &MockMealPlanService{}

	service := NewStreamingMealPlanService(mockOptimizedService, mockFallbackService)

	userID := uuid.New()
	preferences := &models.UserPreferences{
		DietaryRestrictions: []string{"vegetarian"},
		FamilySize:          4,
	}

	// Mock services that will timeout (simulate slow response)
	mockOptimizedService.On("GenerateOptimizedMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return(func(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) (*models.WeeklyMeals, *RotationConstraintReport, error) {
		// Simulate slow operation
		select {
		case <-time.After(10 * time.Second): // Longer than timeout
			return &models.WeeklyMeals{}, &RotationConstraintReport{}, nil
		case <-ctx.Done():
			return nil, nil, ctx.Err()
		}
	})

	mockFallbackService.On("GenerateMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return(func(ctx context.Context, userID uuid.UUID, preferences *models.UserPreferences, patterns []models.UserWeeklyPattern) (*models.WeeklyMeals, error) {
		// Also simulate slow operation for fallback
		select {
		case <-time.After(10 * time.Second):
			return &models.WeeklyMeals{}, nil
		case <-ctx.Done():
			return nil, ctx.Err()
		}
	})

	start := time.Now()
	result, err := service.GenerateWithTimeout(context.Background(), userID, preferences, 100*time.Millisecond) // Very short timeout
	duration := time.Since(start)

	// Should timeout and return fallback meal plan
	assert.NoError(t, err) // Service provides fallback even on timeout
	assert.NotNil(t, result)
	assert.Less(t, duration, 5*time.Second) // Should not wait for the full 10 seconds
}

func TestStreamingMealPlanService_GenerateWithProgress(t *testing.T) {
	mockOptimizedService := &MockOptimizedRotationService{}
	mockFallbackService := &MockMealPlanService{}

	service := NewStreamingMealPlanService(mockOptimizedService, mockFallbackService)

	userID := uuid.New()
	preferences := &models.UserPreferences{
		DietaryRestrictions: []string{"vegetarian"},
		FamilySize:          4,
	}

	expectedMealPlan := &models.WeeklyMeals{
		WeekStartDate: time.Now(),
		Meals: map[string][]models.PlannedMeal{
			"monday": {{
				RecipeID: uuid.New(),
				MealType: "breakfast",
			}},
		},
	}

	constraintReport := &RotationConstraintReport{
		TotalRecipesAnalyzed: 10,
		ConstraintsViolated:  0,
	}

	mockOptimizedService.On("GenerateOptimizedMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return(expectedMealPlan, constraintReport, nil)

	req := &StreamingMealPlanRequest{
		UserID:      userID,
		Preferences: preferences,
		Timeout:     5 * time.Second,
	}

	progressChan, err := service.GenerateWithProgress(context.Background(), req)

	assert.NoError(t, err)
	assert.NotNil(t, progressChan)

	// Collect progress updates
	var updates []*MealPlanProgressUpdate
	for update := range progressChan {
		updates = append(updates, update)
		if update.Status == "completed" {
			break
		}
	}

	// Verify we received progress updates
	assert.NotEmpty(t, updates)
	
	// Verify final update contains the meal plan
	finalUpdate := updates[len(updates)-1]
	assert.Equal(t, "completed", finalUpdate.Status)
	assert.NotNil(t, finalUpdate.MealPlan)
	assert.Equal(t, expectedMealPlan, finalUpdate.MealPlan)

	mockOptimizedService.AssertExpectations(t)
}

func TestStreamingMealPlanService_GenerateWithProgress_FallbackScenario(t *testing.T) {
	mockOptimizedService := &MockOptimizedRotationService{}
	mockFallbackService := &MockMealPlanService{}

	service := NewStreamingMealPlanService(mockOptimizedService, mockFallbackService)

	userID := uuid.New()
	preferences := &models.UserPreferences{
		DietaryRestrictions: []string{"vegetarian"},
		FamilySize:          4,
	}

	fallbackMealPlan := &models.WeeklyMeals{
		WeekStartDate: time.Now(),
		Meals: map[string][]models.PlannedMeal{
			"monday": {{
				RecipeID: uuid.New(),
				MealType: "breakfast",
			}},
		},
	}

	// Mock optimized service failure
	mockOptimizedService.On("GenerateOptimizedMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return((*models.WeeklyMeals)(nil), (*RotationConstraintReport)(nil), assert.AnError)

	// Mock fallback service success
	mockFallbackService.On("GenerateMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return(fallbackMealPlan, nil)

	req := &StreamingMealPlanRequest{
		UserID:      userID,
		Preferences: preferences,
		Timeout:     5 * time.Second,
	}

	progressChan, err := service.GenerateWithProgress(context.Background(), req)

	assert.NoError(t, err)

	// Collect progress updates
	var updates []*MealPlanProgressUpdate
	var foundFallback bool
	for update := range progressChan {
		updates = append(updates, update)
		if update.Status == "fallback" || (update.Message != "" && contains(update.Message, "fallback")) {
			foundFallback = true
		}
		if update.Status == "completed" {
			break
		}
	}

	// Verify we received progress updates and fallback was used
	assert.NotEmpty(t, updates)
	assert.True(t, foundFallback, "Should have indicated fallback was used")
	
	// Verify final update contains the fallback meal plan
	finalUpdate := updates[len(updates)-1]
	assert.Equal(t, "completed", finalUpdate.Status)
	assert.NotNil(t, finalUpdate.MealPlan)

	mockOptimizedService.AssertExpectations(t)
	mockFallbackService.AssertExpectations(t)
}

func TestStreamingMealPlanService_Performance(t *testing.T) {
	mockOptimizedService := &MockOptimizedRotationService{}
	mockFallbackService := &MockMealPlanService{}

	service := NewStreamingMealPlanService(mockOptimizedService, mockFallbackService)

	userID := uuid.New()
	preferences := &models.UserPreferences{
		DietaryRestrictions: []string{"vegetarian"},
		FamilySize:          4,
	}

	expectedMealPlan := &models.WeeklyMeals{
		WeekStartDate: time.Now(),
		Meals: map[string][]models.PlannedMeal{
			"monday": {{RecipeID: uuid.New(), MealType: "breakfast"}},
			"tuesday": {{RecipeID: uuid.New(), MealType: "lunch"}},
			"wednesday": {{RecipeID: uuid.New(), MealType: "dinner"}},
		},
	}

	constraintReport := &RotationConstraintReport{
		TotalRecipesAnalyzed: 100,
		ConstraintsViolated:  0,
	}

	// Mock fast generation
	mockOptimizedService.On("GenerateOptimizedMealPlan", mock.Anything, userID, preferences, mock.AnythingOfType("[]models.UserWeeklyPattern")).Return(expectedMealPlan, constraintReport, nil)

	start := time.Now()
	result, err := service.GenerateWithTimeout(context.Background(), userID, preferences, 2*time.Second)
	duration := time.Since(start)

	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Less(t, duration, 2*time.Second, "Should complete within timeout")

	mockOptimizedService.AssertExpectations(t)
}

// Helper function for string contains check
func contains(s, substr string) bool {
	return len(s) >= len(substr) && s[:len(substr)] == substr || 
		   (len(s) > len(substr) && containsHelper(s, substr))
}

func containsHelper(s, substr string) bool {
	for i := 1; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}