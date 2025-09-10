package tests

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// Integration test for the complete "Fill My Week" workflow
// Tests the end-to-end flow from request to completed meal plan

func TestFillMyWeekWorkflow_EndToEnd(t *testing.T) {
	// Setup all services with mocks
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockMealPlanRepo := new(MockMealPlanRepository)
	mockCache := new(MockCacheService)

	// Create services
	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)
	mealPlanService := services.NewMealPlanService(mockMealPlanRepo, mockRecipeRepo)

	userID := uuid.New()
	weekStartDate := time.Now().Truncate(24 * time.Hour)

	// Step 1: Setup test data
	recipes := createTestRecipes()
	preferences := createTestUserPreferences()
	user := &models.User{
		ID:                     userID,
		PreferenceLearningData: json.RawMessage(`{}`),
	}

	// Step 2: Setup mocks for rotation service
	mockCache.On("Get", mock.AnythingOfType("string")).Return("")
	mockUserRepo.On("GetByID", userID).Return(user, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)
	mockUserRepo.On("GetUserPreferences", userID).Return(preferences, nil)

	searchResponse := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      int64(len(recipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)

	// Step 3: Generate weekly meal plan using rotation service
	startTime := time.Now()
	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)
	generationDuration := time.Since(startTime)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)
	assert.True(t, generationDuration < 2*time.Second, "Generation took %v, should be under 2 seconds", generationDuration)

	// Step 4: Create meal plan using meal plan service
	mealPlanID := uuid.New()
	expectedMealPlan := &models.MealPlan{
		ID:             mealPlanID,
		UserID:         userID,
		WeekStartDate:  weekStartDate,
		GenerationType: "automated",
		GeneratedAt:    time.Now(),
		IsActive:       true,
		Status:         "active",
	}

	mockMealPlanRepo.On("GetByWeekStart", userID, weekStartDate).Return(nil, assert.AnError) // No existing plan
	mockMealPlanRepo.On("Create", mock.AnythingOfType("*models.MealPlan")).Return(nil)

	createInput := &models.CreateMealPlanInput{
		WeekStartDate:  weekStartDate,
		GenerationType: "automated",
		Meals:          *weeklyMeals,
	}

	mealPlan, err := mealPlanService.CreateMealPlan(userID, createInput)

	assert.NoError(t, err)
	assert.NotNil(t, mealPlan)
	assert.Equal(t, userID, mealPlan.UserID)
	assert.Equal(t, "automated", mealPlan.GenerationType)

	// Step 5: Verify meal plan completeness
	var meals models.WeeklyMeals
	err = json.Unmarshal(mealPlan.Meals, &meals)
	assert.NoError(t, err)

	// Verify all days have 3 meals
	days := [][]models.MealSlot{
		meals.Monday, meals.Tuesday, meals.Wednesday,
		meals.Thursday, meals.Friday, meals.Saturday, meals.Sunday,
	}

	totalMeals := 0
	uniqueRecipes := make(map[string]bool)

	for dayIndex, dayMeals := range days {
		assert.Equal(t, 3, len(dayMeals), "Day %d should have 3 meals", dayIndex)

		for mealIndex, meal := range dayMeals {
			assert.NotNil(t, meal.RecipeID, "Meal %d on day %d should have a recipe", mealIndex, dayIndex)
			assert.NotEmpty(t, *meal.RecipeID, "Recipe ID should not be empty")

			uniqueRecipes[*meal.RecipeID] = true
			totalMeals++

			// Verify meal type is correct
			expectedMealTypes := []string{"breakfast", "lunch", "dinner"}
			assert.Equal(t, expectedMealTypes[mealIndex], meal.MealType)
		}
	}

	assert.Equal(t, 21, totalMeals, "Should have exactly 21 meals (7 days × 3 meals)")
	assert.True(t, len(uniqueRecipes) >= 15, "Should have good recipe variety (at least 15 unique recipes)")

	// Verify all mocks were called as expected
	mockRecipeRepo.AssertExpectations(t)
	mockUserRepo.AssertExpectations(t)
	mockCache.AssertExpectations(t)

	t.Logf("Fill My Week workflow completed successfully:")
	t.Logf("  Generation time: %v", generationDuration)
	t.Logf("  Total meals: %d", totalMeals)
	t.Logf("  Unique recipes: %d", len(uniqueRecipes))
	t.Logf("  Recipe variety: %.1f%%", float64(len(uniqueRecipes))/float64(totalMeals)*100)
}

func TestFillMyWeekWorkflow_WithPreferences(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()

	// Create user with specific preferences
	preferences := &models.UserPreferences{
		DietaryRestrictions:     []string{"vegetarian"},
		CookingSkillLevel:       "beginner",
		PreferredMealComplexity: "simple",
		MaxPrepTimePerMeal:      30,
		WeeklyAvailability: map[string]int{
			"monday": 30, "tuesday": 30, "wednesday": 30,
			"thursday": 30, "friday": 30, "saturday": 60, "sunday": 60,
		},
		CuisinePreferences: []string{"italian", "american"},
		AvoidIngredients:   []string{"nuts"},
	}

	recipes := createTestRecipes()

	// Setup mocks
	mockCache.On("Get", mock.AnythingOfType("string")).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: json.RawMessage(`{}`),
	}, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	// Mock search will filter recipes based on preferences
	filteredRecipes := make([]models.Recipe, 0)
	for _, recipe := range recipes {
		// Simulate filtering by complexity (beginner -> simple)
		if recipe.Complexity == "simple" && recipe.PrepTime <= 30 {
			filteredRecipes = append(filteredRecipes, recipe)
		}
	}

	searchResponse := &models.RecipeSearchResponse{
		Recipes:    filteredRecipes,
		Total:      int64(len(filteredRecipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}

	mockRecipeRepo.On("Search", userID, mock.MatchedBy(func(params *models.RecipeSearchParams) bool {
		// Verify that preferences are applied to search
		return len(params.Complexity) > 0 && params.Complexity[0] == "simple"
	})).Return(searchResponse, nil)

	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)

	// Generate meal plan with preferences
	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	if len(filteredRecipes) < 21 {
		// If we don't have enough simple recipes, the generation should fail gracefully
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "insufficient recipes")
		t.Logf("Expected failure due to insufficient simple recipes (%d available)", len(filteredRecipes))
		return
	}

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify preferences were respected
	// This is a simplified check - in reality, we'd verify recipe complexity, prep time, etc.
	totalMealsGenerated := 0
	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	for _, dayMeals := range days {
		totalMealsGenerated += len(dayMeals)
	}

	assert.Equal(t, 21, totalMealsGenerated)

	t.Logf("Preference-based generation completed:")
	t.Logf("  Available filtered recipes: %d", len(filteredRecipes))
	t.Logf("  Generated meals: %d", totalMealsGenerated)
	t.Logf("  Max prep time preference: %d minutes", preferences.MaxPrepTimePerMeal)
	t.Logf("  Complexity preference: %s", preferences.PreferredMealComplexity)
	t.Logf("  Cooking skill level: %s", preferences.CookingSkillLevel)
}

func TestFillMyWeekWorkflow_RotationTracking(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	recipes := createTestRecipes()

	// Simulate existing rotation state with some used recipes
	existingRotationState := `{
		"usedRecipes": {
			"` + recipes[0].ID.String() + `": "2025-09-01T00:00:00Z",
			"` + recipes[1].ID.String() + `": "2025-09-02T00:00:00Z"
		},
		"cycleCount": 0,
		"totalRecipesUsed": 2,
		"mealTypeHistory": {},
		"complexityHistory": []
	}`

	// Setup mocks with existing rotation state
	mockCache.On("Get", mock.AnythingOfType("string")).Return(existingRotationState)
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: json.RawMessage(`{}`),
	}, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	searchResponse := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      int64(len(recipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)

	// Generate meal plan
	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify rotation tracking - previously used recipes should be avoided
	usedRecipeIds := []string{recipes[0].ID.String(), recipes[1].ID.String()}
	
	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	foundPreviouslyUsed := 0
	totalMeals := 0

	for _, dayMeals := range days {
		for _, meal := range dayMeals {
			totalMeals++
			if meal.RecipeID != nil {
				for _, usedID := range usedRecipeIds {
					if *meal.RecipeID == usedID {
						foundPreviouslyUsed++
					}
				}
			}
		}
	}

	// Previously used recipes should be minimized (rotation algorithm should avoid them)
	usagePercentage := float64(foundPreviouslyUsed) / float64(totalMeals) * 100
	assert.True(t, usagePercentage < 20.0, 
		"Too many previously used recipes (%f%%), rotation not working properly", usagePercentage)

	t.Logf("Rotation tracking test results:")
	t.Logf("  Previously used recipes: %d", len(usedRecipeIds))
	t.Logf("  Found in new plan: %d", foundPreviouslyUsed)
	t.Logf("  Usage percentage: %.1f%%", usagePercentage)
	t.Logf("  Total meals: %d", totalMeals)
}

// Mock for MealPlanRepository used in integration test
type MockMealPlanRepository struct {
	mock.Mock
}

func (m *MockMealPlanRepository) Create(mealPlan *models.MealPlan) error {
	args := m.Called(mealPlan)
	return args.Error(0)
}

func (m *MockMealPlanRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.MealPlan, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) GetByUserID(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.MealPlan, error) {
	args := m.Called(userID, filters)
	return args.Get(0).([]models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) GetByWeekStart(userID uuid.UUID, weekStart time.Time) (*models.MealPlan, error) {
	args := m.Called(userID, weekStart)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) Update(id uuid.UUID, userID uuid.UUID, input *models.UpdateMealPlanInput) (*models.MealPlan, error) {
	args := m.Called(id, userID, input)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) UpdateMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string, input *models.UpdateMealSlotInput) (*models.MealPlan, error) {
	args := m.Called(mealPlanID, userID, day, mealType, input)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) DeleteMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string) (*models.MealPlan, error) {
	args := m.Called(mealPlanID, userID, day, mealType)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) Delete(id uuid.UUID, userID uuid.UUID) error {
	args := m.Called(id, userID)
	return args.Error(0)
}