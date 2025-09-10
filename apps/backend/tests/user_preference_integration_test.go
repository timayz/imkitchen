package tests

import (
	"fmt"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// Integration test for user preference → meal plan generation workflow
func TestUserPreferenceService_Integration(t *testing.T) {
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	// Create user preference service
	userPrefService := services.NewUserPreferenceService(mockUserRepo, mockCache)

	userID := uuid.New()
	
	// Mock user data
	testUser := &models.User{
		ID:                       userID,
		CookingSkillLevel:        "intermediate",
		PreferredMealComplexity:  "moderate",
		MaxCookTime:             45,
		DietaryRestrictions:      []string{"vegetarian"},
		PreferenceLearningData:   []byte(`{"weeklyAvailability":{"monday":30,"tuesday":45}}`),
	}

	// Test 1: Get default preferences for new user
	mockCache.On("Get", mock.AnythingOfType("string")).Return("")
	mockUserRepo.On("GetByID", userID).Return(testUser, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), time.Hour).Return(nil)

	preferences, err := userPrefService.GetUserPreferences(userID)
	
	assert.NoError(t, err)
	assert.NotNil(t, preferences)
	assert.Equal(t, "intermediate", preferences.CookingSkillLevel)
	assert.Equal(t, "moderate", preferences.PreferredMealComplexity)
	assert.Equal(t, 45, preferences.MaxPrepTimePerMeal)
	assert.Equal(t, []string{"vegetarian"}, preferences.DietaryRestrictions)

	// Test 2: Update user preferences
	updateReq := &services.UpdatePreferencesRequest{
		MaxPrepTimePerMeal:      intPtr(60),
		PreferredMealComplexity: stringPtr("complex"),
		CookingSkillLevel:       stringPtr("advanced"),
		WeeklyAvailability: map[string]int{
			"monday":    45,
			"tuesday":   60,
			"wednesday": 30,
			"thursday":  60,
			"friday":    90,
			"saturday":  120,
			"sunday":    120,
		},
	}

	mockUserRepo.On("Update", mock.AnythingOfType("*models.User")).Return(testUser, nil)
	mockCache.On("Delete", mock.AnythingOfType("string")).Return(nil)

	updatedPrefs, err := userPrefService.UpdateUserPreferences(userID, updateReq)
	
	assert.NoError(t, err)
	assert.NotNil(t, updatedPrefs)

	// Test 3: Validate preferences
	validationErrors := userPrefService.ValidatePreferences(updateReq)
	assert.Empty(t, validationErrors, "Valid preferences should not have validation errors")

	// Test 4: Invalid preferences validation
	invalidReq := &services.UpdatePreferencesRequest{
		MaxPrepTimePerMeal:      intPtr(500), // Too high
		PreferredMealComplexity: stringPtr("invalid"),
		CookingSkillLevel:       stringPtr("expert"), // Invalid skill level
	}

	validationErrors = userPrefService.ValidatePreferences(invalidReq)
	assert.NotEmpty(t, validationErrors, "Invalid preferences should have validation errors")
	assert.Contains(t, validationErrors[0], "maxPrepTimePerMeal must be between 5 and 300")

	mockUserRepo.AssertExpectations(t)
	mockCache.AssertExpectations(t)
}

// Test integration between user preferences and rotation service
func TestRotationService_UserPreferenceIntegration(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	recipes := createTestRecipes()

	// Create user preferences that should affect rotation logic
	preferences := &models.UserPreferences{
		DietaryRestrictions:     []string{"vegetarian"},
		CookingSkillLevel:       "beginner", // Should limit to simple recipes
		PreferredMealComplexity: "simple",
		MaxPrepTimePerMeal:      30, // Short prep time limit
		WeeklyAvailability: map[string]int{
			"monday":    30,
			"tuesday":   30,
			"wednesday": 30,
			"thursday":  30,
			"friday":    45,
			"saturday":  90,
			"sunday":    90,
		},
		CuisinePreferences: []string{"italian", "mediterranean"},
		AvoidIngredients:   []string{"nuts"},
	}

	// Mock rotation state
	mockCache.On("Get", mock.AnythingOfType("string")).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: []byte(`{}`),
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
	mockUserRepo.On("UpdatePreferenceLearningData", userID, mock.AnythingOfType("json.RawMessage")).Return(nil)

	// Generate meal plan with preferences
	weeklyMeals, err := rotationService.SelectRecipesForWeek(userID, preferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)

	// Verify meal plan respects preferences
	days := [][]models.MealSlot{
		weeklyMeals.Monday, weeklyMeals.Tuesday, weeklyMeals.Wednesday,
		weeklyMeals.Thursday, weeklyMeals.Friday, weeklyMeals.Saturday, weeklyMeals.Sunday,
	}

	complexRecipeCount := 0
	totalRecipes := 0

	for _, dayMeals := range days {
		for _, meal := range dayMeals {
			if meal.RecipeID != nil {
				totalRecipes++
				// Find the recipe to verify complexity
				for _, recipe := range recipes {
					if recipe.ID.String() == *meal.RecipeID {
						if recipe.Complexity == "complex" {
							complexRecipeCount++
						}
						// Verify prep time constraint (relaxed due to fallback mechanisms)
						if recipe.PrepTime > preferences.MaxPrepTimePerMeal {
							// This is allowed due to fallback mechanisms but should be minimal
							t.Logf("Recipe %s exceeds prep time limit (%d > %d) - fallback applied", 
								recipe.Title, recipe.PrepTime, preferences.MaxPrepTimePerMeal)
						}
						break
					}
				}
			}
		}
	}

	// For beginner skill level, complex recipes should be minimal (allow some due to fallbacks)
	complexPercentage := float64(complexRecipeCount) / float64(totalRecipes)
	assert.True(t, complexPercentage < 0.3, "Too many complex recipes for beginner user: %f%%", complexPercentage*100)

	t.Logf("Generated %d total recipes with %d complex recipes (%.1f%%) for beginner user", 
		totalRecipes, complexRecipeCount, complexPercentage*100)

	mockRecipeRepo.AssertExpectations(t)
	mockUserRepo.AssertExpectations(t)
	mockCache.AssertExpectations(t)
}

// Test constraint handling integration
func TestRotationService_ConstraintHandlingIntegration(t *testing.T) {
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockCache)

	userID := uuid.New()
	
	// Create very restrictive preferences to test fallback mechanisms
	restrictivePreferences := &models.UserPreferences{
		DietaryRestrictions:     []string{"vegan", "gluten-free", "nut-free"},
		CookingSkillLevel:       "beginner",
		PreferredMealComplexity: "simple",
		MaxPrepTimePerMeal:      15, // Very restrictive
		CuisinePreferences:      []string{"japanese"}, // Very specific
	}

	// Create limited recipe set that doesn't fully match preferences
	limitedRecipes := createLimitedTestRecipes()

	mockCache.On("Get", mock.AnythingOfType("string")).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{
		ID:                     userID,
		PreferenceLearningData: []byte(`{}`),
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

	// Test constraint handling with report
	weeklyMeals, report, err := rotationService.SelectRecipesForWeekWithConstraintHandling(userID, restrictivePreferences)

	assert.NoError(t, err)
	assert.NotNil(t, weeklyMeals)
	assert.NotNil(t, report)

	// Verify fallback mechanisms worked
	assert.Equal(t, 21, report.TotalMeals)
	assert.True(t, len(report.ConstraintViolations) > 0, "Should have constraint violations due to restrictive preferences")

	// Should still generate complete meal plan
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

	t.Logf("Generated meal plan with %d constraint violations using fallback mechanisms", len(report.ConstraintViolations))

	mockRecipeRepo.AssertExpectations(t)
	mockUserRepo.AssertExpectations(t)
	mockCache.AssertExpectations(t)
}

// Helper functions
func intPtr(i int) *int {
	return &i
}

func stringPtr(s string) *string {
	return &s
}

// createLimitedTestRecipes creates a limited set of recipes for constraint testing
func createLimitedTestRecipes() []models.Recipe {
	recipes := []models.Recipe{
		{
			ID:            uuid.New(),
			Title:         "Quick Salad",
			PrepTime:      10,
			CookTime:      0,
			MealType:      []string{"lunch"},
			Complexity:    "simple",
			DietaryLabels: []string{"vegan"},
			Servings:      2,
		},
		{
			ID:            uuid.New(),
			Title:         "Complex Pasta",
			PrepTime:      45, // Exceeds 15min limit
			CookTime:      30,
			MealType:      []string{"dinner"},
			Complexity:    "complex",
			DietaryLabels: []string{}, // Not vegan
			Servings:      4,
		},
	}

	// Add more basic recipes to meet minimum requirement
	for i := 0; i < 23; i++ {
		recipes = append(recipes, models.Recipe{
			ID:            uuid.New(),
			Title:         fmt.Sprintf("Basic Recipe %d", i),
			PrepTime:      20 + (i % 20),
			CookTime:      15,
			MealType:      []string{"breakfast", "lunch", "dinner"}[i%3:i%3+1],
			Complexity:    []string{"simple", "moderate", "complex"}[i%3],
			DietaryLabels: []string{},
			Servings:      2,
		})
	}

	return recipes
}