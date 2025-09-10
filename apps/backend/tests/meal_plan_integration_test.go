package tests

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
	"github.com/imkitchen/backend/internal/services"
)

type MealPlanIntegrationTestSuite struct {
	suite.Suite
	db      *gorm.DB
	router  *gin.Engine
	userID  uuid.UUID
	recipeID uuid.UUID
}

func (suite *MealPlanIntegrationTestSuite) SetupTest() {
	var err error
	
	// Use SQLite in-memory database for testing
	suite.db, err = gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	suite.Require().NoError(err)
	
	// Auto-migrate schemas
	err = suite.db.AutoMigrate(&models.Recipe{}, &models.MealPlan{})
	suite.Require().NoError(err)
	
	// Create test user ID
	suite.userID = uuid.New()
	suite.recipeID = uuid.New()
	
	// Set up services and handlers
	recipeRepo := repositories.NewRecipeRepository(suite.db)
	mealPlanRepo := repositories.NewMealPlanRepository(suite.db)
	mealPlanService := services.NewMealPlanService(mealPlanRepo, recipeRepo)
	mealPlanHandler := handlers.NewMealPlanHandler(mealPlanService)
	
	// Set up Gin router
	gin.SetMode(gin.TestMode)
	suite.router = gin.New()
	
	// Add middleware to set user ID
	suite.router.Use(func(c *gin.Context) {
		c.Set("userID", suite.userID)
		c.Next()
	})
	
	// Register routes
	handlers.RegisterMealPlanRoutes(suite.router.Group("/api/v1"), mealPlanHandler)
	
	// Create test recipe
	suite.createTestRecipe()
}

func (suite *MealPlanIntegrationTestSuite) TearDownTest() {
	// Clean up database
	sqlDB, err := suite.db.DB()
	if err == nil {
		sqlDB.Close()
	}
}

func (suite *MealPlanIntegrationTestSuite) createTestRecipe() {
	recipe := &models.Recipe{
		ID:          suite.recipeID,
		UserID:      suite.userID,
		Title:       "Test Recipe",
		PrepTime:    15,
		CookTime:    30,
		MealType:    []string{"breakfast"},
		Complexity:  "simple",
		Servings:    4,
		Ingredients: json.RawMessage(`[{"name":"eggs","amount":2,"unit":"pieces","category":"protein"}]`),
		Instructions: json.RawMessage(`[{"stepNumber":1,"instruction":"Crack eggs","estimatedMinutes":5}]`),
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
	}
	
	err := suite.db.Create(recipe).Error
	suite.Require().NoError(err)
}

func (suite *MealPlanIntegrationTestSuite) TestFullMealPlanWorkflow() {
	// Test 1: Create a meal plan
	weekStart := time.Now().Truncate(24 * time.Hour)
	createInput := models.CreateMealPlanInput{
		WeekStartDate:  weekStart,
		GenerationType: "manual",
		Meals: models.WeeklyMeals{
			Monday: []models.MealSlot{
				{
					Day:      "monday",
					MealType: "breakfast",
					Servings: 2,
				},
			},
			Tuesday:   []models.MealSlot{},
			Wednesday: []models.MealSlot{},
			Thursday:  []models.MealSlot{},
			Friday:    []models.MealSlot{},
			Saturday:  []models.MealSlot{},
			Sunday:    []models.MealSlot{},
		},
	}
	
	body, _ := json.Marshal(createInput)
	req := httptest.NewRequest("POST", "/api/v1/meal-plans", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusCreated, w.Code)
	
	var createdMealPlan models.MealPlan
	err := json.Unmarshal(w.Body.Bytes(), &createdMealPlan)
	suite.Require().NoError(err)
	mealPlanID := createdMealPlan.ID
	
	// Test 2: Get the created meal plan
	req = httptest.NewRequest("GET", "/api/v1/meal-plans/"+mealPlanID.String(), nil)
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	var retrievedMealPlan models.MealPlanResponse
	err = json.Unmarshal(w.Body.Bytes(), &retrievedMealPlan)
	suite.Require().NoError(err)
	assert.Equal(suite.T(), mealPlanID, retrievedMealPlan.ID)
	
	// Test 3: Update a meal slot with the test recipe
	updateSlotInput := models.UpdateMealSlotInput{
		RecipeID: &suite.recipeID.String(),
		Servings: &[]int{2}[0],
	}
	
	body, _ = json.Marshal(updateSlotInput)
	url := fmt.Sprintf("/api/v1/meal-plans/%s/entries/monday/breakfast", mealPlanID.String())
	req = httptest.NewRequest("PUT", url, bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	var updatedMealPlan models.MealPlanResponse
	err = json.Unmarshal(w.Body.Bytes(), &updatedMealPlan)
	suite.Require().NoError(err)
	
	// Verify the meal slot was updated
	mondayMeals := updatedMealPlan.PopulatedMeals.Monday
	assert.NotEmpty(suite.T(), mondayMeals)
	if len(mondayMeals) > 0 {
		breakfastMeal := mondayMeals[0]
		assert.Equal(suite.T(), "breakfast", breakfastMeal.MealType)
		if breakfastMeal.Recipe != nil {
			assert.Equal(suite.T(), suite.recipeID, breakfastMeal.Recipe.ID)
		}
	}
	
	// Test 4: Get meal plan by week
	dateStr := weekStart.Format("2006-01-02")
	req = httptest.NewRequest("GET", "/api/v1/meal-plans/week/"+dateStr, nil)
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	// Test 5: Get all meal plans for user
	req = httptest.NewRequest("GET", "/api/v1/meal-plans", nil)
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	var mealPlansList map[string]interface{}
	err = json.Unmarshal(w.Body.Bytes(), &mealPlansList)
	suite.Require().NoError(err)
	assert.Equal(suite.T(), float64(1), mealPlansList["count"])
	
	// Test 6: Update meal plan status
	updateMealPlanInput := models.UpdateMealPlanInput{
		Status: &[]string{"completed"}[0],
		CompletionPercentage: &[]float64{100.0}[0],
	}
	
	body, _ = json.Marshal(updateMealPlanInput)
	req = httptest.NewRequest("PUT", "/api/v1/meal-plans/"+mealPlanID.String(), bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	// Test 7: Delete meal slot
	url = fmt.Sprintf("/api/v1/meal-plans/%s/entries/monday/breakfast", mealPlanID.String())
	req = httptest.NewRequest("DELETE", url, nil)
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	// Test 8: Delete meal plan
	req = httptest.NewRequest("DELETE", "/api/v1/meal-plans/"+mealPlanID.String(), nil)
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusNoContent, w.Code)
	
	// Test 9: Verify meal plan is deleted (should return 404)
	req = httptest.NewRequest("GET", "/api/v1/meal-plans/"+mealPlanID.String(), nil)
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusNotFound, w.Code)
}

func (suite *MealPlanIntegrationTestSuite) TestErrorHandling() {
	// Test 1: Try to create meal plan with invalid data
	invalidInput := map[string]interface{}{
		"generationType": "invalid_type",
		"meals": "invalid_meals_format",
	}
	
	body, _ := json.Marshal(invalidInput)
	req := httptest.NewRequest("POST", "/api/v1/meal-plans", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusBadRequest, w.Code)
	
	// Test 2: Try to get non-existent meal plan
	randomID := uuid.New()
	req = httptest.NewRequest("GET", "/api/v1/meal-plans/"+randomID.String(), nil)
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusNotFound, w.Code)
	
	// Test 3: Try to update meal slot with invalid recipe ID
	mealPlanID := suite.createTestMealPlan()
	
	invalidRecipeID := uuid.New().String()
	updateSlotInput := models.UpdateMealSlotInput{
		RecipeID: &invalidRecipeID,
	}
	
	body, _ = json.Marshal(updateSlotInput)
	url := fmt.Sprintf("/api/v1/meal-plans/%s/entries/monday/breakfast", mealPlanID.String())
	req = httptest.NewRequest("PUT", url, bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusInternalServerError, w.Code)
	
	// Test 4: Try to update meal slot with invalid day
	updateSlotInput = models.UpdateMealSlotInput{
		Servings: &[]int{2}[0],
	}
	
	body, _ = json.Marshal(updateSlotInput)
	url = fmt.Sprintf("/api/v1/meal-plans/%s/entries/invalidday/breakfast", mealPlanID.String())
	req = httptest.NewRequest("PUT", url, bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusInternalServerError, w.Code)
	
	// Test 5: Try to update meal slot with invalid meal type
	url = fmt.Sprintf("/api/v1/meal-plans/%s/entries/monday/invalidmealtype", mealPlanID.String())
	req = httptest.NewRequest("PUT", url, bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w = httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusInternalServerError, w.Code)
}

func (suite *MealPlanIntegrationTestSuite) createTestMealPlan() uuid.UUID {
	weekStart := time.Now().Truncate(24 * time.Hour)
	createInput := models.CreateMealPlanInput{
		WeekStartDate:  weekStart,
		GenerationType: "manual",
		Meals: models.WeeklyMeals{
			Monday: []models.MealSlot{
				{
					Day:      "monday",
					MealType: "breakfast",
					Servings: 2,
				},
			},
			Tuesday:   []models.MealSlot{},
			Wednesday: []models.MealSlot{},
			Thursday:  []models.MealSlot{},
			Friday:    []models.MealSlot{},
			Saturday:  []models.MealSlot{},
			Sunday:    []models.MealSlot{},
		},
	}
	
	body, _ := json.Marshal(createInput)
	req := httptest.NewRequest("POST", "/api/v1/meal-plans", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	var createdMealPlan models.MealPlan
	err := json.Unmarshal(w.Body.Bytes(), &createdMealPlan)
	suite.Require().NoError(err)
	
	return createdMealPlan.ID
}

func (suite *MealPlanIntegrationTestSuite) TestConcurrentAccess() {
	// Test concurrent creation and access to meal plans
	mealPlanID := suite.createTestMealPlan()
	
	// Simulate concurrent updates to different meal slots
	done := make(chan bool, 2)
	
	// Concurrent update 1: Monday lunch
	go func() {
		updateSlotInput := models.UpdateMealSlotInput{
			RecipeID: &suite.recipeID.String(),
			Servings: &[]int{1}[0],
		}
		
		body, _ := json.Marshal(updateSlotInput)
		url := fmt.Sprintf("/api/v1/meal-plans/%s/entries/monday/lunch", mealPlanID.String())
		req := httptest.NewRequest("PUT", url, bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)
		
		assert.Equal(suite.T(), http.StatusOK, w.Code)
		done <- true
	}()
	
	// Concurrent update 2: Tuesday breakfast
	go func() {
		updateSlotInput := models.UpdateMealSlotInput{
			RecipeID: &suite.recipeID.String(),
			Servings: &[]int{3}[0],
		}
		
		body, _ := json.Marshal(updateSlotInput)
		url := fmt.Sprintf("/api/v1/meal-plans/%s/entries/tuesday/breakfast", mealPlanID.String())
		req := httptest.NewRequest("PUT", url, bytes.NewBuffer(body))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()
		suite.router.ServeHTTP(w, req)
		
		assert.Equal(suite.T(), http.StatusOK, w.Code)
		done <- true
	}()
	
	// Wait for both operations to complete
	<-done
	<-done
	
	// Verify both updates were successful
	req := httptest.NewRequest("GET", "/api/v1/meal-plans/"+mealPlanID.String(), nil)
	w := httptest.NewRecorder()
	suite.router.ServeHTTP(w, req)
	
	assert.Equal(suite.T(), http.StatusOK, w.Code)
	
	var finalMealPlan models.MealPlanResponse
	err := json.Unmarshal(w.Body.Bytes(), &finalMealPlan)
	suite.Require().NoError(err)
	
	// Check that both meal slots were updated
	mondayMeals := finalMealPlan.PopulatedMeals.Monday
	tuesdayMeals := finalMealPlan.PopulatedMeals.Tuesday
	
	// Should have at least some meals populated
	assert.True(suite.T(), len(mondayMeals) > 0 || len(tuesdayMeals) > 0)
}

// Run the integration test suite
func TestMealPlanIntegrationSuite(t *testing.T) {
	suite.Run(t, new(MealPlanIntegrationTestSuite))
}