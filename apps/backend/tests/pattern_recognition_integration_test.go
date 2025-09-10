package tests

import (
	"fmt"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// TestPatternRecognitionIntegration tests AC1: Weekend vs. weekday cooking pattern recognition
func TestPatternRecognitionIntegration(t *testing.T) {
	// Setup test environment with mocks
	mockRecipeRepo := new(MockRecipeRepository)
	mockUserRepo := new(MockUserRepository)
	mockPreferenceRepo := new(MockPreferenceRepository)
	
	rotationService := services.NewRotationService(mockRecipeRepo, mockUserRepo, mockPreferenceRepo)

	// Create test user
	user := &models.User{
		ID:                      uuid.New(),
		Email:                   "pattern-test@example.com",
		CookingSkillLevel:       "intermediate",
		PreferredMealComplexity: "moderate",
		MaxCookTime:             90,
	}
	err := db.Create(user).Error
	require.NoError(t, err)

	// Create test recipes with different complexities
	simpleRecipe := &models.Recipe{
		ID:          uuid.New(),
		Title:       "Simple Pasta",
		Complexity:  "simple",
		PrepTime:    20,
		Description: "Quick weekday meal",
		MealType:    []string{"dinner"},
	}
	complexRecipe := &models.Recipe{
		ID:          uuid.New(),
		Title:       "Complex Roast",
		Complexity:  "complex",
		PrepTime:    120,
		Description: "Weekend elaborate meal",
		MealType:    []string{"dinner"},
	}
	err = db.Create([]*models.Recipe{simpleRecipe, complexRecipe}).Error
	require.NoError(t, err)

	// Create weekly patterns
	weekdayPattern := &models.UserWeeklyPattern{
		UserID:              user.ID,
		DayOfWeek:           1, // Monday
		MaxPrepTime:         45,
		PreferredComplexity: "simple",
		IsWeekendPattern:    false,
	}
	weekendPattern := &models.UserWeeklyPattern{
		UserID:              user.ID,
		DayOfWeek:           6, // Saturday
		MaxPrepTime:         120,
		PreferredComplexity: "complex",
		IsWeekendPattern:    true,
	}
	err = db.Create([]*models.UserWeeklyPattern{weekdayPattern, weekendPattern}).Error
	require.NoError(t, err)

	t.Run("Given user has weekend patterns configured", func(t *testing.T) {
		t.Run("When selecting recipe for Saturday", func(t *testing.T) {
			// Create selection criteria for Saturday
			saturday := time.Saturday
			criteria := &services.RecipeSelectionCriteria{
				UserID:   user.ID,
				MealType: []string{"dinner"},
			}
			recipes := []models.Recipe{*simpleRecipe, *complexRecipe}
			weeklyPatterns := []models.UserWeeklyPattern{*weekendPattern}
			rotationState := &services.RotationState{
				UsedRecipes: make(map[string]bool),
			}

			selectedRecipe, err := rotationService.AssignRecipeForDay(saturday, weeklyPatterns, criteria, recipes, rotationState)

			t.Run("Then complex meal should be preferred for weekend", func(t *testing.T) {
				require.NoError(t, err)
				assert.NotNil(t, selectedRecipe)
				// Weekend pattern prefers complex meals and allows longer prep time
				assert.True(t, selectedRecipe.PrepTime <= 120, "Recipe should fit weekend time limit")
				if len(recipes) > 1 {
					// Should prefer complex recipe on weekend
					assert.Equal(t, "complex", selectedRecipe.Complexity, "Should prefer complex meals on weekends")
				}
			})
		})
	})

	t.Run("Given user has weekday patterns configured", func(t *testing.T) {
		t.Run("When selecting recipe for Monday", func(t *testing.T) {
			// Create selection criteria for Monday
			monday := time.Monday
			criteria := &services.RecipeSelectionCriteria{
				UserID:   user.ID,
				MealType: []string{"dinner"},
			}
			recipes := []models.Recipe{*simpleRecipe, *complexRecipe}
			weeklyPatterns := []models.UserWeeklyPattern{*weekdayPattern}
			rotationState := &services.RotationState{
				UsedRecipes: make(map[string]bool),
			}

			selectedRecipe, err := rotationService.AssignRecipeForDay(monday, weeklyPatterns, criteria, recipes, rotationState)

			t.Run("Then simple meal should be preferred for weekday", func(t *testing.T) {
				require.NoError(t, err)
				assert.NotNil(t, selectedRecipe)
				// Weekday pattern prefers simple meals with shorter prep time
				assert.True(t, selectedRecipe.PrepTime <= 45, "Recipe should fit weekday time limit")
				// Should prefer simple recipe on weekday
				assert.Equal(t, "simple", selectedRecipe.Complexity, "Should prefer simple meals on weekdays")
			})
		})
	})

	t.Run("Pattern recognition accuracy test", func(t *testing.T) {
		// Test pattern recognition with various scenarios
		testCases := []struct {
			name           string
			day            time.Weekday
			pattern        *models.UserWeeklyPattern
			expectedMaxTime int
			expectedComplexity string
		}{
			{
				name: "Sunday weekend pattern",
				day:  time.Sunday,
				pattern: &models.UserWeeklyPattern{
					UserID:              user.ID,
					DayOfWeek:           0,
					MaxPrepTime:         90,
					PreferredComplexity: "moderate",
					IsWeekendPattern:    true,
				},
				expectedMaxTime: 90,
				expectedComplexity: "moderate",
			},
			{
				name: "Wednesday weekday pattern",
				day:  time.Wednesday,
				pattern: &models.UserWeeklyPattern{
					UserID:              user.ID,
					DayOfWeek:           3,
					MaxPrepTime:         30,
					PreferredComplexity: "simple",
					IsWeekendPattern:    false,
				},
				expectedMaxTime: 30,
				expectedComplexity: "simple",
			},
		}

		for _, tc := range testCases {
			t.Run(tc.name, func(t *testing.T) {
				// Create pattern in database
				err := db.Create(tc.pattern).Error
				require.NoError(t, err)

				// Create appropriate test recipe
				testRecipe := &models.Recipe{
					ID:          uuid.New(),
					Title:       "Test Recipe " + tc.name,
					Complexity:  tc.expectedComplexity,
					PrepTime:    tc.expectedMaxTime - 5, // Slightly under limit
					Description: "Test recipe for " + tc.name,
					MealType:    []string{"dinner"},
				}
				err = db.Create(testRecipe).Error
				require.NoError(t, err)

				criteria := &services.RecipeSelectionCriteria{
					UserID:   user.ID,
					MealType: []string{"dinner"},
				}
				weeklyPatterns := []models.UserWeeklyPattern{*tc.pattern}
				rotationState := &services.RotationState{
					UsedRecipes: make(map[string]bool),
				}

				selectedRecipe, err := rotationService.AssignRecipeForDay(tc.day, weeklyPatterns, criteria, []models.Recipe{*testRecipe}, rotationState)

				require.NoError(t, err)
				assert.NotNil(t, selectedRecipe)
				assert.True(t, selectedRecipe.PrepTime <= tc.expectedMaxTime, "Recipe should respect time constraint")
				assert.Equal(t, tc.expectedComplexity, selectedRecipe.Complexity, "Recipe should match complexity preference")

				// Cleanup
				db.Delete(tc.pattern)
				db.Delete(testRecipe)
			})
		}
	})
}

// TestPerformancePatternRecognition tests that pattern recognition meets 200ms requirement
func TestPerformancePatternRecognition(t *testing.T) {
	// Skip in short mode
	if testing.Short() {
		t.Skip("Skipping performance test in short mode")
	}

	// Setup test environment
	db := setupTestDB(t)
	defer cleanupTestDB(t, db)

	repos := createTestRepositories(db)
	rotationService := services.NewRotationService(repos.Recipe, repos.User, repos.Preference)

	// Create test data
	user := &models.User{
		ID:    uuid.New(),
		Email: "perf-test@example.com",
	}
	db.Create(user)

	// Create multiple patterns
	patterns := make([]models.UserWeeklyPattern, 7)
	for i := 0; i < 7; i++ {
		patterns[i] = models.UserWeeklyPattern{
			UserID:              user.ID,
			DayOfWeek:           i,
			MaxPrepTime:         60,
			PreferredComplexity: "moderate",
			IsWeekendPattern:    i == 0 || i == 6,
		}
	}
	db.Create(patterns)

	// Create test recipes
	recipes := make([]models.Recipe, 100)
	for i := 0; i < 100; i++ {
		recipes[i] = models.Recipe{
			ID:          uuid.New(),
			Title:       fmt.Sprintf("Recipe %d", i),
			Complexity:  []string{"simple", "moderate", "complex"}[i%3],
			PrepTime:    30 + (i%60),
			MealType:    []string{"dinner"},
		}
	}
	db.Create(recipes)

	// Test pattern recognition performance
	start := time.Now()
	
	for day := 0; day < 7; day++ {
		criteria := &services.RecipeSelectionCriteria{
			UserID:   user.ID,
			MealType: []string{"dinner"},
		}
		rotationState := &services.RotationState{
			UsedRecipes: make(map[string]bool),
		}

		_, err := rotationService.AssignRecipeForDay(time.Weekday(day), patterns, criteria, recipes, rotationState)
		require.NoError(t, err)
	}

	elapsed := time.Since(start)
	
	// Pattern recognition should complete within 200ms for 7 days
	assert.True(t, elapsed < 200*time.Millisecond, 
		"Pattern recognition took %v, should be under 200ms", elapsed)
}