package tests

import (
	"testing"

	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
	"github.com/imkitchen/backend/internal/services"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
)

// MockPreferenceRepository for testing
type MockPreferenceRepository struct {
	mock.Mock
}

func (m *MockPreferenceRepository) GetUserPreferences(userID uuid.UUID) (*models.CoreUserPreferences, error) {
	args := m.Called(userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.CoreUserPreferences), args.Error(1)
}

func (m *MockPreferenceRepository) UpdateUserPreferences(userID uuid.UUID, preferences *models.CoreUserPreferences) error {
	args := m.Called(userID, preferences)
	return args.Error(0)
}

func (m *MockPreferenceRepository) GetUser(userID uuid.UUID) (*models.User, error) {
	args := m.Called(userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.User), args.Error(1)
}

func (m *MockPreferenceRepository) ValidatePreferences(preferences *models.CoreUserPreferences) error {
	args := m.Called(preferences)
	return args.Error(0)
}

func TestPreferenceService_GetUserPreferences(t *testing.T) {
	mockRepo := new(MockPreferenceRepository)
	service := services.NewPreferenceService(mockRepo)
	
	userID := uuid.New()
	
	t.Run("returns preferences successfully", func(t *testing.T) {
		expectedPrefs := &models.CoreUserPreferences{
			MaxCookTime:         45,
			PreferredComplexity: "simple",
		}
		
		mockRepo.On("GetUserPreferences", userID).Return(expectedPrefs, nil).Once()
		
		prefs, err := service.GetUserPreferences(userID)
		require.NoError(t, err)
		assert.Equal(t, expectedPrefs.MaxCookTime, prefs.MaxCookTime)
		assert.Equal(t, expectedPrefs.PreferredComplexity, prefs.PreferredComplexity)
		
		mockRepo.AssertExpectations(t)
	})
	
	t.Run("applies defaults for zero values", func(t *testing.T) {
		emptyPrefs := &models.CoreUserPreferences{
			MaxCookTime:         0,
			PreferredComplexity: "",
		}
		
		mockRepo.On("GetUserPreferences", userID).Return(emptyPrefs, nil).Once()
		
		prefs, err := service.GetUserPreferences(userID)
		require.NoError(t, err)
		assert.Equal(t, 60, prefs.MaxCookTime) // Default applied
		assert.Equal(t, "moderate", prefs.PreferredComplexity) // Default applied
		
		mockRepo.AssertExpectations(t)
	})
	
	t.Run("returns error for invalid user ID", func(t *testing.T) {
		_, err := service.GetUserPreferences(uuid.Nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "invalid user ID")
	})
}

func TestPreferenceService_UpdateUserPreferences(t *testing.T) {
	mockRepo := new(MockPreferenceRepository)
	service := services.NewPreferenceService(mockRepo)
	
	userID := uuid.New()
	
	t.Run("updates preferences successfully", func(t *testing.T) {
		preferences := &models.CoreUserPreferences{
			MaxCookTime:         90,
			PreferredComplexity: "complex",
		}
		
		mockRepo.On("UpdateUserPreferences", userID, preferences).Return(nil).Once()
		
		err := service.UpdateUserPreferences(userID, preferences)
		assert.NoError(t, err)
		
		mockRepo.AssertExpectations(t)
	})
	
	t.Run("validates MaxCookTime range", func(t *testing.T) {
		invalidPrefs := &models.CoreUserPreferences{
			MaxCookTime:         200, // Too high
			PreferredComplexity: "simple",
		}
		
		err := service.UpdateUserPreferences(userID, invalidPrefs)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "max cook time must be between 15 and 180 minutes")
	})
	
	t.Run("validates PreferredComplexity values", func(t *testing.T) {
		invalidPrefs := &models.CoreUserPreferences{
			MaxCookTime:         60,
			PreferredComplexity: "invalid",
		}
		
		err := service.UpdateUserPreferences(userID, invalidPrefs)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "preferred complexity must be one of")
	})
	
	t.Run("returns error for nil preferences", func(t *testing.T) {
		err := service.UpdateUserPreferences(userID, nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "preferences cannot be nil")
	})
	
	t.Run("returns error for invalid user ID", func(t *testing.T) {
		preferences := &models.CoreUserPreferences{
			MaxCookTime:         60,
			PreferredComplexity: "moderate",
		}
		
		err := service.UpdateUserPreferences(uuid.Nil, preferences)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "invalid user ID")
	})
}

func TestPreferenceService_ResetUserPreferences(t *testing.T) {
	mockRepo := new(MockPreferenceRepository)
	service := services.NewPreferenceService(mockRepo)
	
	userID := uuid.New()
	
	t.Run("resets preferences to defaults", func(t *testing.T) {
		// Expect update with default values
		expectedDefaults := &models.CoreUserPreferences{
			MaxCookTime:         60,
			PreferredComplexity: "moderate",
		}
		
		mockRepo.On("UpdateUserPreferences", userID, expectedDefaults).Return(nil).Once()
		
		err := service.ResetUserPreferences(userID)
		assert.NoError(t, err)
		
		mockRepo.AssertExpectations(t)
	})
}