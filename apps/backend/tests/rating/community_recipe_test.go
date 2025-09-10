package rating

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"imkitchen/internal/handlers"
	"imkitchen/internal/services"
)

func TestCommunityRecipeHandlers_GetCommunityRecipes(t *testing.T) {
	router := setupTestRouter()
	
	mockService := &MockCommunityService{}
	handler := handlers.NewCommunityRecipeHandlers(mockService)
	
	router.GET("/api/v1/community/recipes", handler.GetCommunityRecipes)

	t.Run("successful community recipes retrieval", func(t *testing.T) {
		expectedResponse := &services.CommunityRecipeResponse{
			Recipes: []*services.CommunityRecipe{
				{
					ID:              uuid.New(),
					Title:           "Community Recipe 1",
					AverageRating:   4.5,
					TotalRatings:    10,
					RecommendationScore: 4.2,
				},
				{
					ID:              uuid.New(),
					Title:           "Community Recipe 2",
					AverageRating:   4.0,
					TotalRatings:    8,
					RecommendationScore: 3.8,
				},
			},
			Pagination: &services.PaginationInfo{
				Total: 2,
				Page:  1,
				Limit: 20,
			},
		}

		mockService.On("GetCommunityRecipes", mock.Anything, 1, 20).Return(expectedResponse, nil)

		req := httptest.NewRequest(http.MethodGet, "/api/v1/community/recipes", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
		
		var response services.CommunityRecipeResponse
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Len(t, response.Recipes, 2)
		assert.Equal(t, 2, response.Pagination.Total)
	})

	t.Run("with search filters", func(t *testing.T) {
		expectedResponse := &services.CommunityRecipeResponse{
			Recipes:    []*services.CommunityRecipe{},
			Pagination: &services.PaginationInfo{Total: 0, Page: 1, Limit: 20},
		}

		mockService.On("GetCommunityRecipes", mock.MatchedBy(func(filters *services.CommunityRecipeFilters) bool {
			return filters.SearchQuery != nil && *filters.SearchQuery == "pasta"
		}), 1, 20).Return(expectedResponse, nil)

		req := httptest.NewRequest(http.MethodGet, "/api/v1/community/recipes?search=pasta&minRating=4.0&sortBy=rating", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})

	t.Run("with pagination", func(t *testing.T) {
		expectedResponse := &services.CommunityRecipeResponse{
			Recipes:    []*services.CommunityRecipe{},
			Pagination: &services.PaginationInfo{Total: 50, Page: 2, Limit: 10},
		}

		mockService.On("GetCommunityRecipes", mock.Anything, 2, 10).Return(expectedResponse, nil)

		req := httptest.NewRequest(http.MethodGet, "/api/v1/community/recipes?page=2&limit=10", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
		
		var response services.CommunityRecipeResponse
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, 2, response.Pagination.Page)
		assert.Equal(t, 10, response.Pagination.Limit)
	})
}

func TestCommunityRecipeHandlers_GetTrendingRecipes(t *testing.T) {
	router := setupTestRouter()
	
	mockService := &MockCommunityService{}
	handler := handlers.NewCommunityRecipeHandlers(mockService)
	
	router.GET("/api/v1/community/recipes/trending", handler.GetTrendingRecipes)

	t.Run("successful trending recipes retrieval", func(t *testing.T) {
		expectedRecipes := []*services.CommunityRecipe{
			{
				ID:              uuid.New(),
				Title:           "Trending Recipe",
				AverageRating:   4.8,
				TotalRatings:    25,
				RecommendationScore: 4.5,
			},
		}

		mockService.On("GetTrendingRecipes", 20).Return(expectedRecipes, nil)

		req := httptest.NewRequest(http.MethodGet, "/api/v1/community/recipes/trending", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		recipes := response["recipes"].([]interface{})
		assert.Len(t, recipes, 1)
	})

	t.Run("with custom limit", func(t *testing.T) {
		mockService.On("GetTrendingRecipes", 5).Return([]*services.CommunityRecipe{}, nil)

		req := httptest.NewRequest(http.MethodGet, "/api/v1/community/recipes/trending?limit=5", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
	})
}

func TestCommunityRecipeHandlers_GetRecommendedRecipes(t *testing.T) {
	router := setupTestRouter()
	
	mockService := &MockCommunityService{}
	handler := handlers.NewCommunityRecipeHandlers(mockService)
	
	router.GET("/api/v1/users/me/recommendations", handler.GetRecommendedRecipes)

	t.Run("successful personalized recommendations", func(t *testing.T) {
		expectedRecipes := []*services.CommunityRecipe{
			{
				ID:              uuid.New(),
				Title:           "Recommended Recipe",
				AverageRating:   4.3,
				TotalRatings:    15,
				RecommendationScore: 4.1,
			},
		}

		mockService.On("GetRecommendedRecipesForUser", mock.AnythingOfType("uuid.UUID"), 10).Return(expectedRecipes, nil)

		req := httptest.NewRequest(http.MethodGet, "/api/v1/users/me/recommendations", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.True(t, response["personalized"].(bool))
		recipes := response["recipes"].([]interface{})
		assert.Len(t, recipes, 1)
	})

	t.Run("requires authentication", func(t *testing.T) {
		// Create router without auth middleware
		router := gin.New()
		router.GET("/api/v1/users/me/recommendations", handler.GetRecommendedRecipes)

		req := httptest.NewRequest(http.MethodGet, "/api/v1/users/me/recommendations", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusUnauthorized, w.Code)
	})
}

func TestCommunityRecipeHandlers_ImportExternalRecipe(t *testing.T) {
	router := setupTestRouter()
	
	mockService := &MockCommunityService{}
	handler := handlers.NewCommunityRecipeHandlers(mockService)
	
	router.POST("/api/v1/community/recipes/import", handler.ImportExternalRecipe)

	t.Run("successful recipe import", func(t *testing.T) {
		expectedRecipe := &services.CommunityRecipe{
			ID:              uuid.New(),
			Title:           "Imported Recipe",
			ExternalSource:  stringPtr("spoonacular"),
			AverageRating:   0.0,
			TotalRatings:    0,
			RecommendationScore: 2.5,
		}

		importReq := services.RecipeImportRequest{
			Source:      "spoonacular",
			ExternalID:  "12345",
			MakePublic:  true,
			IsCommunity: true,
		}

		mockService.On("ImportExternalRecipe", &importReq).Return(expectedRecipe, nil)

		reqBody := `{
			"source": "spoonacular",
			"externalId": "12345",
			"makePublic": true,
			"isCommunity": true
		}`

		req := httptest.NewRequest(http.MethodPost, "/api/v1/community/recipes/import", strings.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusCreated, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "Recipe imported successfully", response["message"])
	})

	t.Run("invalid source", func(t *testing.T) {
		reqBody := `{
			"source": "invalid_source",
			"externalId": "12345"
		}`

		req := httptest.NewRequest(http.MethodPost, "/api/v1/community/recipes/import", strings.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "validation_error", response["error"])
	})

	t.Run("missing required fields", func(t *testing.T) {
		reqBody := `{
			"source": "spoonacular"
		}`

		req := httptest.NewRequest(http.MethodPost, "/api/v1/community/recipes/import", strings.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
	})
}

func TestCommunityRecipeHandlers_PromoteRecipeToPublic(t *testing.T) {
	router := setupTestRouter()
	
	mockService := &MockCommunityService{}
	handler := handlers.NewCommunityRecipeHandlers(mockService)
	
	router.POST("/api/v1/recipes/:id/promote", handler.PromoteRecipeToPublic)

	t.Run("successful recipe promotion", func(t *testing.T) {
		recipeID := uuid.New()
		userID := uuid.New()
		
		expectedRecipe := &services.CommunityRecipe{
			ID:              recipeID,
			Title:           "Promoted Recipe",
			AverageRating:   0.0,
			TotalRatings:    0,
			RecommendationScore: 2.5,
		}

		mockService.On("PromoteToPublic", recipeID, userID, true).Return(expectedRecipe, nil)

		reqBody := `{"makeCommunity": true}`

		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/"+recipeID.String()+"/promote", strings.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		// Mock user ID in context
		router := gin.New()
		router.Use(func(c *gin.Context) {
			c.Set("userID", userID)
			c.Next()
		})
		router.POST("/api/v1/recipes/:id/promote", handler.PromoteRecipeToPublic)

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "Recipe promoted to public successfully", response["message"])
	})

	t.Run("recipe not found", func(t *testing.T) {
		recipeID := uuid.New()
		userID := uuid.New()

		mockService.On("PromoteToPublic", recipeID, userID, false).Return(nil, services.ErrRecipeNotFound)

		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/"+recipeID.String()+"/promote", strings.NewReader("{}"))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router := gin.New()
		router.Use(func(c *gin.Context) {
			c.Set("userID", userID)
			c.Next()
		})
		router.POST("/api/v1/recipes/:id/promote", handler.PromoteRecipeToPublic)

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusNotFound, w.Code)
	})

	t.Run("unauthorized access", func(t *testing.T) {
		recipeID := uuid.New()
		userID := uuid.New()

		mockService.On("PromoteToPublic", recipeID, userID, false).Return(nil, services.ErrUnauthorized)

		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/"+recipeID.String()+"/promote", strings.NewReader("{}"))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router := gin.New()
		router.Use(func(c *gin.Context) {
			c.Set("userID", userID)
			c.Next()
		})
		router.POST("/api/v1/recipes/:id/promote", handler.PromoteRecipeToPublic)

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusForbidden, w.Code)
	})

	t.Run("invalid recipe ID", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/recipes/invalid-id/promote", strings.NewReader("{}"))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
	})
}

func TestCommunityRecipeHandlers_SearchCommunityRecipes(t *testing.T) {
	router := setupTestRouter()
	
	mockService := &MockCommunityService{}
	handler := handlers.NewCommunityRecipeHandlers(mockService)
	
	router.GET("/api/v1/community/recipes/search", handler.SearchCommunityRecipes)

	t.Run("successful search", func(t *testing.T) {
		expectedResponse := &services.CommunityRecipeResponse{
			Recipes: []*services.CommunityRecipe{
				{
					ID:              uuid.New(),
					Title:           "Pasta Recipe",
					AverageRating:   4.2,
					TotalRatings:    12,
				},
			},
			Pagination: &services.PaginationInfo{Total: 1, Page: 1, Limit: 20},
		}

		mockService.On("GetCommunityRecipes", mock.MatchedBy(func(filters *services.CommunityRecipeFilters) bool {
			return filters.SearchQuery != nil && *filters.SearchQuery == "pasta"
		}), 1, 20).Return(expectedResponse, nil)

		req := httptest.NewRequest(http.MethodGet, "/api/v1/community/recipes/search?q=pasta", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusOK, w.Code)
		
		var response services.CommunityRecipeResponse
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Len(t, response.Recipes, 1)
	})

	t.Run("missing search query", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/community/recipes/search", nil)
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)

		assert.Equal(t, http.StatusBadRequest, w.Code)
		
		var response map[string]interface{}
		json.Unmarshal(w.Body.Bytes(), &response)
		assert.Equal(t, "validation_error", response["error"])
	})
}

// Mock community service
type MockCommunityService struct {
	mock.Mock
}

func (m *MockCommunityService) GetCommunityRecipes(filters *services.CommunityRecipeFilters, page, limit int) (*services.CommunityRecipeResponse, error) {
	args := m.Called(filters, page, limit)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.CommunityRecipeResponse), args.Error(1)
}

func (m *MockCommunityService) GetTrendingRecipes(limit int) ([]*services.CommunityRecipe, error) {
	args := m.Called(limit)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]*services.CommunityRecipe), args.Error(1)
}

func (m *MockCommunityService) GetHighlyRatedRecipes(minRatings, limit int) ([]*services.CommunityRecipe, error) {
	args := m.Called(minRatings, limit)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]*services.CommunityRecipe), args.Error(1)
}

func (m *MockCommunityService) GetRecommendedRecipesForUser(userID uuid.UUID, limit int) ([]*services.CommunityRecipe, error) {
	args := m.Called(userID, limit)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]*services.CommunityRecipe), args.Error(1)
}

func (m *MockCommunityService) ImportExternalRecipe(req *services.RecipeImportRequest) (*services.CommunityRecipe, error) {
	args := m.Called(req)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.CommunityRecipe), args.Error(1)
}

func (m *MockCommunityService) PromoteToPublic(recipeID, userID uuid.UUID, makeCommunity bool) (*services.CommunityRecipe, error) {
	args := m.Called(recipeID, userID, makeCommunity)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.CommunityRecipe), args.Error(1)
}

// Performance tests
func BenchmarkCommunityRecipeSearch(b *testing.B) {
	router := setupTestRouter()
	mockService := &MockCommunityService{}
	handler := handlers.NewCommunityRecipeHandlers(mockService)
	
	router.GET("/api/v1/community/recipes", handler.GetCommunityRecipes)
	
	expectedResponse := &services.CommunityRecipeResponse{
		Recipes:    []*services.CommunityRecipe{},
		Pagination: &services.PaginationInfo{Total: 0, Page: 1, Limit: 20},
	}
	
	mockService.On("GetCommunityRecipes", mock.Anything, mock.Anything, mock.Anything).Return(expectedResponse, nil)
	
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/community/recipes", nil)
		w := httptest.NewRecorder()
		
		router.ServeHTTP(w, req)
	}
}