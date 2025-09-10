package rating

import (
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"imkitchen/internal/models"
	"imkitchen/internal/services"
	"imkitchen/internal/testutil"
)

func TestCommunityRatingIntegration(t *testing.T) {
	// Setup test database
	db := testutil.SetupTestDB(t)
	defer testutil.TeardownTestDB(t, db)

	// Setup services
	ratingRepo := testutil.NewTestRecipeRatingRepository(db)
	communityService := services.NewCommunityRecipeService(db, nil)
	mealPlanRepo := testutil.NewTestMealPlanRepository(db)
	recipeRepo := testutil.NewTestRecipeRepository(db)
	changeHistoryRepo := testutil.NewTestMealPlanChangeHistoryRepository(db)
	mealPlanService := services.NewMealPlanService(mealPlanRepo, recipeRepo, changeHistoryRepo, communityService)

	// Test data
	userID := uuid.New()
	recipeID1 := uuid.New()
	recipeID2 := uuid.New()

	t.Run("Community recipe rating affects recommendations", func(t *testing.T) {
		// Create test recipes
		recipe1 := &models.Recipe{
			ID:          recipeID1,
			UserID:      userID,
			Title:       "High Rated Community Recipe",
			MealType:    []string{"dinner"},
			Complexity:  "moderate",
			PrepTime:    30,
			CookTime:    45,
			IsPublic:    true,
			IsCommunity: true,
		}
		
		recipe2 := &models.Recipe{
			ID:          recipeID2,
			UserID:      userID,
			Title:       "Low Rated Community Recipe",
			MealType:    []string{"dinner"},
			Complexity:  "moderate",
			PrepTime:    25,
			CookTime:    40,
			IsPublic:    true,
			IsCommunity: true,
		}

		err := recipeRepo.Create(recipe1)
		require.NoError(t, err)
		err = recipeRepo.Create(recipe2)
		require.NoError(t, err)

		// Add multiple high ratings to recipe1
		for i := 0; i < 5; i++ {
			ratingUserID := uuid.New()
			rating := &models.RecipeRating{
				ID:               uuid.New(),
				RecipeID:         recipeID1,
				UserID:           ratingUserID,
				Rating:           5,
				Review:           "Excellent recipe!",
				Difficulty:       "as_expected",
				WouldCookAgain:   true,
				ModerationStatus: "approved",
				CreatedAt:        time.Now(),
			}
			err := ratingRepo.Create(rating)
			require.NoError(t, err)
		}

		// Add low ratings to recipe2
		for i := 0; i < 3; i++ {
			ratingUserID := uuid.New()
			rating := &models.RecipeRating{
				ID:               uuid.New(),
				RecipeID:         recipeID2,
				UserID:           ratingUserID,
				Rating:           2,
				Review:           "Not great",
				Difficulty:       "harder",
				WouldCookAgain:   false,
				ModerationStatus: "approved",
				CreatedAt:        time.Now(),
			}
			err := ratingRepo.Create(rating)
			require.NoError(t, err)
		}

		// Get community recommendations
		filters := &models.MealPlanFilters{
			MealType: &[]string{"dinner"},
		}
		
		recommendations, err := mealPlanService.GetCommunityRecommendedMeals(userID, filters)
		require.NoError(t, err)
		assert.NotEmpty(t, recommendations)

		// Verify that higher rated recipe appears first in recommendations
		foundHighRated := false
		foundLowRated := false
		highRatedIndex := -1
		lowRatedIndex := -1

		for i, recipe := range recommendations {
			if recipe.ID == recipeID1 {
				foundHighRated = true
				highRatedIndex = i
			}
			if recipe.ID == recipeID2 {
				foundLowRated = true
				lowRatedIndex = i
			}
		}

		assert.True(t, foundHighRated, "High rated recipe should be in recommendations")
		if foundLowRated {
			assert.Less(t, highRatedIndex, lowRatedIndex, "High rated recipe should appear before low rated recipe")
		}
	})

	t.Run("Rating-aware meal plan generation prioritizes community recipes", func(t *testing.T) {
		// Create meal plan preferences
		preferences := &models.MealPlanPreferences{
			WeekStartDate:         time.Now().Truncate(24 * time.Hour),
			PreferredMealTypes:    &[]string{"breakfast", "lunch", "dinner"},
			PreferredComplexity:   &"moderate",
			MaxPrepTime:           &60,
			MaxCookTime:           &90,
			IncludeSnacks:         false,
			ExcludedMeals:         nil,
		}

		// Generate rating-aware meal plan
		mealPlan, err := mealPlanService.GenerateRatingAwareMealPlan(userID, preferences)
		require.NoError(t, err)
		assert.NotNil(t, mealPlan)
		assert.Equal(t, "rating_aware_automatic", mealPlan.GenerationType)

		// Verify meal plan includes community recipes with ratings
		var meals map[string]map[string]*models.MealEntry
		err = testutil.UnmarshalJSON(mealPlan.Meals, &meals)
		require.NoError(t, err)

		foundCommunityRecipe := false
		for _, dayMeals := range meals {
			for _, mealEntry := range dayMeals {
				if mealEntry.Notes != "" && testutil.ContainsString(mealEntry.Notes, "Community rating") {
					foundCommunityRecipe = true
					break
				}
			}
			if foundCommunityRecipe {
				break
			}
		}

		assert.True(t, foundCommunityRecipe, "Meal plan should include community recipes with rating information")
	})

	t.Run("Trending recipes reflect recent rating activity", func(t *testing.T) {
		// Add recent ratings to make recipe1 trending
		for i := 0; i < 3; i++ {
			ratingUserID := uuid.New()
			rating := &models.RecipeRating{
				ID:               uuid.New(),
				RecipeID:         recipeID1,
				UserID:           ratingUserID,
				Rating:           4,
				Review:           "Good recent recipe!",
				Difficulty:       "as_expected",
				WouldCookAgain:   true,
				ModerationStatus: "approved",
				CreatedAt:        time.Now(), // Recent rating
			}
			err := ratingRepo.Create(rating)
			require.NoError(t, err)
		}

		// Get trending recipes
		trendingRecipes, err := communityService.GetTrendingRecipes(10)
		require.NoError(t, err)

		// Verify recipe1 appears in trending (it has recent activity)
		foundTrending := false
		for _, recipe := range trendingRecipes {
			if recipe.ID == recipeID1 {
				foundTrending = true
				assert.True(t, recipe.AverageRating > 4.0, "Trending recipe should have high average rating")
				assert.True(t, recipe.TotalRatings >= 5, "Trending recipe should have multiple ratings")
				break
			}
		}

		assert.True(t, foundTrending, "Recently rated recipe should appear in trending")
	})

	t.Run("Highly rated recipes meet minimum rating threshold", func(t *testing.T) {
		minRatings := 3
		highlyRated, err := communityService.GetHighlyRatedRecipes(minRatings, 10)
		require.NoError(t, err)

		for _, recipe := range highlyRated {
			assert.GreaterOrEqual(t, recipe.TotalRatings, minRatings, "Highly rated recipe should meet minimum rating count")
			assert.GreaterOrEqual(t, recipe.AverageRating, 3.0, "Highly rated recipe should have decent rating")
			
			// Verify rating distribution is populated
			assert.NotNil(t, recipe.RatingDistribution, "Rating distribution should be populated")
			assert.NotEmpty(t, recipe.RatingDistribution, "Rating distribution should have data")
		}
	})

	t.Run("Personalized recommendations consider user rating history", func(t *testing.T) {
		// Create a rating by the target user
		userRating := &models.RecipeRating{
			ID:               uuid.New(),
			RecipeID:         recipeID1,
			UserID:           userID,
			Rating:           5,
			Review:           "Love this recipe!",
			Difficulty:       "as_expected",
			WouldCookAgain:   true,
			ModerationStatus: "approved",
			CreatedAt:        time.Now(),
		}
		err := ratingRepo.Create(userRating)
		require.NoError(t, err)

		// Get personalized recommendations
		recommendations, err := communityService.GetRecommendedRecipesForUser(userID, 10)
		require.NoError(t, err)

		// Verify that already rated recipes are excluded from recommendations
		for _, recipe := range recommendations {
			assert.NotEqual(t, recipeID1, recipe.ID, "Already rated recipes should be excluded from recommendations")
		}

		// Verify recommendations match user preferences (similar meal types, complexity)
		if len(recommendations) > 0 {
			foundSimilar := false
			for _, recipe := range recommendations {
				if testutil.ContainsString(recipe.MealType, "dinner") && recipe.Complexity == "moderate" {
					foundSimilar = true
					break
				}
			}
			assert.True(t, foundSimilar, "Recommendations should include recipes similar to user's preferences")
		}
	})

	t.Run("Rating aggregation maintains data consistency", func(t *testing.T) {
		// Verify recipe1 has correct aggregated rating data
		recipe, err := recipeRepo.GetByID(recipeID1, userID)
		require.NoError(t, err)

		// Should have 9 total ratings (5 initial + 3 trending + 1 user rating)
		assert.Equal(t, 9, recipe.TotalRatings, "Recipe should have correct total rating count")
		
		// Average should be high (most ratings are 4-5 stars)
		assert.GreaterOrEqual(t, recipe.AverageRating, 4.0, "Recipe should have high average rating")

		// Verify rating distribution
		var distribution map[string]int
		err = testutil.UnmarshalJSON(recipe.RatingDistribution, &distribution)
		require.NoError(t, err)
		
		assert.Greater(t, distribution["fiveStar"], 0, "Should have 5-star ratings")
		assert.Greater(t, distribution["fourStar"], 0, "Should have 4-star ratings")
		
		// Total in distribution should match total ratings
		total := distribution["oneStar"] + distribution["twoStar"] + distribution["threeStar"] + 
				distribution["fourStar"] + distribution["fiveStar"]
		assert.Equal(t, recipe.TotalRatings, total, "Rating distribution should sum to total ratings")
	})

	t.Run("Community recipe filtering works correctly", func(t *testing.T) {
		// Test filtering by minimum rating
		filters := &services.CommunityRecipeFilters{
			MinRating:    &[]float64{4.0}[0],
			SortBy:       "rating",
		}

		response, err := communityService.GetCommunityRecipes(filters, 1, 10)
		require.NoError(t, err)
		assert.NotNil(t, response)

		// All returned recipes should meet minimum rating
		for _, recipe := range response.Recipes {
			assert.GreaterOrEqual(t, recipe.AverageRating, 4.0, "Filtered recipes should meet minimum rating")
		}

		// Test filtering by meal type
		filters = &services.CommunityRecipeFilters{
			MealTypes: []string{"dinner"},
			SortBy:    "rating",
		}

		response, err = communityService.GetCommunityRecipes(filters, 1, 10)
		require.NoError(t, err)

		// All returned recipes should be dinner recipes
		for _, recipe := range response.Recipes {
			assert.True(t, testutil.ContainsString(recipe.MealType, "dinner"), "Filtered recipes should be dinner recipes")
		}
	})
}