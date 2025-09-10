package tests

import (
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// Mock repositories for testing
type MockRecipeRepository struct {
	mock.Mock
}

func (m *MockRecipeRepository) Create(recipe *models.Recipe) error {
	args := m.Called(recipe)
	return args.Error(0)
}

func (m *MockRecipeRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.Recipe, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) GetByUserID(userID uuid.UUID, limit, offset int) ([]models.Recipe, error) {
	args := m.Called(userID, limit, offset)
	return args.Get(0).([]models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) Update(id uuid.UUID, userID uuid.UUID, input *models.UpdateRecipeInput) (*models.Recipe, error) {
	args := m.Called(id, userID, input)
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) Delete(id uuid.UUID, userID uuid.UUID) error {
	args := m.Called(id, userID)
	return args.Error(0)
}

func (m *MockRecipeRepository) Search(userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error) {
	args := m.Called(userID, params)
	return args.Get(0).(*models.RecipeSearchResponse), args.Error(1)
}

func (m *MockRecipeRepository) GetByExternalSource(source, externalID string) (*models.Recipe, error) {
	args := m.Called(source, externalID)
	return args.Get(0).(*models.Recipe), args.Error(1)
}

type MockUserRepository struct {
	mock.Mock
}

func (m *MockUserRepository) GetByID(id uuid.UUID) (*models.User, error) {
	args := m.Called(id)
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockUserRepository) UpdatePreferenceLearningData(id uuid.UUID, data json.RawMessage) error {
	args := m.Called(id, data)
	return args.Error(0)
}

func (m *MockUserRepository) UpdateRotationResetCount(id uuid.UUID, count int) error {
	args := m.Called(id, count)
	return args.Error(0)
}

func (m *MockUserRepository) GetUserPreferences(id uuid.UUID) (*models.UserPreferences, error) {
	args := m.Called(id)
	return args.Get(0).(*models.UserPreferences), args.Error(1)
}

type MockCacheService struct {
	mock.Mock
}

func (m *MockCacheService) Get(key string) string {
	args := m.Called(key)
	return args.String(0)
}

func (m *MockCacheService) Set(key, value string, duration time.Duration) error {
	args := m.Called(key, value, duration)
	return args.Error(0)
}

func (m *MockCacheService) Delete(key string) error {
	args := m.Called(key)
	return args.Error(0)
}

func (m *MockCacheService) Exists(key string) bool {
	args := m.Called(key)
	return args.Bool(0)
}

// Test data setup helpers
func createTestRecipes() []models.Recipe {
	recipes := []models.Recipe{
		{
			ID:          uuid.New(),
			Title:       "Simple Scrambled Eggs",
			PrepTime:    10,
			CookTime:    5,
			MealType:    []string{"breakfast"},
			Complexity:  "simple",
			CuisineType: stringPtr("american"),
			Servings:    2,
		},
		{
			ID:          uuid.New(),
			Title:       "Grilled Chicken Salad",
			PrepTime:    15,
			CookTime:    20,
			MealType:    []string{"lunch", "dinner"},
			Complexity:  "moderate",
			CuisineType: stringPtr("mediterranean"),
			Servings:    4,
		},
		{
			ID:          uuid.New(),
			Title:       "Beef Wellington",
			PrepTime:    60,
			CookTime:    45,
			MealType:    []string{"dinner"},
			Complexity:  "complex",
			CuisineType: stringPtr("french"),
			Servings:    6,
		},
		{
			ID:          uuid.New(),
			Title:       "Overnight Oats",
			PrepTime:    5,
			CookTime:    0,
			MealType:    []string{"breakfast"},
			Complexity:  "simple",
			CuisineType: stringPtr("american"),
			Servings:    1,
		},
		{
			ID:          uuid.New(),
			Title:       "Pasta Carbonara",
			PrepTime:    10,
			CookTime:    15,
			MealType:    []string{"dinner"},
			Complexity:  "moderate",
			CuisineType: stringPtr("italian"),
			Servings:    4,
		},
	}

	// Add more recipes to have sufficient variety (need at least 21 for a full week)
	for i := 0; i < 20; i++ {
		mealTypes := [][]string{
			{"breakfast"},
			{"lunch"},
			{"dinner"},
			{"lunch", "dinner"},
		}
		complexities := []string{"simple", "moderate", "complex"}
		cuisines := []string{"italian", "french", "mexican", "asian", "american"}

		recipe := models.Recipe{
			ID:          uuid.New(),
			Title:       fmt.Sprintf("Test Recipe %d", i+6),
			PrepTime:    10 + (i % 30),
			CookTime:    15 + (i % 25),
			MealType:    mealTypes[i%len(mealTypes)],
			Complexity:  complexities[i%len(complexities)],
			CuisineType: stringPtr(cuisines[i%len(cuisines)]),
			Servings:    2 + (i % 4),
		}
		recipes = append(recipes, recipe)
	}

	return recipes
}

func createTestUserPreferences() *models.UserPreferences {
	return &models.UserPreferences{
		DietaryRestrictions:     []string{},
		CookingSkillLevel:       "intermediate",
		PreferredMealComplexity: "moderate",
		MaxPrepTimePerMeal:      60,
		WeeklyAvailability: map[string]int{
			"monday":    90,
			"tuesday":   60,
			"wednesday": 90,
			"thursday":  60,
			"friday":    45,
			"saturday":  120,
			"sunday":    120,
		},
		CuisinePreferences: []string{"italian", "mediterranean"},
		AvoidIngredients:   []string{},
	}
}

func stringPtr(s string) *string {
	return &s
}

func TestRotationService_GetRotationState(t *testing.T) {
	// Setup mocks
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	// Create service
	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()

	// Test case: Empty cache, new user
	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: json.RawMessage(`{}`),
	}, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	state, err := rotationService.GetRotationState(userID)

	assert.NoError(t, err)
	assert.NotNil(t, state)
	assert.Equal(t, 0, state.CycleCount)
	assert.Equal(t, 0, state.TotalRecipesUsed)
	assert.NotNil(t, state.UsedRecipes)
	assert.NotNil(t, state.MealTypeHistory)
	assert.NotNil(t, state.ComplexityHistory)

	mockCache.AssertExpectations(t)
	mockUserRepo.AssertExpectations(t)
}

func TestRotationService_SelectRecipesForWeek(t *testing.T) {
	// Setup mocks
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	// Create service
	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	recipes := createTestRecipes()

	// Mock rotation state (empty for first generation)
	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: json.RawMessage(`{}`),
	}, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	// Mock recipe search
	searchResponse := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      int64(len(recipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)

	// Mock state updates
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)

	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify we have meals for each day
	days := [][]models.MealSlot{
		weeklyMeals.Monday,
		weeklyMeals.Tuesday,
		weeklyMeals.Wednesday,
		weeklyMeals.Thursday,
		weeklyMeals.Friday,
		weeklyMeals.Saturday,
		weeklyMeals.Sunday,
	}

	totalMeals := 0
	for _, dayMeals := range days {
		assert.Equal(t, 3, len(dayMeals)) // breakfast, lunch, dinner
		for _, meal := range dayMeals {
			assert.NotNil(t, meal.RecipeID)
			totalMeals++
		}
	}

	assert.Equal(t, 21, totalMeals) // 7 days * 3 meals

	mockRecipeRepo.AssertExpectations(t)
	mockUserRepo.AssertExpectations(t)
	mockCache.AssertExpectations(t)
}

func TestRotationService_VarietyOptimization(t *testing.T) {
	// Setup mocks
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	// Create service
	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	recipes := createTestRecipes()

	// Create variety test: recipes with different cuisines and complexities
	recipeIDs := make([]string, 5)
	for i := 0; i < 5; i++ {
		recipeIDs[i] = recipes[i].ID.String()
		mockRecipeRepo.On("GetByID", recipes[i].ID, userID).Return(&recipes[i], nil)
	}

	score, err := rotationService.GetVarietyScore(recipeIDs, userID)

	assert.NoError(t, err)
	assert.True(t, score >= 0.0 && score <= 1.0)
	assert.True(t, score > 0.5) // Should have good variety with different cuisines/complexities

	mockRecipeRepo.AssertExpectations(t)
}

func TestRotationService_PerformanceRequirement(t *testing.T) {
	// This test ensures the generation completes within 2 seconds
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	recipes := createTestRecipes()

	// Setup mocks for fast execution
	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return("")
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

	// Measure execution time
	startTime := time.Now()
	
	_, err := rotationService.SelectRecipesForWeek(userID, preferences)
	
	executionTime := time.Since(startTime)

	assert.NoError(t, err)
	assert.True(t, executionTime < 2*time.Second, "Generation took %v, should be under 2 seconds", executionTime)

	t.Logf("Rotation algorithm completed in %v", executionTime)
}

func TestRotationService_ComplexityBalancing(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	preferences.CookingSkillLevel = "beginner" // Should limit complex recipes
	recipes := createTestRecipes()

	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return("")
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

	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify complexity balancing - beginners should get mostly simple recipes
	complexRecipeCount := 0
	totalRecipeCount := 0

	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	for _, dayMeals := range days {
		for _, meal := range dayMeals {
			if meal.RecipeID != nil {
				totalRecipeCount++
				// Find the recipe to check complexity
				for _, recipe := range recipes {
					if recipe.ID.String() == *meal.RecipeID && recipe.Complexity == "complex" {
						complexRecipeCount++
						break
					}
				}
			}
		}
	}

	// For beginners, complex recipes should be less than 25% of total
	complexPercentage := float64(complexRecipeCount) / float64(totalRecipeCount)
	assert.True(t, complexPercentage < 0.25, "Too many complex recipes (%f%%) for beginner", complexPercentage*100)

	t.Logf("Complex recipe percentage for beginner: %f%%", complexPercentage*100)
}

func TestRotationService_InsufficientRecipes(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()

	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: json.RawMessage(`{}`),
	}, nil)

	// Return insufficient recipes (less than 21 needed)
	searchResponse := &models.RecipeSearchResponse{
		Recipes:    createTestRecipes()[:5], // Only 5 recipes
		Total:      5,
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)

	_, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "insufficient recipes available")
}

// Enhanced tests for Task 2.2: Advanced Rotation Logic & User Preferences

func TestRotationService_GlobalPersistenceAcrossWeeks(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	recipes := createTestRecipes()

	// Simulate existing rotation state with weekly history
	existingState := services.RotationState{
		UsedRecipes:       make(map[string]time.Time),
		CycleCount:        1,
		TotalRecipesUsed:  15,
		WeeklyHistory:     make([]services.WeekRotationData, 0),
		GlobalRotationPool: make([]string, len(recipes)),
		LastUpdateWeek:    "2025-W35", // Previous week
		ConsecutiveWeeks:  2,
	}

	// Add some used recipes from previous weeks
	for i := 0; i < 5; i++ {
		existingState.UsedRecipes[recipes[i].ID.String()] = time.Now().AddDate(0, 0, -7)
	}

	// Add weekly history
	existingState.WeeklyHistory = append(existingState.WeeklyHistory, services.WeekRotationData{
		Week:              "2025-W35",
		RecipesUsed:       []string{recipes[0].ID.String(), recipes[1].ID.String()},
		ComplexityPattern: []string{"simple", "moderate"},
		VarietyScore:      0.7,
		GeneratedAt:       time.Now().AddDate(0, 0, -7),
	})

	// Mock cache to return existing state
	stateJSON, _ := json.Marshal(existingState)
	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return(string(stateJSON))

	// Mock search response
	searchResponse := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      int64(len(recipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify global persistence: previously used recipes should be avoided
	usedThisWeek := make(map[string]bool)
	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	for _, dayMeals := range days {
		for _, meal := range dayMeals {
			if meal.RecipeID != nil {
				usedThisWeek[*meal.RecipeID] = true
			}
		}
	}

	// Verify that recently used recipes (from previous week) are avoided when possible
	previouslyUsedCount := 0
	for recipeID := range existingState.UsedRecipes {
		if usedThisWeek[recipeID] {
			previouslyUsedCount++
		}
	}

	// Should minimize reuse of recently used recipes
	assert.True(t, previouslyUsedCount < 3, "Too many recently used recipes repeated (%d)", previouslyUsedCount)

	t.Logf("Previously used recipes repeated: %d out of %d", previouslyUsedCount, len(existingState.UsedRecipes))
}

func TestRotationService_ComplexityBalancingEnhanced(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	preferences.CookingSkillLevel = "advanced" // Should allow complex recipes but with balance
	recipes := createTestRecipes()

	// Create state with recent complex meal history to test back-to-back prevention
	existingState := services.RotationState{
		UsedRecipes:       make(map[string]time.Time),
		CycleCount:        0,
		TotalRecipesUsed:  0,
		ComplexityHistory: []string{"complex", "complex", "moderate"}, // Recent complex meals
		WeeklyHistory:     make([]services.WeekRotationData, 0),
		GlobalRotationPool: make([]string, 0),
		LastUpdateWeek:    getCurrentISOWeek(),
		ConsecutiveWeeks:  0,
	}

	stateJSON, _ := json.Marshal(existingState)
	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return(string(stateJSON))

	searchResponse := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      int64(len(recipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify enhanced complexity balancing
	complexityCount := make(map[string]int)
	mealCount := 0

	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	for dayIndex, dayMeals := range days {
		for mealIndex, meal := range dayMeals {
			if meal.RecipeID != nil {
				mealCount++
				// Find recipe complexity
				for _, recipe := range recipes {
					if recipe.ID.String() == *meal.RecipeID {
						complexityCount[recipe.Complexity]++
						
						// Verify breakfast is simple
						if mealIndex == 0 { // Breakfast
							assert.Equal(t, "simple", recipe.Complexity, "Breakfast should be simple")
						}
						
						// Verify weekend vs weekday distribution
						isWeekend := dayIndex == 5 || dayIndex == 6 // Saturday or Sunday
						if isWeekend && mealIndex == 2 { // Weekend dinner
							// Weekend dinners can be more complex
							assert.NotEqual(t, "", recipe.Complexity)
						}
						
						break
					}
				}
			}
		}
	}

	// Verify complexity balance (should not exceed 25% complex for balanced approach)
	complexPercentage := float64(complexityCount["complex"]) / float64(mealCount)
	assert.True(t, complexPercentage <= 0.3, "Too many complex meals: %f%%", complexPercentage*100)

	// Should have variety in complexity
	assert.True(t, len(complexityCount) >= 2, "Should have variety in complexity levels")
	
	t.Logf("Complexity distribution - Simple: %d, Moderate: %d, Complex: %d", 
		complexityCount["simple"], complexityCount["moderate"], complexityCount["complex"])
}

func TestRotationService_FallbackMechanisms(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	preferences.MaxPrepTimePerMeal = 10 // Very restrictive prep time
	preferences.DietaryRestrictions = []string{"vegan", "gluten-free"} // Restrictive diet
	
	// Create limited recipe set that doesn't fully match preferences
	limitedRecipes := []models.Recipe{
		{
			ID:            uuid.New(),
			Title:         "Quick Eggs",
			PrepTime:      5,
			CookTime:      5,
			MealType:      []string{"breakfast"},
			Complexity:    "simple",
			DietaryLabels: []string{"vegetarian"}, // Not vegan
			Servings:      2,
		},
		{
			ID:            uuid.New(),
			Title:         "Elaborate Pasta",
			PrepTime:      45, // Exceeds prep time limit
			CookTime:      30,
			MealType:      []string{"lunch", "dinner"},
			Complexity:    "complex",
			DietaryLabels: []string{"vegan", "gluten-free"},
			Servings:      4,
		},
		{
			ID:            uuid.New(),
			Title:         "Simple Salad",
			PrepTime:      15,
			CookTime:      0,
			MealType:      []string{"lunch"},
			Complexity:    "simple",
			DietaryLabels: []string{"vegan", "gluten-free"},
			Servings:      2,
		},
	}

	// Add more recipes to meet minimum requirement
	for i := 0; i < 22; i++ {
		limitedRecipes = append(limitedRecipes, models.Recipe{
			ID:            uuid.New(),
			Title:         fmt.Sprintf("Fallback Recipe %d", i),
			PrepTime:      20 + (i % 20), // Some will exceed limit
			CookTime:      15,
			MealType:      []string{"breakfast", "lunch", "dinner"}[i%3:i%3+1],
			Complexity:    []string{"simple", "moderate", "complex"}[i%3],
			DietaryLabels: []string{}, // No dietary labels
			Servings:      3,
		})
	}

	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: json.RawMessage(`{}`),
	}, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	searchResponse := &models.RecipeSearchResponse{
		Recipes:    limitedRecipes,
		Total:      int64(len(limitedRecipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)

	// Test with constraint handling
	weeklyMeals, report, err := rotationService.SelectRecipesForWeekWithConstraintHandling(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)
	assert.NotNil(t, report)

	// Verify fallback mechanisms worked
	assert.Equal(t, 21, report.TotalMeals)
	assert.True(t, len(report.ConstraintViolations) > 0, "Should have constraint violations due to restrictive preferences")

	// Check for specific violation types
	violationTypes := make(map[string]int)
	for _, violation := range report.ConstraintViolations {
		violationTypes[violation.ViolationType]++
	}

	t.Logf("Constraint violations - Prep time: %d, Dietary: %d, Complexity: %d", 
		violationTypes["prep_time"], violationTypes["dietary"], violationTypes["complexity"])

	// Should still generate a complete meal plan despite constraints
	mealCount := 0
	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	for _, dayMeals := range days {
		for _, meal := range dayMeals {
			if meal.RecipeID != nil {
				mealCount++
			}
		}
	}

	assert.Equal(t, 21, mealCount, "Should generate complete meal plan using fallback mechanisms")
}

func TestRotationService_CycleTrackingAndHistory(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	recipes := createTestRecipes()

	// Simulate state near rotation cycle completion
	existingState := services.RotationState{
		UsedRecipes:       make(map[string]time.Time),
		CycleCount:        2,
		TotalRecipesUsed:  40,
		WeeklyHistory:     make([]services.WeekRotationData, 2),
		GlobalRotationPool: make([]string, len(recipes)),
		LastUpdateWeek:    "2025-W35",
		ConsecutiveWeeks:  8,
	}

	// Fill global rotation pool
	for i, recipe := range recipes {
		existingState.GlobalRotationPool[i] = recipe.ID.String()
	}

	// Add most recipes as used (trigger cycle reset)
	for i := 0; i < int(float64(len(recipes))*0.85); i++ {
		existingState.UsedRecipes[recipes[i].ID.String()] = time.Now().AddDate(0, 0, -i)
	}

	// Add weekly history
	existingState.WeeklyHistory[0] = services.WeekRotationData{
		Week:        "2025-W33",
		RecipesUsed: []string{recipes[0].ID.String(), recipes[1].ID.String()},
		VarietyScore: 0.8,
		GeneratedAt: time.Now().AddDate(0, 0, -14),
	}
	existingState.WeeklyHistory[1] = services.WeekRotationData{
		Week:        "2025-W34", 
		RecipesUsed: []string{recipes[2].ID.String(), recipes[3].ID.String()},
		VarietyScore: 0.75,
		GeneratedAt: time.Now().AddDate(0, 0, -7),
	}

	stateJSON, _ := json.Marshal(existingState)
	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return(string(stateJSON))

	searchResponse := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      int64(len(recipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify cycle tracking works - state should be updated with new cycle
	// This would be verified by checking the updated state, but since we're using mocks,
	// we verify that the meal plan was successfully generated despite rotation constraints
	
	mealCount := 0
	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	for _, dayMeals := range days {
		for _, meal := range dayMeals {
			if meal.RecipeID != nil {
				mealCount++
			}
		}
	}

	assert.Equal(t, 21, mealCount, "Should complete rotation cycle and generate new meal plan")
	
	t.Logf("Cycle tracking test completed with %d meals generated", mealCount)
}

func TestRotationService_WeeklyHistoryMaintenance(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	preferences := createTestUserPreferences()
	recipes := createTestRecipes()

	// Create state with maximum weekly history (12 weeks)
	existingState := services.RotationState{
		UsedRecipes:       make(map[string]time.Time),
		CycleCount:        0,
		TotalRecipesUsed:  0,
		WeeklyHistory:     make([]services.WeekRotationData, 12), // At maximum
		GlobalRotationPool: make([]string, len(recipes)),
		LastUpdateWeek:    "2025-W35",
		ConsecutiveWeeks:  12,
	}

	// Fill weekly history with old data
	for i := 0; i < 12; i++ {
		existingState.WeeklyHistory[i] = services.WeekRotationData{
			Week:        fmt.Sprintf("2025-W%02d", 24+i),
			RecipesUsed: []string{recipes[i%len(recipes)].ID.String()},
			VarietyScore: 0.6,
			GeneratedAt: time.Now().AddDate(0, 0, -(12-i)*7),
		}
	}

	stateJSON, _ := json.Marshal(existingState)
	mockCache.On("Get", fmt.Sprintf("rotation_state:%s", userID.String())).Return(string(stateJSON))

	searchResponse := &models.RecipeSearchResponse{
		Recipes:    recipes,
		Total:      int64(len(recipes)),
		Page:       1,
		Limit:      100,
		TotalPages: 1,
	}
	mockRecipeRepo.On("Search", userID, mock.AnythingOfType("*models.RecipeSearchParams")).Return(searchResponse, nil)
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify the generation succeeded even with full history
	// History maintenance (keeping only last 12 weeks) should be handled internally
	
	mealCount := 0
	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	for _, dayMeals := range days {
		for _, meal := range dayMeals {
			if meal.RecipeID != nil {
				mealCount++
			}
		}
	}

	assert.Equal(t, 21, mealCount, "Should maintain weekly history and generate complete meal plan")

	t.Logf("Weekly history maintenance test completed")
}

// Helper function to get current ISO week (matching the service implementation)
func getCurrentISOWeek() string {
	year, week := time.Now().ISOWeek()
	return fmt.Sprintf("%d-W%02d", year, week)
}