package services

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/imkitchen/backend/internal/models"
)

func TestMealPlanCacheService_GetCachedMealPlan(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache)

	userID := uuid.New()
	preferences := &models.UserPreferences{
		DietaryRestrictions: []string{"vegetarian"},
		FamilySize:          4,
	}

	key := &MealPlanCacheKey{
		UserID:         userID,
		PreferencesHash: "test-hash",
		WeekOffset:     0,
		GenerationType: "standard",
	}

	// Test cache hit
	cachedPlan := &CachedMealPlan{
		MealPlan: &models.WeeklyMeals{
			WeekStartDate: time.Now(),
			Meals: map[string][]models.PlannedMeal{
				"monday": {{
					RecipeID: uuid.New(),
					MealType: "breakfast",
				}},
			},
		},
		GeneratedAt: time.Now(),
		TTL:         2 * time.Hour,
		Version:     "1.0",
	}

	cachedData, _ := json.Marshal(cachedPlan)
	cacheKey := service.(*mealPlanCacheService).generateCacheKey(key)
	
	mockCache.On("Get", mock.Anything, cacheKey).Return(string(cachedData), nil)

	result, err := service.GetCachedMealPlan(context.Background(), key)

	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.NotNil(t, result.MealPlan)
	assert.Equal(t, "1.0", result.Version)

	mockCache.AssertExpectations(t)
}

func TestMealPlanCacheService_CacheMealPlan(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache)

	userID := uuid.New()
	key := &MealPlanCacheKey{
		UserID:         userID,
		PreferencesHash: "test-hash",
		WeekOffset:     0,
		GenerationType: "standard",
	}

	mealPlan := &models.WeeklyMeals{
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

	ttl := 2 * time.Hour
	cacheKey := service.(*mealPlanCacheService).generateCacheKey(key)
	userCacheKey := "user_meal_plans:" + userID.String()

	mockCache.On("Set", mock.Anything, cacheKey, mock.AnythingOfType("*services.CachedMealPlan"), ttl).Return(nil)
	mockCache.On("Get", mock.Anything, userCacheKey).Return("", nil) // No existing keys
	mockCache.On("Set", mock.Anything, userCacheKey, mock.AnythingOfType("[]string"), 7*24*time.Hour).Return(nil)

	err := service.CacheMealPlan(context.Background(), key, mealPlan, constraintReport, ttl)

	assert.NoError(t, err)
	mockCache.AssertExpectations(t)
}

func TestMealPlanCacheService_InvalidateMealPlans(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache)

	userID := uuid.New()
	userCacheKey := "user_meal_plans:" + userID.String()
	
	// Mock existing cached keys
	keys := []string{"meal_plan:key1", "meal_plan:key2"}
	keysData, _ := json.Marshal(keys)

	mockCache.On("Get", mock.Anything, userCacheKey).Return(string(keysData), nil)
	mockCache.On("Delete", mock.Anything, "meal_plan:key1").Return(nil)
	mockCache.On("Delete", mock.Anything, "meal_plan:key2").Return(nil)
	mockCache.On("Delete", mock.Anything, userCacheKey).Return(nil)

	err := service.InvalidateMealPlans(context.Background(), userID)

	assert.NoError(t, err)
	mockCache.AssertExpectations(t)
}

func TestMealPlanCacheService_GetOrGenerateMealPlan(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache)

	userID := uuid.New()
	key := &MealPlanCacheKey{
		UserID:         userID,
		PreferencesHash: "test-hash",
		WeekOffset:     0,
		GenerationType: "standard",
	}

	// Test cache miss - should generate new meal plan
	cacheKey := service.(*mealPlanCacheService).generateCacheKey(key)
	mockCache.On("Get", mock.Anything, cacheKey).Return("", assert.AnError) // Cache miss

	generatedMealPlan := &models.WeeklyMeals{
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

	generator := func() (*models.WeeklyMeals, *RotationConstraintReport, error) {
		return generatedMealPlan, constraintReport, nil
	}

	result, err := service.GetOrGenerateMealPlan(context.Background(), key, generator)

	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, generatedMealPlan, result.MealPlan)
	assert.Equal(t, constraintReport, result.ConstraintReport)

	mockCache.AssertExpectations(t)
}

func TestMealPlanCacheService_CacheRecipePool(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache)

	userID := uuid.New()
	recipes := []models.Recipe{
		{
			ID:    uuid.New(),
			Title: "Test Recipe 1",
		},
		{
			ID:    uuid.New(),
			Title: "Test Recipe 2",
		},
	}

	ttl := time.Hour
	cacheKey := "recipe_pool:" + userID.String()

	mockCache.On("Set", mock.Anything, cacheKey, mock.AnythingOfType("struct { Recipes []models.Recipe \"json:\\\"recipes\\\"\"; CachedAt time.Time \"json:\\\"cachedAt\\\"\"; Count int \"json:\\\"count\\\"\" }"), ttl).Return(nil)

	err := service.CacheRecipePoolForUser(context.Background(), userID, recipes, ttl)

	assert.NoError(t, err)
	mockCache.AssertExpectations(t)
}

func TestMealPlanCacheService_GetCachedRecipePool(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache)

	userID := uuid.New()
	cacheKey := "recipe_pool:" + userID.String()

	recipes := []models.Recipe{
		{
			ID:    uuid.New(),
			Title: "Test Recipe 1",
		},
	}

	poolData := struct {
		Recipes  []models.Recipe `json:"recipes"`
		CachedAt time.Time       `json:"cachedAt"`
		Count    int             `json:"count"`
	}{
		Recipes:  recipes,
		CachedAt: time.Now(),
		Count:    1,
	}

	cachedData, _ := json.Marshal(poolData)
	mockCache.On("Get", mock.Anything, cacheKey).Return(string(cachedData), nil)

	result, err := service.GetCachedRecipePool(context.Background(), userID)

	assert.NoError(t, err)
	assert.Len(t, result, 1)
	assert.Equal(t, recipes[0].Title, result[0].Title)

	mockCache.AssertExpectations(t)
}

func TestMealPlanCacheService_GenerateCacheKey(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache).(*mealPlanCacheService)

	userID := uuid.New()
	key1 := &MealPlanCacheKey{
		UserID:            userID,
		PreferencesHash:   "hash1",
		WeeklyPatternHash: "pattern1",
		WeekOffset:        0,
		GenerationType:    "standard",
	}

	key2 := &MealPlanCacheKey{
		UserID:            userID,
		PreferencesHash:   "hash1",
		WeeklyPatternHash: "pattern1",
		WeekOffset:        1, // Different week offset
		GenerationType:    "standard",
	}

	cacheKey1 := service.generateCacheKey(key1)
	cacheKey2 := service.generateCacheKey(key2)

	// Same parameters should generate same key
	cacheKey1Duplicate := service.generateCacheKey(key1)

	assert.Equal(t, cacheKey1, cacheKey1Duplicate)
	assert.NotEqual(t, cacheKey1, cacheKey2) // Different week offset should generate different keys
	assert.Contains(t, cacheKey1, "meal_plan:")
}

func TestMealPlanCacheService_CalculateOptimalTTL(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache).(*mealPlanCacheService)

	tests := []struct {
		name           string
		generationType string
		generationTime time.Duration
		expectedMin    time.Duration
		expectedMax    time.Duration
	}{
		{
			name:           "standard generation fast",
			generationType: "standard",
			generationTime: 500 * time.Millisecond,
			expectedMin:    1 * time.Hour,
			expectedMax:    3 * time.Hour,
		},
		{
			name:           "standard generation slow",
			generationType: "standard",
			generationTime: 2 * time.Second,
			expectedMin:    5 * time.Hour,
			expectedMax:    8 * time.Hour,
		},
		{
			name:           "pattern-aware generation",
			generationType: "pattern-aware",
			generationTime: 1 * time.Second,
			expectedMin:    10 * time.Hour,
			expectedMax:    20 * time.Hour,
		},
		{
			name:           "constraint-handling generation",
			generationType: "constraint-handling",
			generationTime: 1 * time.Second,
			expectedMin:    15 * time.Hour,
			expectedMax:    25 * time.Hour,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			key := &MealPlanCacheKey{
				GenerationType: tt.generationType,
			}

			ttl := service.calculateOptimalTTL(key, tt.generationTime)

			assert.GreaterOrEqual(t, ttl, tt.expectedMin)
			assert.LessOrEqual(t, ttl, tt.expectedMax)
		})
	}
}

func TestMealPlanCacheService_PrewarmCache(t *testing.T) {
	mockCache := &MockCacheService{}
	service := NewMealPlanCacheService(mockCache)

	userID := uuid.New()
	preferences := &models.UserPreferences{
		DietaryRestrictions: []string{"vegetarian"},
		FamilySize:          4,
	}

	patterns := []models.UserWeeklyPattern{
		{
			DayOfWeek: "monday",
			MealType:  "breakfast",
			RecipeID:  &uuid.UUID{},
		},
	}

	// Mock cache misses for prewarming scenarios
	mockCache.On("Get", mock.Anything, mock.AnythingOfType("string")).Return("", assert.AnError).Maybe()

	err := service.PrewarmCache(context.Background(), userID, preferences, patterns)

	assert.NoError(t, err)
	// Note: PrewarmCache runs goroutines, so we can't easily test the cache calls
	// In a real implementation, you might want to add synchronization for testing
}