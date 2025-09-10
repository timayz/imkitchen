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
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/suite"

	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/models"
)

// Mock MealPlan Service
type MockMealPlanService struct {
	mock.Mock
}

func (m *MockMealPlanService) CreateMealPlan(userID uuid.UUID, input *models.CreateMealPlanInput) (*models.MealPlan, error) {
	args := m.Called(userID, input)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanService) GetMealPlan(id uuid.UUID, userID uuid.UUID) (*models.MealPlanResponse, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.MealPlanResponse), args.Error(1)
}

func (m *MockMealPlanService) GetUserMealPlans(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.MealPlanResponse, error) {
	args := m.Called(userID, filters)
	return args.Get(0).([]models.MealPlanResponse), args.Error(1)
}

func (m *MockMealPlanService) GetMealPlanByWeek(userID uuid.UUID, weekStart time.Time) (*models.MealPlanResponse, error) {
	args := m.Called(userID, weekStart)
	return args.Get(0).(*models.MealPlanResponse), args.Error(1)
}

func (m *MockMealPlanService) UpdateMealPlan(id uuid.UUID, userID uuid.UUID, input *models.UpdateMealPlanInput) (*models.MealPlanResponse, error) {
	args := m.Called(id, userID, input)
	return args.Get(0).(*models.MealPlanResponse), args.Error(1)
}

func (m *MockMealPlanService) UpdateMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string, input *models.UpdateMealSlotInput) (*models.MealPlanResponse, error) {
	args := m.Called(mealPlanID, userID, day, mealType, input)
	return args.Get(0).(*models.MealPlanResponse), args.Error(1)
}

func (m *MockMealPlanService) DeleteMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string) (*models.MealPlanResponse, error) {
	args := m.Called(mealPlanID, userID, day, mealType)
	return args.Get(0).(*models.MealPlanResponse), args.Error(1)
}

func (m *MockMealPlanService) DeleteMealPlan(id uuid.UUID, userID uuid.UUID) error {
	args := m.Called(id, userID)
	return args.Error(0)
}

// Test Suite
type MealPlanHandlerTestSuite struct {
	suite.Suite
	router      *gin.Engine
	service     *MockMealPlanService
	handler     *handlers.MealPlanHandler
	userID      uuid.UUID
}

func (suite *MealPlanHandlerTestSuite) SetupTest() {
	gin.SetMode(gin.TestMode)
	
	suite.service = new(MockMealPlanService)
	suite.handler = handlers.NewMealPlanHandler(suite.service)
	suite.userID = uuid.New()
	
	suite.router = gin.New()
	
	// Middleware to set user ID in context
	suite.router.Use(func(c *gin.Context) {
		c.Set("userID", suite.userID)
		c.Next()
	})
	
	// Register routes
	handlers.RegisterMealPlanRoutes(suite.router.Group("/api/v1"), suite.handler)
}

func (suite *MealPlanHandlerTestSuite) TestCreateMealPlan_Success() {
	// Arrange
	input := models.CreateMealPlanInput{
		WeekStartDate:  time.Now().Truncate(24 * time.Hour),
		GenerationType: "manual",
		Meals:          models.WeeklyMeals{},
	}
	
	expectedMealPlan := &models.MealPlan{
		ID:             uuid.New(),
		UserID:         suite.userID,
		WeekStartDate:  input.WeekStartDate,
		GenerationType: input.GenerationType,
		Status:         "active",
	}
	
	suite.service.On("CreateMealPlan", suite.userID, mock.MatchedBy(func(i *models.CreateMealPlanInput) bool {
		return i.GenerationType == "manual"
	})).Return(expectedMealPlan, nil)
	
	body, _ := json.Marshal(input)
	
	// Act
	req := httptest.NewRequest("POST", "/api/v1/meal-plans", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusCreated, w.Code)
	
	var response models.MealPlan
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Equal(suite.T(), expectedMealPlan.ID, response.ID)
	assert.Equal(suite.T(), expectedMealPlan.GenerationType, response.GenerationType)
	
	suite.service.AssertExpectations(suite.T())
}

func (suite *MealPlanHandlerTestSuite) TestCreateMealPlan_InvalidInput() {
	// Arrange
	invalidInput := map[string]interface{}{
		"generationType": "invalid_type",
	}
	
	body, _ := json.Marshal(invalidInput)
	
	// Act
	req := httptest.NewRequest("POST", "/api/v1/meal-plans", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusBadRequest, w.Code)
	
	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Contains(suite.T(), response["error"], "Invalid request body")
}

func (suite *MealPlanHandlerTestSuite) TestGetMealPlan_Success() {
	// Arrange
	mealPlanID := uuid.New()
	expectedResponse := &models.MealPlanResponse{
		MealPlan: models.MealPlan{
			ID:     mealPlanID,
			UserID: suite.userID,
		},
		PopulatedMeals: models.WeeklyMealsWithRecipes{},
	}
	
	suite.service.On("GetMealPlan", mealPlanID, suite.userID).Return(expectedResponse, nil)
	
	// Act
	req := httptest.NewRequest("GET", "/api/v1/meal-plans/"+mealPlanID.String(), nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	var response models.MealPlanResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Equal(suite.T(), mealPlanID, response.ID)
	
	suite.service.AssertExpectations(suite.T())
}

func (suite *MealPlanHandlerTestSuite) TestGetMealPlan_InvalidID() {
	// Act
	req := httptest.NewRequest("GET", "/api/v1/meal-plans/invalid-uuid", nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusBadRequest, w.Code)
	
	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Contains(suite.T(), response["error"], "Invalid meal plan ID")
}

func (suite *MealPlanHandlerTestSuite) TestGetMealPlans_Success() {
	// Arrange
	expectedMealPlans := []models.MealPlanResponse{
		{
			MealPlan: models.MealPlan{
				ID:     uuid.New(),
				UserID: suite.userID,
			},
		},
	}
	
	suite.service.On("GetUserMealPlans", suite.userID, mock.AnythingOfType("*models.MealPlanFilters")).Return(expectedMealPlans, nil)
	
	// Act
	req := httptest.NewRequest("GET", "/api/v1/meal-plans", nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Equal(suite.T(), float64(1), response["count"])
	
	suite.service.AssertExpectations(suite.T())
}

func (suite *MealPlanHandlerTestSuite) TestUpdateMealSlot_Success() {
	// Arrange
	mealPlanID := uuid.New()
	day := "monday"
	mealType := "breakfast"
	
	input := models.UpdateMealSlotInput{
		RecipeID: &[]string{"recipe-123"}[0],
		Servings: &[]int{2}[0],
	}
	
	expectedResponse := &models.MealPlanResponse{
		MealPlan: models.MealPlan{
			ID:     mealPlanID,
			UserID: suite.userID,
		},
	}
	
	suite.service.On("UpdateMealSlot", mealPlanID, suite.userID, day, mealType, mock.MatchedBy(func(i *models.UpdateMealSlotInput) bool {
		return i.RecipeID != nil && *i.RecipeID == "recipe-123"
	})).Return(expectedResponse, nil)
	
	body, _ := json.Marshal(input)
	
	// Act
	url := "/api/v1/meal-plans/" + mealPlanID.String() + "/entries/" + day + "/" + mealType
	req := httptest.NewRequest("PUT", url, bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	var response models.MealPlanResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Equal(suite.T(), mealPlanID, response.ID)
	
	suite.service.AssertExpectations(suite.T())
}

func (suite *MealPlanHandlerTestSuite) TestDeleteMealSlot_Success() {
	// Arrange
	mealPlanID := uuid.New()
	day := "monday"
	mealType := "breakfast"
	
	expectedResponse := &models.MealPlanResponse{
		MealPlan: models.MealPlan{
			ID:     mealPlanID,
			UserID: suite.userID,
		},
	}
	
	suite.service.On("DeleteMealSlot", mealPlanID, suite.userID, day, mealType).Return(expectedResponse, nil)
	
	// Act
	url := "/api/v1/meal-plans/" + mealPlanID.String() + "/entries/" + day + "/" + mealType
	req := httptest.NewRequest("DELETE", url, nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	suite.service.AssertExpectations(suite.T())
}

func (suite *MealPlanHandlerTestSuite) TestDeleteMealPlan_Success() {
	// Arrange
	mealPlanID := uuid.New()
	
	suite.service.On("DeleteMealPlan", mealPlanID, suite.userID).Return(nil)
	
	// Act
	req := httptest.NewRequest("DELETE", "/api/v1/meal-plans/"+mealPlanID.String(), nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusNoContent, w.Code)
	
	suite.service.AssertExpectations(suite.T())
}

func (suite *MealPlanHandlerTestSuite) TestGetMealPlanByWeek_Success() {
	// Arrange
	weekStart := time.Now().Truncate(24 * time.Hour)
	dateStr := weekStart.Format("2006-01-02")
	
	expectedResponse := &models.MealPlanResponse{
		MealPlan: models.MealPlan{
			ID:            uuid.New(),
			UserID:        suite.userID,
			WeekStartDate: weekStart,
		},
	}
	
	suite.service.On("GetMealPlanByWeek", suite.userID, weekStart).Return(expectedResponse, nil)
	
	// Act
	req := httptest.NewRequest("GET", "/api/v1/meal-plans/week/"+dateStr, nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	var response models.MealPlanResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Equal(suite.T(), expectedResponse.ID, response.ID)
	
	suite.service.AssertExpectations(suite.T())
}

func (suite *MealPlanHandlerTestSuite) TestGetMealPlanByWeek_InvalidDate() {
	// Act
	req := httptest.NewRequest("GET", "/api/v1/meal-plans/week/invalid-date", nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	// Assert
	assert.Equal(suite.T(), http.StatusBadRequest, w.Code)
	
	var response map[string]interface{}
	err := json.Unmarshal(w.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Contains(suite.T(), response["error"], "Invalid date format")
}

// Run the test suite
func TestMealPlanHandlerSuite(t *testing.T) {
	suite.Run(t, new(MealPlanHandlerTestSuite))
}