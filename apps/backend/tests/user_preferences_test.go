package tests

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// MockUserRepository for testing user preference service
type MockUserRepository struct {
	mock.Mock
}

func (m *MockUserRepository) GetByID(id uuid.UUID) (*models.User, error) {
	args := m.Called(id)
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockUserRepository) Update(user *models.User) (*models.User, error) {
	args := m.Called(user)
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockUserRepository) UpdatePreferenceLearningData(id uuid.UUID, data json.RawMessage) error {
	args := m.Called(id, data)
	return args.Error(0)
}

func (m *MockUserRepository) UpdateRotationResetCount(id uuid.UUID, count int) error {
	args := m.Called(id, count)
	return args.Error(0)
}

func (m *MockUserRepository) GetUserPreferences(id uuid.UUID) (*models.UserPreferences, error) {
	args := m.Called(id)
	return args.Get(0).(*models.UserPreferences), args.Error(1)
}

// MockCacheService for testing
type MockCacheService struct {
	mock.Mock
}

func (m *MockCacheService) Get(key string) ([]byte, error) {
	args := m.Called(key)
	return args.Get(0).([]byte), args.Error(1)
}

func (m *MockCacheService) Set(key string, value []byte, ttl int) error {
	args := m.Called(key, value, ttl)
	return args.Error(0)
}

func (m *MockCacheService) Delete(key string) error {
	args := m.Called(key)
	return args.Error(0)
}

func TestUserPreferenceService_GetUserPreferences(t *testing.T) {
	// Setup
	userID := uuid.New()
	mockRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)
	
	// Create test user
	testUser := &models.User{
		ID:                      userID,
		CookingSkillLevel:       "intermediate",
		PreferredMealComplexity: "moderate",
		MaxCookTime:             60,
		DietaryRestrictions:     []string{"vegetarian"},
		PreferenceLearningData:  json.RawMessage(`{"weeklyAvailability":{"monday":45,"tuesday":30}}`),
	}

	// Mock expectations
	mockCache.On("Get", mock.AnythingOfType("string")).Return([]byte{}, assert.AnError)
	mockRepo.On("GetByID", userID).Return(testUser, nil)
	mockCache.On("Set", mock.AnythingOfType("string"), mock.Anything, mock.AnythingOfType("int")).Return(nil)

	// Create service
	service := services.NewUserPreferenceService(mockRepo, mockCache)

	// Test
	preferences, err := service.GetUserPreferences(userID)

	// Assertions
	assert.NoError(t, err)
	assert.NotNil(t, preferences)
	assert.Equal(t, "intermediate", preferences.CookingSkillLevel)
	assert.Equal(t, "moderate", preferences.PreferredMealComplexity)
	assert.Equal(t, 60, preferences.MaxPrepTimePerMeal)
	assert.Contains(t, preferences.DietaryRestrictions, "vegetarian")

	// Verify mocks
	mockRepo.AssertExpectations(t)
	mockCache.AssertExpectations(t)
}

func TestUserPreferenceService_UpdateUserPreferences(t *testing.T) {
	// Setup
	userID := uuid.New()
	mockRepo := new(MockUserRepository)
	mockCache := new(MockCacheService)

	// Create test user
	testUser := &models.User{
		ID:                      userID,
		CookingSkillLevel:       "beginner",
		PreferredMealComplexity: "simple",
		MaxCookTime:             30,
		DietaryRestrictions:     []string{},
		PreferenceLearningData:  json.RawMessage(`{}`),
	}

	updatedUser := &models.User{
		ID:                      userID,
		CookingSkillLevel:       "intermediate",
		PreferredMealComplexity: "moderate",
		MaxCookTime:             45,
		DietaryRestrictions:     []string{"vegetarian"},
		PreferenceLearningData:  json.RawMessage(`{"weeklyAvailability":{"monday":60}}`),
	}

	// Mock expectations
	mockRepo.On("GetByID", userID).Return(testUser, nil)
	mockRepo.On("Update", mock.AnythingOfType("*models.User")).Return(updatedUser, nil)
	mockCache.On("Delete", mock.AnythingOfType("string")).Return(nil)

	// Create service
	service := services.NewUserPreferenceService(mockRepo, mockCache)

	// Create update request
	maxPrepTime := 45
	complexity := "moderate"
	skill := "intermediate"
	updateReq := &services.UpdatePreferencesRequest{
		MaxPrepTimePerMeal:      &maxPrepTime,
		PreferredMealComplexity: &complexity,
		CookingSkillLevel:       &skill,
		DietaryRestrictions:     []string{"vegetarian"},
		WeeklyAvailability:      map[string]int{"monday": 60},
	}

	// Test
	preferences, err := service.UpdateUserPreferences(userID, updateReq)

	// Assertions
	assert.NoError(t, err)
	assert.NotNil(t, preferences)
	assert.Equal(t, "intermediate", preferences.CookingSkillLevel)
	assert.Equal(t, "moderate", preferences.PreferredMealComplexity)
	assert.Equal(t, 45, preferences.MaxPrepTimePerMeal)
	assert.Contains(t, preferences.DietaryRestrictions, "vegetarian")

	// Verify mocks
	mockRepo.AssertExpectations(t)
	mockCache.AssertExpectations(t)
}

func TestUserPreferenceService_ValidatePreferences(t *testing.T) {
	// Setup
	service := services.NewUserPreferenceService(nil, nil)

	t.Run("Valid preferences", func(t *testing.T) {
		maxPrepTime := 45
		complexity := "moderate"
		skill := "intermediate"
		req := &services.UpdatePreferencesRequest{
			MaxPrepTimePerMeal:      &maxPrepTime,
			PreferredMealComplexity: &complexity,
			CookingSkillLevel:       &skill,
			WeeklyAvailability: map[string]int{
				"monday":  60,
				"tuesday": 45,
			},
		}

		errors := service.ValidatePreferences(req)
		assert.Empty(t, errors)
	})

	t.Run("Invalid max prep time", func(t *testing.T) {
		maxPrepTime := 500 // Too high
		req := &services.UpdatePreferencesRequest{
			MaxPrepTimePerMeal: &maxPrepTime,
		}

		errors := service.ValidatePreferences(req)
		assert.NotEmpty(t, errors)
		assert.Contains(t, errors[0], "maxPrepTimePerMeal must be between 5 and 300")
	})

	t.Run("Invalid complexity", func(t *testing.T) {
		complexity := "invalid"
		req := &services.UpdatePreferencesRequest{
			PreferredMealComplexity: &complexity,
		}

		errors := service.ValidatePreferences(req)
		assert.NotEmpty(t, errors)
		assert.Contains(t, errors[0], "preferredMealComplexity must be one of: simple, moderate, complex")
	})

	t.Run("Invalid weekly availability", func(t *testing.T) {
		req := &services.UpdatePreferencesRequest{
			WeeklyAvailability: map[string]int{
				"invalid_day": 60,
				"monday":      500, // Too high
			},
		}

		errors := service.ValidatePreferences(req)
		assert.Len(t, errors, 2)
	})
}

func TestUserPreferenceService_GetDefaultPreferences(t *testing.T) {
	// Setup
	service := services.NewUserPreferenceService(nil, nil)

	// Test
	defaults := service.GetDefaultPreferences()

	// Assertions
	assert.NotNil(t, defaults)
	assert.Equal(t, "intermediate", defaults.CookingSkillLevel)
	assert.Equal(t, "moderate", defaults.PreferredMealComplexity)
	assert.Equal(t, 45, defaults.MaxPrepTimePerMeal)
	assert.Equal(t, 7, len(defaults.WeeklyAvailability))
	assert.Equal(t, 60, defaults.WeeklyAvailability["saturday"])
	assert.Equal(t, 60, defaults.WeeklyAvailability["sunday"])
	assert.Equal(t, 30, defaults.WeeklyAvailability["monday"])
}