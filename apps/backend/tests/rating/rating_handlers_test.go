package rating

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"imkitchen/internal/handlers"
	"imkitchen/internal/services"
)

func setupTestRouter() *gin.Engine {
	gin.SetMode(gin.TestMode)
	router := gin.New()
	
	// Mock middleware to set user ID
	router.Use(func(c *gin.Context) {
		c.Set("userID", uuid.New())
		c.Next()
	})
	
	return router
}

func TestSubmitRatingHandler(t *testing.T) {
	router := setupTestRouter()
	
	// This would normally use a mock service
	mockService := &MockRatingService{}
	handler := handlers.NewRecipeRatingHandlers(mockService)
	
	router.POST("/api/v1/recipes/:id/rating", handler.SubmitRating)

	t.Run("successful rating submission", func(t *testing.T) {
		recipeID := uuid.New()
		requestBody := map[string]interface{}{
			"overallRating": 4,
			"reviewText":    "Great recipe!",
		}
		
		jsonBody, _ := json.Marshal(requestBody)
		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/"+recipeID.String()+"/rating", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		mockService.On("SubmitRating").Return(&MockRating{
			ID:            uuid.New(),
			OverallRating: 4,
			ReviewText:    "Great recipe!",
		}, nil)

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusCreated, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "Rating submitted successfully", response["message"])
	})

	t.Run("invalid recipe ID format", func(t *testing.T) {
		requestBody := map[string]interface{}{
			"overallRating": 4,
		}
		
		jsonBody, _ := json.Marshal(requestBody)
		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/invalid-id/rating", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "invalid_recipe_id", response["error"])
	})

	t.Run("invalid request body", func(t *testing.T) {
		recipeID := uuid.New()
		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/"+recipeID.String()+"/rating", bytes.NewBufferString("invalid json"))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "invalid_request", response["error"])
	})

	t.Run("duplicate rating error", func(t *testing.T) {
		recipeID := uuid.New()
		requestBody := map[string]interface{}{
			"overallRating": 4,
		}
		
		jsonBody, _ := json.Marshal(requestBody)
		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/"+recipeID.String()+"/rating", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		mockService.On("SubmitRating").Return(nil, services.ErrDuplicateRating)

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusConflict, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "duplicate_rating", response["error"])
	})
}

func TestGetRecipeRatingsHandler(t *testing.T) {
	router := setupTestRouter()
	
	mockService := &MockRatingService{}
	handler := handlers.NewRecipeRatingHandlers(mockService)
	
	router.GET("/api/v1/recipes/:id/ratings", handler.GetRecipeRatings)

	t.Run("successful ratings retrieval", func(t *testing.T) {
		recipeID := uuid.New()
		req := httptest.NewRequest(http.MethodGet, "/api/v1/recipes/"+recipeID.String()+"/ratings?page=1&limit=10", nil)
		w := httptest.NewRecorder()

		expectedResponse := &services.PaginatedRatingsResponse{
			Ratings: []*MockRating{
				{ID: uuid.New(), OverallRating: 4},
				{ID: uuid.New(), OverallRating: 5},
			},
			Pagination: &services.PaginationInfo{
				Total: 2,
				Page:  1,
				Limit: 10,
			},
		}

		mockService.On("GetRatingsByRecipe").Return(expectedResponse, nil)

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
		
		var response services.PaginatedRatingsResponse
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, 2, response.Pagination.Total)
	})

	t.Run("default pagination parameters", func(t *testing.T) {
		recipeID := uuid.New()
		req := httptest.NewRequest(http.MethodGet, "/api/v1/recipes/"+recipeID.String()+"/ratings", nil)
		w := httptest.NewRecorder()

		mockService.On("GetRatingsByRecipe").Return(&services.PaginatedRatingsResponse{
			Ratings:    []*MockRating{},
			Pagination: &services.PaginationInfo{Total: 0, Page: 1, Limit: 20},
		}, nil)

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})
}

func TestRatingValidationMiddleware(t *testing.T) {
	router := setupTestRouter()
	
	// Apply validation middleware
	router.POST("/test", middleware.RatingValidationMiddleware(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"success": true})
	})

	t.Run("valid rating data", func(t *testing.T) {
		requestBody := map[string]interface{}{
			"overallRating": 4,
			"reviewText":    "Great recipe!",
		}
		
		jsonBody, _ := json.Marshal(requestBody)
		req := httptest.NewRequest(http.MethodPost, "/test", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})

	t.Run("missing overall rating", func(t *testing.T) {
		requestBody := map[string]interface{}{
			"reviewText": "Great recipe!",
		}
		
		jsonBody, _ := json.Marshal(requestBody)
		req := httptest.NewRequest(http.MethodPost, "/test", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "validation_error", response["error"])
	})

	t.Run("invalid rating value", func(t *testing.T) {
		requestBody := map[string]interface{}{
			"overallRating": 6, // Invalid - should be 1-5
		}
		
		jsonBody, _ := json.Marshal(requestBody)
		req := httptest.NewRequest(http.MethodPost, "/test", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
	})

	t.Run("review text too long", func(t *testing.T) {
		longText := string(make([]byte, 501))
		for i := range longText {
			longText = longText[:i] + "a" + longText[i+1:]
		}
		
		requestBody := map[string]interface{}{
			"overallRating": 4,
			"reviewText":    longText,
		}
		
		jsonBody, _ := json.Marshal(requestBody)
		req := httptest.NewRequest(http.MethodPost, "/test", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
	})

	t.Run("invalid cooking context", func(t *testing.T) {
		requestBody := map[string]interface{}{
			"overallRating":  4,
			"cookingContext": "invalid_context",
		}
		
		jsonBody, _ := json.Marshal(requestBody)
		req := httptest.NewRequest(http.MethodPost, "/test", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
	})
}

func TestRateLimitingMiddleware(t *testing.T) {
	router := setupTestRouter()
	rateLimiter := middleware.NewRatingRateLimiter()
	
	// Set low limits for testing
	rateLimiter.UpdateConfig(2, time.Minute, true)
	
	router.POST("/test", rateLimiter.RatingRateLimit(), func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"success": true})
	})

	userID := uuid.New()
	
	t.Run("requests within limit", func(t *testing.T) {
		for i := 0; i < 2; i++ {
			req := httptest.NewRequest(http.MethodPost, "/test", nil)
			// Mock user ID in context - this would come from auth middleware
			w := httptest.NewRecorder()

			// Create new context with userID
			router := gin.New()
			router.Use(func(c *gin.Context) {
				c.Set("userID", userID)
				c.Next()
			})
			router.POST("/test", rateLimiter.RatingRateLimit(), func(c *gin.Context) {
				c.JSON(http.StatusOK, gin.H{"success": true})
			})

			router.ServeHTTP(w, req)
			assert.Equal(t, http.StatusOK, w.Code)
		}
	})

	t.Run("rate limit exceeded", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodPost, "/test", nil)
		w := httptest.NewRecorder()

		router := gin.New()
		router.Use(func(c *gin.Context) {
			c.Set("userID", userID)
			c.Next()
		})
		router.POST("/test", rateLimiter.RatingRateLimit(), func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{"success": true})
		})

		router.ServeHTTP(w, req)
		assert.Equal(t, http.StatusTooManyRequests, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "rate_limit_exceeded", response["error"])
	})
}

// Mock types and services for testing
type MockRating struct {
	ID            uuid.UUID `json:"id"`
	OverallRating int       `json:"overallRating"`
	ReviewText    string    `json:"reviewText"`
}

type MockRatingService struct {
	mock.Mock
}

func (m *MockRatingService) SubmitRating(userID uuid.UUID, req *services.RatingSubmissionRequest) (*MockRating, error) {
	args := m.Called()
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*MockRating), args.Error(1)
}

func (m *MockRatingService) GetRatingsByRecipe(recipeID uuid.UUID, page, limit int) (*services.PaginatedRatingsResponse, error) {
	args := m.Called()
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.PaginatedRatingsResponse), args.Error(1)
}

// Additional mock methods would be implemented here...

// Integration test helpers
func setupTestDB(t *testing.T) *sql.DB {
	// This would set up a test database
	t.Skip("Test database setup required")
	return nil
}

func cleanupTestData(db *sql.DB, userID, recipeID uuid.UUID) {
	// Clean up test data after tests
	db.Exec("DELETE FROM recipe_ratings WHERE user_id = $1 OR recipe_id = $2", userID, recipeID)
}

// Performance tests
func BenchmarkRatingSubmission(b *testing.B) {
	// Benchmark rating submission endpoint performance
	router := setupTestRouter()
	mockService := &MockRatingService{}
	handler := handlers.NewRecipeRatingHandlers(mockService)
	
	router.POST("/api/v1/recipes/:id/rating", handler.SubmitRating)
	
	recipeID := uuid.New()
	requestBody := map[string]interface{}{
		"overallRating": 4,
		"reviewText":    "Great recipe!",
	}
	jsonBody, _ := json.Marshal(requestBody)
	
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/"+recipeID.String()+"/rating", bytes.NewBuffer(jsonBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()
		
		mockService.On("SubmitRating").Return(&MockRating{}, nil)
		router.ServeHTTP(w, req)
	}
}