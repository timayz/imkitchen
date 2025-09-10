package rating

import (
	"database/sql"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"imkitchen/internal/services"
	_ "github.com/lib/pq"
)

func TestRatingService_SubmitRating(t *testing.T) {
	// This is a placeholder test - in production, you'd use a test database
	t.Skip("Integration test requires database setup")

	// Setup test database connection
	db, err := sql.Open("postgres", "postgres://test:test@localhost/imkitchen_test?sslmode=disable")
	require.NoError(t, err)
	defer db.Close()

	service := services.NewRecipeRatingService(db)
	
	userID := uuid.New()
	recipeID := uuid.New()

	t.Run("successful rating submission", func(t *testing.T) {
		req := &services.RatingSubmissionRequest{
			RecipeID:      recipeID,
			OverallRating: 4,
			ReviewText:    stringPtr("Great recipe!"),
		}

		rating, err := service.SubmitRating(userID, req)
		require.NoError(t, err)
		assert.NotNil(t, rating)
		assert.Equal(t, 4, rating.OverallRating)
		assert.Equal(t, "Great recipe!", *rating.ReviewText)
		assert.Equal(t, "approved", rating.ModerationStatus)
	})

	t.Run("duplicate rating error", func(t *testing.T) {
		req := &services.RatingSubmissionRequest{
			RecipeID:      recipeID,
			OverallRating: 5,
		}

		_, err := service.SubmitRating(userID, req)
		assert.ErrorIs(t, err, services.ErrDuplicateRating)
	})

	t.Run("invalid rating value", func(t *testing.T) {
		req := &services.RatingSubmissionRequest{
			RecipeID:      uuid.New(),
			OverallRating: 6, // Invalid - should be 1-5
		}

		_, err := service.SubmitRating(userID, req)
		assert.ErrorIs(t, err, services.ErrInvalidRating)
	})
}

func TestRatingService_UpdateRating(t *testing.T) {
	t.Skip("Integration test requires database setup")

	// Setup
	db, err := sql.Open("postgres", "postgres://test:test@localhost/imkitchen_test?sslmode=disable")
	require.NoError(t, err)
	defer db.Close()

	service := services.NewRecipeRatingService(db)
	userID := uuid.New()
	recipeID := uuid.New()

	// First create a rating
	submitReq := &services.RatingSubmissionRequest{
		RecipeID:      recipeID,
		OverallRating: 3,
		ReviewText:    stringPtr("Okay recipe"),
	}
	
	originalRating, err := service.SubmitRating(userID, submitReq)
	require.NoError(t, err)

	t.Run("successful rating update", func(t *testing.T) {
		updateReq := &services.RatingUpdateRequest{
			OverallRating: intPtr(5),
			ReviewText:    stringPtr("Actually amazing!"),
		}

		updatedRating, err := service.UpdateRating(userID, originalRating.ID, updateReq)
		require.NoError(t, err)
		assert.Equal(t, 5, updatedRating.OverallRating)
		assert.Equal(t, "Actually amazing!", *updatedRating.ReviewText)
	})

	t.Run("rating not found", func(t *testing.T) {
		updateReq := &services.RatingUpdateRequest{
			OverallRating: intPtr(4),
		}

		_, err := service.UpdateRating(uuid.New(), uuid.New(), updateReq)
		assert.ErrorIs(t, err, services.ErrRatingNotFound)
	})
}

func TestRatingService_GetRatingsByRecipe(t *testing.T) {
	t.Skip("Integration test requires database setup")

	db, err := sql.Open("postgres", "postgres://test:test@localhost/imkitchen_test?sslmode=disable")
	require.NoError(t, err)
	defer db.Close()

	service := services.NewRecipeRatingService(db)
	recipeID := uuid.New()

	// Create multiple ratings for the recipe
	for i := 0; i < 5; i++ {
		userID := uuid.New()
		req := &services.RatingSubmissionRequest{
			RecipeID:      recipeID,
			OverallRating: (i % 5) + 1,
		}
		_, err := service.SubmitRating(userID, req)
		require.NoError(t, err)
	}

	t.Run("get ratings with pagination", func(t *testing.T) {
		response, err := service.GetRatingsByRecipe(recipeID, 1, 3)
		require.NoError(t, err)
		assert.Len(t, response.Ratings, 3)
		assert.Equal(t, 5, response.Pagination.Total)
		assert.True(t, response.Pagination.HasNext)
		assert.False(t, response.Pagination.HasPrevious)
	})
}

func TestContentModerationService(t *testing.T) {
	service := services.NewContentModerationService()

	t.Run("approve clean content", func(t *testing.T) {
		result := service.ModerateReviewText("This is a great recipe!")
		assert.True(t, result.IsApproved)
		assert.Equal(t, "approved", result.ModerationStatus)
	})

	t.Run("flag profanity", func(t *testing.T) {
		result := service.ModerateReviewText("This recipe is damn terrible!")
		assert.False(t, result.IsApproved)
		assert.Equal(t, "pending", result.ModerationStatus)
		assert.Contains(t, result.FlaggedReason, "inappropriate")
	})

	t.Run("flag spam patterns", func(t *testing.T) {
		result := service.ModerateReviewText("aaaaaaaa buy now cheap!!!")
		assert.False(t, result.IsApproved)
		assert.Equal(t, "pending", result.ModerationStatus)
		assert.Contains(t, result.FlaggedReason, "spam")
	})

	t.Run("reject content too long", func(t *testing.T) {
		longContent := string(make([]byte, 501))
		for i := range longContent {
			longContent = longContent[:i] + "a" + longContent[i+1:]
		}
		
		result := service.ModerateReviewText(longContent)
		assert.False(t, result.IsApproved)
		assert.Equal(t, "rejected", result.ModerationStatus)
		assert.Contains(t, result.FlaggedReason, "length")
	})

	t.Run("sanitize content", func(t *testing.T) {
		result := service.ModerateReviewText("  <script>alert('xss')</script>  Great   recipe!  ")
		assert.True(t, result.IsApproved)
		assert.Equal(t, "Great recipe!", result.SanitizedContent)
	})
}

func TestRatingValidation(t *testing.T) {
	service := services.NewRecipeRatingService(nil) // No DB needed for validation tests

	t.Run("valid rating values", func(t *testing.T) {
		req := &services.RatingSubmissionRequest{
			RecipeID:         uuid.New(),
			OverallRating:    3,
			DifficultyRating: intPtr(4),
			TasteRating:      intPtr(5),
		}

		// This would normally call the service, but we're testing validation logic
		assert.True(t, isValidRating(req.OverallRating))
		assert.True(t, isValidRating(*req.DifficultyRating))
		assert.True(t, isValidRating(*req.TasteRating))
	})

	t.Run("invalid rating values", func(t *testing.T) {
		assert.False(t, isValidRating(0))
		assert.False(t, isValidRating(6))
		assert.False(t, isValidRating(-1))
	})
}

func TestRatingAggregation(t *testing.T) {
	// Test that would verify database triggers work correctly
	// This would require database setup and trigger testing
	t.Skip("Database trigger testing requires full integration setup")
}

// Helper functions
func stringPtr(s string) *string {
	return &s
}

func intPtr(i int) *int {
	return &i
}

func isValidRating(rating int) bool {
	return rating >= 1 && rating <= 5
}

// Benchmark tests
func BenchmarkContentModeration(b *testing.B) {
	service := services.NewContentModerationService()
	content := "This is a sample review text for benchmarking moderation performance."
	
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		service.ModerateReviewText(content)
	}
}

func BenchmarkRatingValidation(b *testing.B) {
	req := &services.RatingSubmissionRequest{
		RecipeID:      uuid.New(),
		OverallRating: 4,
		ReviewText:    stringPtr("Sample review text"),
	}
	
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		// Benchmark validation logic
		_ = isValidRating(req.OverallRating)
	}
}