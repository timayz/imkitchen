package tests

import (
	"fmt"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// TestFavoritesIntegration tests AC2: Recipe favorites with increased rotation frequency
func TestFavoritesIntegration(t *testing.T) {
	// Setup test environment
	db := setupTestDB(t)
	defer cleanupTestDB(t, db)

	repos := createTestRepositories(db)
	rotationService := services.NewRotationService(repos.Recipe, repos.User, repos.Preference)

	// Create test user
	user := &models.User{
		ID:                      uuid.New(),
		Email:                   "favorites-test@example.com",
		CookingSkillLevel:       "intermediate",
		PreferredMealComplexity: "moderate",
		MaxCookTime:             60,
	}
	err := db.Create(user).Error
	require.NoError(t, err)

	// Create test recipes
	favoriteRecipe := &models.Recipe{
		ID:          uuid.New(),
		Title:       "Favorite Pasta",
		Complexity:  "moderate",
		PrepTime:    30,
		Description: "User's favorite recipe",
		MealType:    []string{"dinner"},
	}
	regularRecipe := &models.Recipe{
		ID:          uuid.New(),
		Title:       "Regular Chicken",
		Complexity:  "moderate",
		PrepTime:    35,
		Description: "Regular recipe",
		MealType:    []string{"dinner"},
	}
	err = db.Create([]*models.Recipe{favoriteRecipe, regularRecipe}).Error
	require.NoError(t, err)

	// Create favorite relationship with 1.5x multiplier
	favorite := &models.UserRecipeFavorite{
		UserID:           user.ID,
		RecipeID:         favoriteRecipe.ID,
		WeightMultiplier: 1.5,
		FavoritedAt:      time.Now(),
	}
	err = db.Create(favorite).Error
	require.NoError(t, err)

	t.Run("Given user has marked a recipe as favorite", func(t *testing.T) {
		t.Run("When generating meal plans", func(t *testing.T) {
			// Test multiple selections to verify frequency increase
			favoriteCount := 0
			regularCount := 0
			totalSelections := 100

			for i := 0; i < totalSelections; i++ {
				criteria := &services.RecipeSelectionCriteria{
					UserID:   user.ID,
					MealType: []string{"dinner"},
				}
				recipes := []models.Recipe{*favoriteRecipe, *regularRecipe}
				rotationState := &services.RotationState{
					UsedRecipes: make(map[string]bool),
				}

				// Use favorites-weighted selection
				selectedRecipe, err := rotationService.SelectWithFavoritesWeighting(recipes, criteria, rotationState)
				require.NoError(t, err)

				if selectedRecipe.ID == favoriteRecipe.ID {
					favoriteCount++
				} else if selectedRecipe.ID == regularRecipe.ID {
					regularCount++
				}
			}

			t.Run("Then favorite recipe should appear 1.5x more frequently", func(t *testing.T) {
				// Calculate ratio
				ratio := float64(favoriteCount) / float64(regularCount)
				
				// Should be approximately 1.5x (allow some variance for randomization)
				assert.True(t, ratio >= 1.2, "Favorite recipe should appear at least 1.2x more frequently, got ratio: %.2f", ratio)
				assert.True(t, ratio <= 1.8, "Favorite recipe should not appear more than 1.8x frequently, got ratio: %.2f", ratio)
				
				t.Logf("Favorite selections: %d, Regular selections: %d, Ratio: %.2f", 
					favoriteCount, regularCount, ratio)
			})
		})
	})

	t.Run("Favorites weighting algorithm correctness", func(t *testing.T) {
		// Create multiple recipes with different favorite multipliers
		recipes := []*models.Recipe{
			{
				ID:          uuid.New(),
				Title:       "Super Favorite",
				Complexity:  "moderate",
				PrepTime:    25,
				MealType:    []string{"dinner"},
			},
			{
				ID:          uuid.New(),
				Title:       "Moderate Favorite", 
				Complexity:  "moderate",
				PrepTime:    30,
				MealType:    []string{"dinner"},
			},
			{
				ID:          uuid.New(),
				Title:       "Not Favorite",
				Complexity:  "moderate",
				PrepTime:    35,
				MealType:    []string{"dinner"},
			},
		}
		err = db.Create(recipes).Error
		require.NoError(t, err)

		// Create favorites with different multipliers
		favorites := []*models.UserRecipeFavorite{
			{
				UserID:           user.ID,
				RecipeID:         recipes[0].ID,
				WeightMultiplier: 2.0, // 2x multiplier
				FavoritedAt:      time.Now(),
			},
			{
				UserID:           user.ID,
				RecipeID:         recipes[1].ID,
				WeightMultiplier: 1.3, // 1.3x multiplier
				FavoritedAt:      time.Now(),
			},
			// recipes[2] is not a favorite (1.0x multiplier)
		}
		err = db.Create(favorites).Error
		require.NoError(t, err)

		// Test selection frequency over many iterations
		counts := make(map[uuid.UUID]int)
		totalSelections := 300

		for i := 0; i < totalSelections; i++ {
			criteria := &services.RecipeSelectionCriteria{
				UserID:   user.ID,
				MealType: []string{"dinner"},
			}
			recipeSlice := []models.Recipe{*recipes[0], *recipes[1], *recipes[2]}
			rotationState := &services.RotationState{
				UsedRecipes: make(map[string]bool),
			}

			selectedRecipe, err := rotationService.SelectWithFavoritesWeighting(recipeSlice, criteria, rotationState)
			require.NoError(t, err)
			counts[selectedRecipe.ID]++
		}

		// Verify relative frequencies match multipliers
		superFavoriteCount := counts[recipes[0].ID]
		moderateFavoriteCount := counts[recipes[1].ID]
		regularCount := counts[recipes[2].ID]

		t.Logf("Super Favorite (2.0x): %d selections", superFavoriteCount)
		t.Logf("Moderate Favorite (1.3x): %d selections", moderateFavoriteCount)
		t.Logf("Regular (1.0x): %d selections", regularCount)

		// Super favorite should appear roughly 2x more than regular
		if regularCount > 0 {
			superRatio := float64(superFavoriteCount) / float64(regularCount)
			assert.True(t, superRatio >= 1.5, "Super favorite should appear at least 1.5x more than regular")
			assert.True(t, superRatio <= 2.5, "Super favorite should not appear more than 2.5x than regular")
		}

		// Moderate favorite should appear roughly 1.3x more than regular
		if regularCount > 0 {
			moderateRatio := float64(moderateFavoriteCount) / float64(regularCount)
			assert.True(t, moderateRatio >= 1.0, "Moderate favorite should appear at least as much as regular")
			assert.True(t, moderateRatio <= 1.8, "Moderate favorite should not appear more than 1.8x than regular")
		}

		// Cleanup
		db.Delete(favorites)
		db.Delete(recipes)
	})
}

// TestFavoritesPerformance tests that favorites weighting meets 50ms requirement
func TestFavoritesPerformance(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping performance test in short mode")
	}

	// Setup test environment
	db := setupTestDB(t)
	defer cleanupTestDB(t, db)

	repos := createTestRepositories(db)
	rotationService := services.NewRotationService(repos.Recipe, repos.User, repos.Preference)

	// Create test user
	user := &models.User{
		ID:    uuid.New(),
		Email: "perf-favorites@example.com",
	}
	db.Create(user)

	// Create many recipes
	recipes := make([]models.Recipe, 200)
	for i := 0; i < 200; i++ {
		recipes[i] = models.Recipe{
			ID:          uuid.New(),
			Title:       fmt.Sprintf("Recipe %d", i),
			Complexity:  "moderate",
			PrepTime:    30,
			MealType:    []string{"dinner"},
		}
	}
	db.Create(recipes)

	// Create favorites for half the recipes
	favorites := make([]models.UserRecipeFavorite, 100)
	for i := 0; i < 100; i++ {
		favorites[i] = models.UserRecipeFavorite{
			UserID:           user.ID,
			RecipeID:         recipes[i].ID,
			WeightMultiplier: 1.5,
			FavoritedAt:      time.Now(),
		}
	}
	db.Create(favorites)

	// Test favorites weighting performance
	start := time.Now()

	// Perform multiple selections to test sustained performance
	for i := 0; i < 20; i++ {
		criteria := &services.RecipeSelectionCriteria{
			UserID:   user.ID,
			MealType: []string{"dinner"},
		}
		rotationState := &services.RotationState{
			UsedRecipes: make(map[string]bool),
		}

		_, err := rotationService.SelectWithFavoritesWeighting(recipes, criteria, rotationState)
		require.NoError(t, err)
	}

	elapsed := time.Since(start)
	averageTime := elapsed / 20

	// Favorites weighting should complete within 50ms per selection
	assert.True(t, averageTime < 50*time.Millisecond, 
		"Favorites weighting took %v per selection, should be under 50ms", averageTime)
}

// TestCombinedPatternsAndFavorites tests the integration of both pattern recognition and favorites
func TestCombinedPatternsAndFavorites(t *testing.T) {
	// Setup test environment
	db := setupTestDB(t)
	defer cleanupTestDB(t, db)

	repos := createTestRepositories(db)
	rotationService := services.NewRotationService(repos.Recipe, repos.User, repos.Preference)

	// Create test user
	user := &models.User{
		ID:    uuid.New(),
		Email: "combined-test@example.com",
	}
	err := db.Create(user).Error
	require.NoError(t, err)

	// Create recipes suitable for different patterns
	weekdayFavorite := &models.Recipe{
		ID:          uuid.New(),
		Title:       "Quick Favorite Pasta",
		Complexity:  "simple",
		PrepTime:    20,
		MealType:    []string{"dinner"},
	}
	weekendFavorite := &models.Recipe{
		ID:          uuid.New(),
		Title:       "Elaborate Favorite Roast",
		Complexity:  "complex",
		PrepTime:    90,
		MealType:    []string{"dinner"},
	}
	regularSimple := &models.Recipe{
		ID:          uuid.New(),
		Title:       "Regular Quick Meal",
		Complexity:  "simple",
		PrepTime:    25,
		MealType:    []string{"dinner"},
	}

	err = db.Create([]*models.Recipe{weekdayFavorite, weekendFavorite, regularSimple}).Error
	require.NoError(t, err)

	// Create patterns
	weekdayPattern := &models.UserWeeklyPattern{
		UserID:              user.ID,
		DayOfWeek:           1, // Monday
		MaxPrepTime:         30,
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

	// Create favorites
	favorites := []*models.UserRecipeFavorite{
		{
			UserID:           user.ID,
			RecipeID:         weekdayFavorite.ID,
			WeightMultiplier: 1.5,
			FavoritedAt:      time.Now(),
		},
		{
			UserID:           user.ID,
			RecipeID:         weekendFavorite.ID,
			WeightMultiplier: 1.5,
			FavoritedAt:      time.Now(),
		},
	}
	err = db.Create(favorites).Error
	require.NoError(t, err)

	t.Run("Given user has both patterns and favorites configured", func(t *testing.T) {
		t.Run("When selecting recipes for weekdays", func(t *testing.T) {
			criteria := &services.RecipeSelectionCriteria{
				UserID:   user.ID,
				MealType: []string{"dinner"},
			}
			recipes := []models.Recipe{*weekdayFavorite, *weekendFavorite, *regularSimple}
			weeklyPatterns := []models.UserWeeklyPattern{*weekdayPattern}
			rotationState := &services.RotationState{
				UsedRecipes: make(map[string]bool),
			}

			selectedRecipe, err := rotationService.AssignRecipeForDay(time.Monday, weeklyPatterns, criteria, recipes, rotationState)

			t.Run("Then weekday favorite should be preferred", func(t *testing.T) {
				require.NoError(t, err)
				assert.NotNil(t, selectedRecipe)
				// Should prefer simple recipes for weekday
				assert.Equal(t, "simple", selectedRecipe.Complexity)
				// Should fit weekday time constraint
				assert.True(t, selectedRecipe.PrepTime <= 30)
				// Should prefer the favorite among valid options
				assert.Equal(t, weekdayFavorite.ID, selectedRecipe.ID)
			})
		})

		t.Run("When selecting recipes for weekends", func(t *testing.T) {
			criteria := &services.RecipeSelectionCriteria{
				UserID:   user.ID,
				MealType: []string{"dinner"},
			}
			recipes := []models.Recipe{*weekdayFavorite, *weekendFavorite, *regularSimple}
			weeklyPatterns := []models.UserWeeklyPattern{*weekendPattern}
			rotationState := &services.RotationState{
				UsedRecipes: make(map[string]bool),
			}

			selectedRecipe, err := rotationService.AssignRecipeForDay(time.Saturday, weeklyPatterns, criteria, recipes, rotationState)

			t.Run("Then weekend favorite should be preferred", func(t *testing.T) {
				require.NoError(t, err)
				assert.NotNil(t, selectedRecipe)
				// Should prefer complex recipes for weekend
				assert.Equal(t, "complex", selectedRecipe.Complexity)
				// Should fit weekend time allowance
				assert.True(t, selectedRecipe.PrepTime <= 120)
				// Should prefer the favorite among valid options
				assert.Equal(t, weekendFavorite.ID, selectedRecipe.ID)
			})
		})
	})
}