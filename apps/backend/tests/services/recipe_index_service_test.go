package services

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/imkitchen/backend/internal/models"
)

// MockRecipeRepository for testing
type MockRecipeRepository struct {
	mock.Mock
}

func (m *MockRecipeRepository) GetByUserID(userID uuid.UUID, limit, offset int) ([]models.Recipe, error) {
	args := m.Called(userID, limit, offset)
	return args.Get(0).([]models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) GetSimilarRecipes(userID uuid.UUID, filters *models.RecipeFilters) ([]models.Recipe, error) {
	args := m.Called(userID, filters)
	return args.Get(0).([]models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) Create(recipe *models.Recipe) error {
	args := m.Called(recipe)
	return args.Error(0)
}

func (m *MockRecipeRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.Recipe, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.Recipe), args.Error(1)
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

// MockCacheService for testing
type MockCacheService struct {
	mock.Mock
}

func (m *MockCacheService) Get(ctx context.Context, key string) (string, error) {
	args := m.Called(ctx, key)
	return args.String(0), args.Error(1)
}

func (m *MockCacheService) Set(ctx context.Context, key string, value interface{}, ttl time.Duration) error {
	args := m.Called(ctx, key, value, ttl)
	return args.Error(0)
}

func (m *MockCacheService) Delete(ctx context.Context, key string) error {
	args := m.Called(ctx, key)
	return args.Error(0)
}

func TestRecipeIndexService_BuildIndex(t *testing.T) {
	mockRepo := &MockRecipeRepository{}
	mockCache := &MockCacheService{}
	service := NewRecipeIndexService(mockRepo, mockCache)

	userID := uuid.New()
	recipes := []models.Recipe{
		{
			ID:        uuid.New(),
			Title:     "Breakfast Pancakes",
			MealType:  []string{"breakfast"},
			Complexity: "easy",
			DietaryLabels: []string{"vegetarian"},
			CuisineType: "american",
			TotalTime: 30,
			PrepTime:  10,
			CookTime:  20,
		},
		{
			ID:        uuid.New(),
			Title:     "Chicken Stir Fry",
			MealType:  []string{"dinner"},
			Complexity: "medium",
			DietaryLabels: []string{"gluten-free"},
			CuisineType: "asian",
			TotalTime: 45,
			PrepTime:  15,
			CookTime:  30,
		},
	}

	mockRepo.On("GetByUserID", userID, mock.AnythingOfType("int"), mock.AnythingOfType("int")).Return(recipes, nil)

	index, err := service.BuildIndex(context.Background(), userID)

	assert.NoError(t, err)
	assert.NotNil(t, index)
	assert.Contains(t, index.ByMealType, "breakfast")
	assert.Contains(t, index.ByMealType, "dinner")
	assert.Contains(t, index.ByComplexity, "easy")
	assert.Contains(t, index.ByComplexity, "medium")
	assert.Len(t, index.RecipeMetadata, 2)

	mockRepo.AssertExpectations(t)
}

func TestRecipeIndexService_GetCachedIndex(t *testing.T) {
	mockRepo := &MockRecipeRepository{}
	mockCache := &MockCacheService{}
	service := NewRecipeIndexService(mockRepo, mockCache)

	userID := uuid.New()
	cacheKey := "recipe_index:" + userID.String()

	// Test cache hit
	cachedIndex := &RecipeIndex{
		ByMealType: map[string][]string{
			"breakfast": {"recipe-1"},
		},
		RecipeMetadata: map[string]*RecipeIndexEntry{
			"recipe-1": {
				ID:           "recipe-1",
				Title:        "Test Recipe",
				MealType:     []string{"breakfast"},
				Complexity:   "easy",
				TotalTime:    30,
			},
		},
	}

	cachedData, _ := json.Marshal(cachedIndex)
	mockCache.On("Get", mock.Anything, cacheKey).Return(string(cachedData), nil)

	index, err := service.GetCachedIndex(context.Background(), userID)

	assert.NoError(t, err)
	assert.NotNil(t, index)
	assert.Contains(t, index.ByMealType, "breakfast")

	mockCache.AssertExpectations(t)
}

func TestRecipeIndexService_InvalidateIndex(t *testing.T) {
	mockRepo := &MockRecipeRepository{}
	mockCache := &MockCacheService{}
	service := NewRecipeIndexService(mockRepo, mockCache)

	userID := uuid.New()
	cacheKey := "recipe_index:" + userID.String()

	mockCache.On("Delete", mock.Anything, cacheKey).Return(nil)

	err := service.InvalidateIndex(context.Background(), userID)

	assert.NoError(t, err)
	mockCache.AssertExpectations(t)
}

func TestRecipeIndexService_GetRecipesByFilter(t *testing.T) {
	mockRepo := &MockRecipeRepository{}
	mockCache := &MockCacheService{}
	service := NewRecipeIndexService(mockRepo, mockCache)

	userID := uuid.New()
	
	// Mock cached index
	cachedIndex := &RecipeIndex{
		ByMealType: map[string][]string{
			"breakfast": {"recipe-1", "recipe-2"},
			"dinner":    {"recipe-3"},
		},
		ByComplexity: map[string][]string{
			"easy":   {"recipe-1", "recipe-3"},
			"medium": {"recipe-2"},
		},
		RecipeMetadata: map[string]*RecipeIndexEntry{
			"recipe-1": {
				ID:           "recipe-1",
				MealType:     []string{"breakfast"},
				Complexity:   "easy",
				TotalTime:    30,
			},
			"recipe-2": {
				ID:           "recipe-2",
				MealType:     []string{"breakfast"},
				Complexity:   "medium",
				TotalTime:    45,
			},
			"recipe-3": {
				ID:           "recipe-3",
				MealType:     []string{"dinner"},
				Complexity:   "easy",
				TotalTime:    60,
			},
		},
	}

	cachedData, _ := json.Marshal(cachedIndex)
	cacheKey := "recipe_index:" + userID.String()
	mockCache.On("Get", mock.Anything, cacheKey).Return(string(cachedData), nil)

	filters := &RecipeIndexFilter{
		MealType:   []string{"breakfast"},
		Complexity: []string{"easy"},
	}

	recipes, err := service.GetRecipesByFilter(context.Background(), userID, filters)

	assert.NoError(t, err)
	assert.Len(t, recipes, 1)
	assert.Equal(t, "recipe-1", recipes[0].ID)

	mockCache.AssertExpectations(t)
}

func TestRecipeIndexService_Performance(t *testing.T) {
	mockRepo := &MockRecipeRepository{}
	mockCache := &MockCacheService{}
	service := NewRecipeIndexService(mockRepo, mockCache)

	userID := uuid.New()
	
	// Create a large number of recipes to test performance
	recipes := make([]models.Recipe, 1000)
	for i := 0; i < 1000; i++ {
		recipes[i] = models.Recipe{
			ID:         uuid.New(),
			Title:      "Recipe " + string(rune(i)),
			MealType:   []string{"breakfast", "lunch", "dinner"}[i%3:i%3+1],
			Complexity: []string{"easy", "medium", "hard"}[i%3],
			TotalTime:  30 + (i % 60),
		}
	}

	mockRepo.On("GetByUserID", userID, mock.AnythingOfType("int"), mock.AnythingOfType("int")).Return(recipes, nil)

	start := time.Now()
	index, err := service.BuildIndex(context.Background(), userID)
	duration := time.Since(start)

	assert.NoError(t, err)
	assert.NotNil(t, index)
	assert.Less(t, duration, 100*time.Millisecond, "Index building should be fast")

	// Test filtering performance
	filters := &RecipeIndexFilter{
		MealType: []string{"breakfast"},
	}

	cachedData, _ := json.Marshal(index)
	cacheKey := "recipe_index:" + userID.String()
	mockCache.On("Get", mock.Anything, cacheKey).Return(string(cachedData), nil)

	start = time.Now()
	filteredRecipes, err := service.GetRecipesByFilter(context.Background(), userID, filters)
	filterDuration := time.Since(start)

	assert.NoError(t, err)
	assert.NotEmpty(t, filteredRecipes)
	assert.Less(t, filterDuration, 10*time.Millisecond, "Filtering should be very fast")

	mockRepo.AssertExpectations(t)
}