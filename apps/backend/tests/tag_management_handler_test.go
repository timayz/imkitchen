package tests

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/services"
)

// Mock TagManagementService
type MockTagManagementService struct {
	mock.Mock
}

func (m *MockTagManagementService) GetTagSuggestions(userID, query, recipeID string, exclude []string, limit int) ([]handlers.TagSuggestion, error) {
	args := m.Called(userID, query, recipeID, exclude, limit)
	return args.Get(0).([]handlers.TagSuggestion), args.Error(1)
}

func (m *MockTagManagementService) GetPopularTags(userID string, limit int, categoryFilter, timePeriod string) ([]handlers.PopularTag, error) {
	args := m.Called(userID, limit, categoryFilter, timePeriod)
	return args.Get(0).([]handlers.PopularTag), args.Error(1)
}

func (m *MockTagManagementService) ValidateTags(userID string, tags []string) ([]string, []services.InvalidTagResult, error) {
	args := m.Called(userID, tags)
	return args.Get(0).([]string), args.Get(1).([]services.InvalidTagResult), args.Error(2)
}

func (m *MockTagManagementService) UpdateRecipeTags(userID, recipeID string, tags []string, action string) ([]string, error) {
	args := m.Called(userID, recipeID, tags, action)
	return args.Get(0).([]string), args.Error(1)
}

func (m *MockTagManagementService) GetRecipeTags(userID, recipeID string) ([]string, []handlers.CommunityTag, map[string]handlers.TagStat, error) {
	args := m.Called(userID, recipeID)
	return args.Get(0).([]string), args.Get(1).([]handlers.CommunityTag), args.Get(2).(map[string]handlers.TagStat), args.Error(3)
}

func (m *MockTagManagementService) VoteOnTag(userID, recipeID, tag, action string) (int, bool, error) {
	args := m.Called(userID, recipeID, tag, action)
	return args.Get(0).(int), args.Get(1).(bool), args.Error(2)
}

func setupTagManagementRouter() (*gin.Engine, *MockTagManagementService) {
	gin.SetMode(gin.TestMode)
	router := gin.New()
	
	mockService := &MockTagManagementService{}
	handler := handlers.NewTagManagementHandler(mockService)
	
	// Add auth middleware mock
	router.Use(func(c *gin.Context) {
		c.Set("user_id", "test-user-id")
		c.Next()
	})
	
	v1 := router.Group("/api/v1")
	handler.RegisterRoutes(v1)
	
	return router, mockService
}

func TestGetTagSuggestions(t *testing.T) {
	router, mockService := setupTagManagementRouter()
	
	expectedSuggestions := []handlers.TagSuggestion{
		{Tag: "vegetarian", Confidence: 0.9, UsageCount: 150, Category: "dietary"},
		{Tag: "vegan", Confidence: 0.8, UsageCount: 100, Category: "dietary"},
	}
	
	mockService.On("GetTagSuggestions", "test-user-id", "veg", "recipe-123", []string{}, 10).
		Return(expectedSuggestions, nil)
	
	requestBody := map[string]interface{}{
		"query":     "veg",
		"recipe_id": "recipe-123",
		"exclude":   []string{},
		"limit":     10,
	}
	
	body, _ := json.Marshal(requestBody)
	req := httptest.NewRequest("GET", "/api/v1/tags/suggestions", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusOK, w.Code)
	
	var response handlers.TagSuggestionsResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Len(t, response.Suggestions, 2)
	assert.Equal(t, "vegetarian", response.Suggestions[0].Tag)
	
	mockService.AssertExpectations(t)
}

func TestGetPopularTags(t *testing.T) {
	router, mockService := setupTagManagementRouter()
	
	expectedTags := []handlers.PopularTag{
		{Tag: "quick", UsageCount: 200, Category: "time", TrendingUp: true},
		{Tag: "healthy", UsageCount: 180, Category: "style", TrendingUp: false},
	}
	
	mockService.On("GetPopularTags", "test-user-id", 20, "", "week").
		Return(expectedTags, nil)
	
	req := httptest.NewRequest("GET", "/api/v1/tags/popular?limit=20&period=week", nil)
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusOK, w.Code)
	
	var response handlers.PopularTagsResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Len(t, response.Tags, 2)
	assert.Equal(t, "quick", response.Tags[0].Tag)
	assert.True(t, response.Tags[0].TrendingUp)
	
	mockService.AssertExpectations(t)
}

func TestValidateTags(t *testing.T) {
	router, mockService := setupTagManagementRouter()
	
	validTags := []string{"healthy", "quick"}
	invalidTags := []services.InvalidTagResult{
		{Tag: "spam", Reason: "Contains banned words"},
	}
	
	mockService.On("ValidateTags", "test-user-id", []string{"healthy", "quick", "spam"}).
		Return(validTags, invalidTags, nil)
	
	requestBody := map[string]interface{}{
		"tags": []string{"healthy", "quick", "spam"},
	}
	
	body, _ := json.Marshal(requestBody)
	req := httptest.NewRequest("POST", "/api/v1/tags/validate", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusOK, w.Code)
	
	var response handlers.ValidateTagsResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Len(t, response.ValidTags, 2)
	assert.Len(t, response.InvalidTags, 1)
	assert.Equal(t, "spam", response.InvalidTags[0].Tag)
	
	mockService.AssertExpectations(t)
}

func TestUpdateRecipeTags(t *testing.T) {
	router, mockService := setupTagManagementRouter()
	
	updatedTags := []string{"healthy", "quick", "vegetarian"}
	
	mockService.On("UpdateRecipeTags", "test-user-id", "recipe-123", []string{"vegetarian"}, "add").
		Return(updatedTags, nil)
	
	requestBody := map[string]interface{}{
		"tags":   []string{"vegetarian"},
		"action": "add",
	}
	
	body, _ := json.Marshal(requestBody)
	req := httptest.NewRequest("PUT", "/api/v1/recipes/recipe-123/tags", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusOK, w.Code)
	
	var response handlers.UpdateRecipeTagsResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, "recipe-123", response.RecipeID)
	assert.Len(t, response.UpdatedTags, 3)
	
	mockService.AssertExpectations(t)
}

func TestGetRecipeTags(t *testing.T) {
	router, mockService := setupTagManagementRouter()
	
	userTags := []string{"healthy", "quick"}
	communityTags := []handlers.CommunityTag{
		{Tag: "vegetarian", VoteCount: 5, UserVoted: false, Confidence: 0.8},
	}
	tagStats := map[string]handlers.TagStat{
		"healthy": {UsageCount: 100, Trending: true, Category: "style", Confidence: 0.9},
	}
	
	mockService.On("GetRecipeTags", "test-user-id", "recipe-123").
		Return(userTags, communityTags, tagStats, nil)
	
	req := httptest.NewRequest("GET", "/api/v1/recipes/recipe-123/tags", nil)
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusOK, w.Code)
	
	var response handlers.GetRecipeTagsResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, "recipe-123", response.RecipeID)
	assert.Len(t, response.UserTags, 2)
	assert.Len(t, response.CommunityTags, 1)
	assert.Len(t, response.TagStats, 1)
	
	mockService.AssertExpectations(t)
}

func TestVoteOnTag(t *testing.T) {
	router, mockService := setupTagManagementRouter()
	
	mockService.On("VoteOnTag", "test-user-id", "recipe-123", "vegetarian", "upvote").
		Return(6, true, nil)
	
	requestBody := map[string]interface{}{
		"tag":    "vegetarian",
		"action": "upvote",
	}
	
	body, _ := json.Marshal(requestBody)
	req := httptest.NewRequest("POST", "/api/v1/recipes/recipe-123/tags/vote", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusOK, w.Code)
	
	var response handlers.VoteOnTagResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, "vegetarian", response.Tag)
	assert.Equal(t, 6, response.VoteCount)
	assert.True(t, response.UserVoted)
	
	mockService.AssertExpectations(t)
}

func TestValidateTagsWithInvalidInput(t *testing.T) {
	router, _ := setupTagManagementRouter()
	
	// Test with too many tags
	requestBody := map[string]interface{}{
		"tags": make([]string, 15), // More than max of 10
	}
	
	body, _ := json.Marshal(requestBody)
	req := httptest.NewRequest("POST", "/api/v1/tags/validate", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusBadRequest, w.Code)
}

func TestUpdateRecipeTagsWithInvalidAction(t *testing.T) {
	router, _ := setupTagManagementRouter()
	
	requestBody := map[string]interface{}{
		"tags":   []string{"test"},
		"action": "invalid-action",
	}
	
	body, _ := json.Marshal(requestBody)
	req := httptest.NewRequest("PUT", "/api/v1/recipes/recipe-123/tags", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusBadRequest, w.Code)
}