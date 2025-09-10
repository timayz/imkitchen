package tests

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
)

// MockRotationService for testing
type MockRotationService struct {
	mock.Mock
}

func (m *MockRotationService) GetRotationState(userID uuid.UUID) (*services.RotationState, error) {
	args := m.Called(userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.RotationState), args.Error(1)
}

func (m *MockRotationService) UpdateRotationState(userID uuid.UUID, state *services.RotationState) error {
	args := m.Called(userID, state)
	return args.Error(0)
}

func (m *MockRotationService) SelectRecipesForWeek(userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, error) {
	args := m.Called(userID, preferences)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.WeeklyMeals), args.Error(1)
}

func (m *MockRotationService) SelectRecipesForWeekWithConstraintHandling(userID uuid.UUID, preferences *models.UserPreferences) (*models.WeeklyMeals, *services.RotationConstraintReport, error) {
	args := m.Called(userID, preferences)
	if args.Get(0) == nil {
		return nil, nil, args.Error(2)
	}
	return args.Get(0).(*models.WeeklyMeals), args.Get(1).(*services.RotationConstraintReport), args.Error(2)
}

func (m *MockRotationService) SelectRecipesForWeekWithPatterns(userID uuid.UUID, preferences *models.UserPreferences, weeklyPatterns []models.UserWeeklyPattern) (*models.WeeklyMeals, error) {
	args := m.Called(userID, preferences, weeklyPatterns)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.WeeklyMeals), args.Error(1)
}

func (m *MockRotationService) AssignRecipeForDay(day time.Weekday, userPatterns []models.UserWeeklyPattern, criteria *services.RecipeSelectionCriteria, recipes []models.Recipe, rotationState *services.RotationState) (*models.Recipe, error) {
	args := m.Called(day, userPatterns, criteria, recipes, rotationState)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRotationService) ResetRotationCycle(userID uuid.UUID) error {
	args := m.Called(userID)
	return args.Error(0)
}

func (m *MockRotationService) ResetRotationCycleWithOptions(userID uuid.UUID, req *models.RotationResetRequest) error {
	args := m.Called(userID, req)
	return args.Error(0)
}

func (m *MockRotationService) GetVarietyScore(recipeIDs []string, userID uuid.UUID) (float64, error) {
	args := m.Called(recipeIDs, userID)
	return args.Get(0).(float64), args.Error(1)
}

func (m *MockRotationService) GetRotationAnalytics(userID uuid.UUID, weeks int) (*models.RotationAnalytics, error) {
	args := m.Called(userID, weeks)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.RotationAnalytics), args.Error(1)
}

func (m *MockRotationService) ExportRotationData(userID uuid.UUID, format string, startDate, endDate time.Time) ([]byte, error) {
	args := m.Called(userID, format, startDate, endDate)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]byte), args.Error(1)
}

// TestResetRotationCycleWithOptions tests the enhanced reset functionality
func TestResetRotationCycleWithOptions(t *testing.T) {
	gin.SetMode(gin.TestMode)
	
	// Create mock service
	mockService := new(MockRotationService)
	
	// Setup router
	router := gin.New()
	router.Use(func(c *gin.Context) {
		c.Set("UserID", "550e8400-e29b-41d4-a716-446655440000")
		c.Next()
	})
	
	router.POST("/api/v1/users/rotation/reset", handlers.ResetRotationCycleWithOptions(mockService))

	t.Run("Given valid reset request with confirmation", func(t *testing.T) {
		// Setup
		resetReq := models.RotationResetRequest{
			ConfirmReset:      true,
			PreservePatterns:  true,
			PreserveFavorites: true,
		}

		userID := uuid.MustParse("550e8400-e29b-41d4-a716-446655440000")
		mockService.On("ResetRotationCycleWithOptions", userID, &resetReq).Return(nil)

		// Create request
		body, _ := json.Marshal(resetReq)
		req, _ := http.NewRequest("POST", "/api/v1/users/rotation/reset", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		t.Run("When reset is requested", func(t *testing.T) {
			t.Run("Then reset should succeed", func(t *testing.T) {
				assert.Equal(t, http.StatusOK, w.Code)
				
				var response map[string]interface{}
				err := json.Unmarshal(w.Body.Bytes(), &response)
				require.NoError(t, err)
				
				assert.Equal(t, "Rotation cycle reset successfully", response["message"])
				assert.Contains(t, response, "resetAt")
				assert.Contains(t, response, "preserved")
				
				preserved := response["preserved"].(map[string]interface{})
				assert.Equal(t, true, preserved["patterns"])
				assert.Equal(t, true, preserved["favorites"])
				
				mockService.AssertExpectations(t)
			})
		})
	})

	t.Run("Given reset request without confirmation", func(t *testing.T) {
		// Create new mock for this test
		mockService2 := new(MockRotationService)
		router2 := gin.New()
		router2.Use(func(c *gin.Context) {
			c.Set("UserID", "550e8400-e29b-41d4-a716-446655440000")
			c.Next()
		})
		router2.POST("/api/v1/users/rotation/reset", handlers.ResetRotationCycleWithOptions(mockService2))

		resetReq := models.RotationResetRequest{
			ConfirmReset:      false,
			PreservePatterns:  true,
			PreserveFavorites: true,
		}

		body, _ := json.Marshal(resetReq)
		req, _ := http.NewRequest("POST", "/api/v1/users/rotation/reset", bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		
		w := httptest.NewRecorder()
		router2.ServeHTTP(w, req)

		t.Run("When reset is requested", func(t *testing.T) {
			t.Run("Then request should be rejected", func(t *testing.T) {
				assert.Equal(t, http.StatusBadRequest, w.Code)
				
				var response map[string]interface{}
				err := json.Unmarshal(w.Body.Bytes(), &response)
				require.NoError(t, err)
				
				assert.Equal(t, "Reset confirmation required", response["error"])
				assert.Equal(t, "confirmReset must be true to proceed with reset", response["message"])
				
				// Ensure service was never called
				mockService2.AssertNotCalled(t, "ResetRotationCycleWithOptions")
			})
		})
	})
}

// TestRotationAnalyticsEndpoint tests the analytics API endpoint
func TestRotationAnalyticsEndpoint(t *testing.T) {
	gin.SetMode(gin.TestMode)
	
	mockService := new(MockRotationService)
	
	router := gin.New()
	router.Use(func(c *gin.Context) {
		c.Set("UserID", "550e8400-e29b-41d4-a716-446655440000")
		c.Next()
	})
	
	router.GET("/api/v1/users/rotation/stats", handlers.GetRotationAnalytics(mockService))

	t.Run("Given user requests analytics", func(t *testing.T) {
		userID := uuid.MustParse("550e8400-e29b-41d4-a716-446655440000")
		
		expectedAnalytics := &models.RotationAnalytics{
			UserID:        userID,
			CalculatedAt:  time.Now(),
			VarietyScore:  85.5,
			WeeksAnalyzed: 12,
			ComplexityDistribution: map[string]float64{
				"simple":   40.0,
				"moderate": 45.0,
				"complex":  15.0,
			},
			RotationEfficiency: 92.0,
		}

		mockService.On("GetRotationAnalytics", userID, 12).Return(expectedAnalytics, nil)

		req, _ := http.NewRequest("GET", "/api/v1/users/rotation/stats", nil)
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		t.Run("When analytics are requested", func(t *testing.T) {
			t.Run("Then analytics should be returned", func(t *testing.T) {
				assert.Equal(t, http.StatusOK, w.Code)
				
				var response map[string]interface{}
				err := json.Unmarshal(w.Body.Bytes(), &response)
				require.NoError(t, err)
				
				assert.Contains(t, response, "data")
				assert.Contains(t, response, "metadata")
				
				data := response["data"].(map[string]interface{})
				assert.Equal(t, 85.5, data["varietyScore"])
				assert.Equal(t, float64(12), data["weeksAnalyzed"])
				
				mockService.AssertExpectations(t)
			})
		})
	})

	t.Run("Given user requests analytics with custom weeks parameter", func(t *testing.T) {
		// Create new mock for this test
		mockService2 := new(MockRotationService)
		router2 := gin.New()
		router2.Use(func(c *gin.Context) {
			c.Set("UserID", "550e8400-e29b-41d4-a716-446655440000")
			c.Next()
		})
		router2.GET("/api/v1/users/rotation/stats", handlers.GetRotationAnalytics(mockService2))

		userID := uuid.MustParse("550e8400-e29b-41d4-a716-446655440000")
		
		expectedAnalytics := &models.RotationAnalytics{
			UserID:        userID,
			WeeksAnalyzed: 24,
			VarietyScore:  78.0,
		}

		mockService2.On("GetRotationAnalytics", userID, 24).Return(expectedAnalytics, nil)

		req, _ := http.NewRequest("GET", "/api/v1/users/rotation/stats?weeks=24", nil)
		w := httptest.NewRecorder()
		router2.ServeHTTP(w, req)

		t.Run("When analytics with custom weeks are requested", func(t *testing.T) {
			t.Run("Then analytics should respect weeks parameter", func(t *testing.T) {
				assert.Equal(t, http.StatusOK, w.Code)
				
				var response map[string]interface{}
				err := json.Unmarshal(w.Body.Bytes(), &response)
				require.NoError(t, err)
				
				data := response["data"].(map[string]interface{})
				assert.Equal(t, float64(24), data["weeksAnalyzed"])
				
				mockService2.AssertExpectations(t)
			})
		})
	})
}