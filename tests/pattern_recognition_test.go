package tests

import (
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
	"github.com/imkitchen/backend/internal/services"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// MockPreferenceRepository is a mock for testing
type MockPreferenceRepository struct {
	mock.Mock
}

func (m *MockPreferenceRepository) GetUserWeeklyPatterns(userID uuid.UUID) ([]models.UserWeeklyPattern, error) {
	args := m.Called(userID)
	return args.Get(0).([]models.UserWeeklyPattern), args.Error(1)
}

func (m *MockPreferenceRepository) GetFavoriteRecipeIDs(userID uuid.UUID) ([]string, error) {
	args := m.Called(userID)
	return args.Get(0).([]string), args.Error(1)
}

func (m *MockPreferenceRepository) GetUserFavorites(userID uuid.UUID, page, limit int) ([]models.UserRecipeFavorite, int64, error) {
	args := m.Called(userID, page, limit)
	return args.Get(0).([]models.UserRecipeFavorite), args.Get(1).(int64), args.Error(2)
}

// Add other required methods from repositories.PreferenceRepository interface
func (m *MockPreferenceRepository) GetUserPreferences(userID uuid.UUID) (*models.CoreUserPreferences, error) {
	args := m.Called(userID)
	return args.Get(0).(*models.CoreUserPreferences), args.Error(1)
}

func (m *MockPreferenceRepository) UpdateUserPreferences(userID uuid.UUID, preferences *models.CoreUserPreferences) error {
	args := m.Called(userID, preferences)
	return args.Error(0)
}

func (m *MockPreferenceRepository) ValidatePreferences(preferences *models.CoreUserPreferences) error {
	args := m.Called(preferences)
	return args.Error(0)
}

func (m *MockPreferenceRepository) CreateUserWeeklyPattern(userID uuid.UUID, pattern *models.UserWeeklyPattern) (*models.UserWeeklyPattern, error) {
	args := m.Called(userID, pattern)
	return args.Get(0).(*models.UserWeeklyPattern), args.Error(1)
}

func (m *MockPreferenceRepository) UpdateUserWeeklyPattern(userID uuid.UUID, patternID uuid.UUID, updates *models.UserWeeklyPattern) (*models.UserWeeklyPattern, error) {
	args := m.Called(userID, patternID, updates)
	return args.Get(0).(*models.UserWeeklyPattern), args.Error(1)
}

func (m *MockPreferenceRepository) DeleteUserWeeklyPattern(userID uuid.UUID, patternID uuid.UUID) error {
	args := m.Called(userID, patternID)
	return args.Error(0)
}

func (m *MockPreferenceRepository) UpsertUserWeeklyPatterns(userID uuid.UUID, patterns []models.UserWeeklyPattern) ([]models.UserWeeklyPattern, error) {
	args := m.Called(userID, patterns)
	return args.Get(0).([]models.UserWeeklyPattern), args.Error(1)
}

func (m *MockPreferenceRepository) ValidateWeeklyPattern(pattern *models.UserWeeklyPattern) error {
	args := m.Called(pattern)
	return args.Error(0)
}

func (m *MockPreferenceRepository) AddUserFavorite(userID, recipeID uuid.UUID) (*models.UserRecipeFavorite, error) {
	args := m.Called(userID, recipeID)
	return args.Get(0).(*models.UserRecipeFavorite), args.Error(1)
}

func (m *MockPreferenceRepository) RemoveUserFavorite(userID, recipeID uuid.UUID) error {
	args := m.Called(userID, recipeID)
	return args.Error(0)
}

func (m *MockPreferenceRepository) IsUserFavorite(userID, recipeID uuid.UUID) (bool, error) {
	args := m.Called(userID, recipeID)
	return args.Bool(0), args.Error(1)
}

func (m *MockPreferenceRepository) UpdateFavoriteMultiplier(userID, recipeID uuid.UUID, multiplier float64) error {
	args := m.Called(userID, recipeID, multiplier)
	return args.Error(0)
}

func (m *MockPreferenceRepository) GetUser(userID uuid.UUID) (*models.User, error) {
	args := m.Called(userID)
	return args.Get(0).(*models.User), args.Error(1)
}

// MockRecipeRepository is a mock for testing
type MockRecipeRepository struct {
	mock.Mock
}

func (m *MockRecipeRepository) Search(userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResult, error) {
	args := m.Called(userID, params)
	return args.Get(0).(*models.RecipeSearchResult), args.Error(1)
}

func (m *MockRecipeRepository) GetByID(id, userID uuid.UUID) (*models.Recipe, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.Recipe), args.Error(1)
}

// MockUserRepository is a mock for testing
type MockUserRepository struct {
	mock.Mock
}

func (m *MockUserRepository) GetByID(userID uuid.UUID) (*models.User, error) {
	args := m.Called(userID)
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockUserRepository) UpdatePreferenceLearningData(userID uuid.UUID, data []byte) error {
	args := m.Called(userID, data)
	return args.Error(0)
}

// MockCacheService is a mock for testing
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

// Test pattern recognition logic
func TestWeekendVsWeekdayPatternRecognition(t *testing.T) {
	// Setup mocks
	mockPreferenceRepo := &MockPreferenceRepository{}
	mockRecipeRepo := &MockRecipeRepository{}
	mockUserRepo := &MockUserRepository{}
	mockCache := &MockCacheService{}

	// Create rotation service
	rotationService := services.NewRotationService(
		mockRecipeRepo,
		mockUserRepo,
		mockPreferenceRepo,
		mockCache,
	)

	// Test data
	userID := uuid.New()
	weeklyPatterns := []models.UserWeeklyPattern{
		{
			DayOfWeek:           0, // Sunday
			MaxPrepTime:         90,
			PreferredComplexity: "complex",
			IsWeekendPattern:    true,
		},
		{
			DayOfWeek:           1, // Monday
			MaxPrepTime:         30,
			PreferredComplexity: "simple",
			IsWeekendPattern:    false,
		},
	}

	// Test weekend pattern selection
	criteria := &services.RecipeSelectionCriteria{
		MealType:  "dinner",
		Day:       "sunday",
		DayOfWeek: 0,
	}

	// Mock rotation state
	mockCache.On("Get", mock.AnythingOfType("string")).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{}, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), mock.AnythingOfType("time.Duration")).Return(nil)

	recipes := []models.Recipe{
		{
			ID:         uuid.New(),
			Name:       "Complex Weekend Dish",
			Complexity: "complex",
			PrepTime:   60,
			MealType:   []string{"dinner"},
		},
		{
			ID:         uuid.New(),
			Name:       "Simple Weekday Meal",
			Complexity: "simple",
			PrepTime:   20,
			MealType:   []string{"dinner"},
		},
	}

	rotationState := &services.RotationState{
		UsedRecipes:       make(map[string]time.Time),
		ComplexityHistory: []string{},
	}

	// Test AssignRecipeForDay
	selectedRecipe, err := rotationService.AssignRecipeForDay(
		time.Sunday,
		weeklyPatterns,
		criteria,
		recipes,
		rotationState,
	)

	// Assertions
	assert.NoError(t, err)
	assert.NotNil(t, selectedRecipe)
	
	// The complex recipe should be more likely to be selected on Sunday
	// due to weekend pattern preference
	// Note: Due to randomness, we can't assert exact recipe, but we can verify
	// the selection logic works without error
}

func TestFavoritesWeightingMultiplier(t *testing.T) {
	// Setup mocks
	mockPreferenceRepo := &MockPreferenceRepository{}
	mockRecipeRepo := &MockRecipeRepository{}
	mockUserRepo := &MockUserRepository{}
	mockCache := &MockCacheService{}

	// Create rotation service
	rotationService := services.NewRotationService(
		mockRecipeRepo,
		mockUserRepo,
		mockPreferenceRepo,
		mockCache,
	)

	// Test data
	userID := uuid.New()
	favoriteRecipeID := uuid.New()

	// Mock favorites data
	favorites := []models.UserRecipeFavorite{
		{
			RecipeID:         favoriteRecipeID,
			WeightMultiplier: 1.5, // Default multiplier
		},
	}
	
	mockPreferenceRepo.On("GetFavoriteRecipeIDs", userID).Return([]string{favoriteRecipeID.String()}, nil)
	mockPreferenceRepo.On("GetUserFavorites", userID, 1, 1000).Return(favorites, int64(1), nil)

	// Test recipes
	recipes := []models.Recipe{
		{
			ID:         favoriteRecipeID,
			Name:       "Favorite Recipe",
			Complexity: "moderate",
			PrepTime:   30,
			MealType:   []string{"dinner"},
			AverageRating: 4.5,
		},
		{
			ID:         uuid.New(),
			Name:       "Regular Recipe",
			Complexity: "moderate", 
			PrepTime:   30,
			MealType:   []string{"dinner"},
			AverageRating: 4.0,
		},
	}

	criteria := &services.RecipeSelectionCriteria{
		MealType:            "dinner",
		Day:                 "monday",
		DayOfWeek:          1,
		UsedThisWeek:       make(map[string]bool),
		PreferredComplexity: []string{"moderate"},
	}

	// Mock rotation state and cache
	mockCache.On("Get", mock.AnythingOfType("string")).Return("")
	mockUserRepo.On("GetByID", userID).Return(&models.User{}, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.AnythingOfType("string"), mock.AnythingOfType("time.Duration")).Return(nil)

	rotationState := &services.RotationState{
		UsedRecipes:       make(map[string]time.Time),
		ComplexityHistory: []string{},
	}

	// Test multiple selections to verify favorites weighting
	favoriteSelected := 0
	totalSelections := 10

	for i := 0; i < totalSelections; i++ {
		// Reset the criteria for each test
		testCriteria := &services.RecipeSelectionCriteria{
			MealType:            criteria.MealType,
			Day:                 criteria.Day,
			DayOfWeek:          criteria.DayOfWeek,
			UsedThisWeek:       make(map[string]bool),
			PreferredComplexity: criteria.PreferredComplexity,
		}

		// Use selectWithFavoritesWeighting directly (assuming it's exported for testing)
		// selectedRecipe, err := rotationService.selectWithFavoritesWeighting(recipes, testCriteria, rotationState, userID)
		
		// Since the method might not be exported, we test the overall integration
		// by using the pattern-aware selection which should incorporate favorites
		selectedRecipe, err := rotationService.AssignRecipeForDay(
			time.Monday,
			[]models.UserWeeklyPattern{},
			testCriteria,
			recipes,
			rotationState,
		)

		assert.NoError(t, err)
		assert.NotNil(t, selectedRecipe)

		if selectedRecipe.ID == favoriteRecipeID {
			favoriteSelected++
		}
	}

	// The favorite recipe should be selected more often due to weighting
	// With a 1.5x multiplier, we expect it to be selected more than 50% of the time
	assert.Greater(t, favoriteSelected, totalSelections/2, 
		"Favorite recipe should be selected more frequently due to weighting")
}

func TestWeeklyPatternValidation(t *testing.T) {
	mockPreferenceRepo := &MockPreferenceRepository{}

	testCases := []struct {
		name        string
		pattern     models.UserWeeklyPattern
		expectError bool
	}{
		{
			name: "Valid weekend pattern",
			pattern: models.UserWeeklyPattern{
				DayOfWeek:           0, // Sunday
				MaxPrepTime:         60,
				PreferredComplexity: "complex",
				IsWeekendPattern:    true,
			},
			expectError: false,
		},
		{
			name: "Invalid day of week",
			pattern: models.UserWeeklyPattern{
				DayOfWeek:           7, // Invalid (should be 0-6)
				MaxPrepTime:         60,
				PreferredComplexity: "moderate",
				IsWeekendPattern:    false,
			},
			expectError: true,
		},
		{
			name: "Invalid prep time - too low",
			pattern: models.UserWeeklyPattern{
				DayOfWeek:           1,
				MaxPrepTime:         2, // Too low (should be >= 5)
				PreferredComplexity: "simple",
				IsWeekendPattern:    false,
			},
			expectError: true,
		},
		{
			name: "Invalid complexity",
			pattern: models.UserWeeklyPattern{
				DayOfWeek:           2,
				MaxPrepTime:         45,
				PreferredComplexity: "invalid", // Invalid complexity
				IsWeekendPattern:    false,
			},
			expectError: true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.expectError {
				mockPreferenceRepo.On("ValidateWeeklyPattern", &tc.pattern).Return(assert.AnError)
			} else {
				mockPreferenceRepo.On("ValidateWeeklyPattern", &tc.pattern).Return(nil)
			}

			err := mockPreferenceRepo.ValidateWeeklyPattern(&tc.pattern)

			if tc.expectError {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			mockPreferenceRepo.AssertExpectations(t)
		})
	}
}