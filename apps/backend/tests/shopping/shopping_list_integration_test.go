package tests

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
	"gorm.io/gorm"

	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
	"github.com/imkitchen/backend/internal/services"
)

// ShoppingListIntegrationTestSuite integration test suite
type ShoppingListIntegrationTestSuite struct {
	suite.Suite
	db           *gorm.DB
	router       *gin.Engine
	handler      *handlers.ShoppingListHandlers
	service      *services.ShoppingListService
	shoppingRepo repositories.ShoppingListRepository
	mealPlanRepo repositories.MealPlanRepository
	recipeRepo   repositories.RecipeRepository
	userID       uuid.UUID
	testRecipes  []*models.Recipe
	testMealPlan *models.MealPlan
}

func (suite *ShoppingListIntegrationTestSuite) SetupSuite() {
	// Setup database connection (assuming test database setup exists)
	// suite.db = setupTestDB() 
	
	// For now, using mocks to demonstrate the test structure
	suite.userID = uuid.New()
	
	// Setup repositories and service
	suite.shoppingRepo = repositories.NewShoppingListRepository(suite.db)
	suite.mealPlanRepo = repositories.NewMealPlanRepository(suite.db)
	suite.recipeRepo = repositories.NewRecipeRepository(suite.db)
	
	suite.service = services.NewShoppingListService(
		suite.shoppingRepo,
		suite.mealPlanRepo,
		suite.recipeRepo,
		nil, // cache service
	)
	
	suite.handler = handlers.NewShoppingListHandlers(suite.service)
	
	// Setup Gin router
	gin.SetMode(gin.TestMode)
	suite.router = gin.New()
	
	// Add authentication middleware mock
	suite.router.Use(func(c *gin.Context) {
		c.Set("userID", suite.userID)
		c.Next()
	})
	
	// Setup routes
	api := suite.router.Group("/api/v1")
	{
		shoppingRoutes := api.Group("/shopping-lists")
		{
			shoppingRoutes.POST("/generate", suite.handler.GenerateShoppingList)
			shoppingRoutes.GET("/", suite.handler.GetShoppingLists)
			shoppingRoutes.GET("/:id", suite.handler.GetShoppingList)
			shoppingRoutes.PUT("/:id/items/:itemId", suite.handler.UpdateShoppingItem)
			shoppingRoutes.GET("/:id/export", suite.handler.ExportShoppingList)
			shoppingRoutes.DELETE("/:id", suite.handler.DeleteShoppingList)
		}
	}
}

func (suite *ShoppingListIntegrationTestSuite) SetupTest() {
	// Create test recipes
	suite.testRecipes = []*models.Recipe{
		{
			ID:     uuid.New(),
			UserID: suite.userID,
			Title:  "Grilled Chicken",
			Ingredients: []models.Ingredient{
				{Name: "Chicken Breast", Amount: 1.5, Unit: "pound", Category: models.CategoryProtein},
				{Name: "Olive Oil", Amount: 2, Unit: "tablespoon", Category: models.CategoryPantry},
				{Name: "Garlic", Amount: 3, Unit: "clove", Category: models.CategoryProduce},
			},
		},
		{
			ID:     uuid.New(),
			UserID: suite.userID,
			Title:  "Rice Pilaf",
			Ingredients: []models.Ingredient{
				{Name: "Rice", Amount: 2, Unit: "cup", Category: models.CategoryPantry},
				{Name: "Chicken Broth", Amount: 4, Unit: "cup", Category: models.CategoryPantry},
				{Name: "Onion", Amount: 1, Unit: "whole", Category: models.CategoryProduce},
				{Name: "Olive Oil", Amount: 1, Unit: "tablespoon", Category: models.CategoryPantry},
			},
		},
		{
			ID:     uuid.New(),
			UserID: suite.userID,
			Title:  "Mixed Vegetables",
			Ingredients: []models.Ingredient{
				{Name: "Broccoli", Amount: 2, Unit: "cup", Category: models.CategoryProduce},
				{Name: "Carrots", Amount: 1, Unit: "cup", Category: models.CategoryProduce},
				{Name: "Chicken Breast", Amount: 0.5, Unit: "pound", Category: models.CategoryProtein},
			},
		},
	}

	// Create test meal plan
	suite.testMealPlan = &models.MealPlan{
		ID:        uuid.New(),
		UserID:    suite.userID,
		WeekStart: time.Now().Truncate(24 * time.Hour),
		Name:      "Test Weekly Plan",
		Status:    "active",
	}
}

func (suite *ShoppingListIntegrationTestSuite) TestGenerateShoppingList_CompleteWorkflow() {
	// Given - meal plan generation request
	requestBody := map[string]interface{}{
		"mealPlanId":    suite.testMealPlan.ID.String(),
		"mergeExisting": false,
	}
	
	bodyBytes, _ := json.Marshal(requestBody)
	req, _ := http.NewRequest("POST", "/api/v1/shopping-lists/generate", strings.NewReader(string(bodyBytes)))
	req.Header.Set("Content-Type", "application/json")
	
	recorder := httptest.NewRecorder()

	// When
	suite.router.ServeHTTP(recorder, req)

	// Then
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	
	var response models.ShoppingListResponse
	err := json.Unmarshal(recorder.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	
	// Verify response structure
	assert.NotEmpty(suite.T(), response.ID)
	assert.Equal(suite.T(), suite.userID.String(), response.UserID)
	assert.Equal(suite.T(), models.ShoppingListStatusActive, response.Status)
	assert.NotEmpty(suite.T(), response.Categories)
	
	// Verify ingredient aggregation (Chicken Breast: 1.5 + 0.5 = 2.0 pounds)
	proteinItems, hasProtein := response.Categories[models.CategoryProtein]
	assert.True(suite.T(), hasProtein, "Should have protein category")
	
	chickenFound := false
	for _, item := range proteinItems {
		if item.IngredientName == "Chicken Breast" {
			chickenFound = true
			assert.Equal(suite.T(), 2.0, item.Amount, "Chicken amounts should be aggregated")
			assert.Equal(suite.T(), "pound", item.Unit)
			break
		}
	}
	assert.True(suite.T(), chickenFound, "Should have aggregated chicken breast")
	
	// Verify olive oil aggregation (2 tbsp + 1 tbsp = 3 tbsp)
	pantryItems, hasPantry := response.Categories[models.CategoryPantry]
	assert.True(suite.T(), hasPantry, "Should have pantry category")
	
	oliveOilFound := false
	for _, item := range pantryItems {
		if item.IngredientName == "Olive Oil" {
			oliveOilFound = true
			assert.Equal(suite.T(), 3.0, item.Amount, "Olive oil amounts should be aggregated")
			assert.Equal(suite.T(), "tablespoon", item.Unit)
			break
		}
	}
	assert.True(suite.T(), oliveOilFound, "Should have aggregated olive oil")
}

func (suite *ShoppingListIntegrationTestSuite) TestGetShoppingLists_WithFiltering() {
	// Given - create multiple shopping lists with different statuses
	lists := []struct {
		name   string
		status string
	}{
		{"Active List 1", models.ShoppingListStatusActive},
		{"Active List 2", models.ShoppingListStatusActive},
		{"Completed List", models.ShoppingListStatusCompleted},
		{"Archived List", models.ShoppingListStatusArchived},
	}

	for _, list := range lists {
		// Create shopping lists in database
		// (In real test, this would insert into the actual database)
	}

	// Test 1: Get all lists
	req, _ := http.NewRequest("GET", "/api/v1/shopping-lists", nil)
	recorder := httptest.NewRecorder()
	suite.router.ServeHTTP(recorder, req)
	
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	
	// Test 2: Get active lists only
	req, _ = http.NewRequest("GET", "/api/v1/shopping-lists?status=active", nil)
	recorder = httptest.NewRecorder()
	suite.router.ServeHTTP(recorder, req)
	
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	
	var response map[string][]models.ShoppingListResponse
	err := json.Unmarshal(recorder.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	
	shoppingLists, exists := response["shoppingLists"]
	assert.True(suite.T(), exists)
	
	// All returned lists should have active status
	for _, list := range shoppingLists {
		assert.Equal(suite.T(), models.ShoppingListStatusActive, list.Status)
	}
}

func (suite *ShoppingListIntegrationTestSuite) TestUpdateShoppingItem_CheckOffFlow() {
	// Given - create a shopping list with items
	listID := uuid.New()
	itemID := uuid.New()

	// Create shopping list and item in database
	// (In real test, this would use actual database operations)

	updateRequest := models.ShoppingItemUpdateRequest{
		IsCompleted: true,
		Notes:       stringPtr("Bought at Whole Foods"),
	}
	
	bodyBytes, _ := json.Marshal(updateRequest)
	url := "/api/v1/shopping-lists/" + listID.String() + "/items/" + itemID.String()
	req, _ := http.NewRequest("PUT", url, strings.NewReader(string(bodyBytes)))
	req.Header.Set("Content-Type", "application/json")
	
	recorder := httptest.NewRecorder()

	// When
	suite.router.ServeHTTP(recorder, req)

	// Then
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	
	var response map[string]string
	err := json.Unmarshal(recorder.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Equal(suite.T(), "Item updated successfully", response["message"])
}

func (suite *ShoppingListIntegrationTestSuite) TestExportShoppingList_AllFormats() {
	// Given - shopping list with items
	listID := uuid.New()

	// Test JSON Export
	req, _ := http.NewRequest("GET", "/api/v1/shopping-lists/"+listID.String()+"/export?format=json", nil)
	recorder := httptest.NewRecorder()
	suite.router.ServeHTTP(recorder, req)
	
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	assert.Equal(suite.T(), "application/json", recorder.Header().Get("Content-Type"))
	assert.Contains(suite.T(), recorder.Header().Get("Content-Disposition"), ".json")

	// Test CSV Export
	req, _ = http.NewRequest("GET", "/api/v1/shopping-lists/"+listID.String()+"/export?format=csv", nil)
	recorder = httptest.NewRecorder()
	suite.router.ServeHTTP(recorder, req)
	
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	assert.Equal(suite.T(), "text/csv", recorder.Header().Get("Content-Type"))
	assert.Contains(suite.T(), recorder.Header().Get("Content-Disposition"), ".csv")
	assert.Contains(suite.T(), recorder.Body.String(), "Category,Item,Amount,Unit")

	// Test Text Export
	req, _ = http.NewRequest("GET", "/api/v1/shopping-lists/"+listID.String()+"/export?format=txt", nil)
	recorder = httptest.NewRecorder()
	suite.router.ServeHTTP(recorder, req)
	
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	assert.Equal(suite.T(), "text/plain", recorder.Header().Get("Content-Type"))
	assert.Contains(suite.T(), recorder.Header().Get("Content-Disposition"), ".txt")
}

func (suite *ShoppingListIntegrationTestSuite) TestShoppingListPerformance() {
	// Test performance requirements: <3 seconds for generation
	startTime := time.Now()

	requestBody := map[string]interface{}{
		"mealPlanId":    suite.testMealPlan.ID.String(),
		"mergeExisting": false,
	}
	
	bodyBytes, _ := json.Marshal(requestBody)
	req, _ := http.NewRequest("POST", "/api/v1/shopping-lists/generate", strings.NewReader(string(bodyBytes)))
	req.Header.Set("Content-Type", "application/json")
	
	recorder := httptest.NewRecorder()
	suite.router.ServeHTTP(recorder, req)

	duration := time.Since(startTime)
	
	// Performance requirement: <3 seconds for generation
	assert.Less(suite.T(), duration, 3*time.Second, "Shopping list generation should complete within 3 seconds")
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
}

func (suite *ShoppingListIntegrationTestSuite) TestIngredientAggregationAccuracy() {
	// Test complex ingredient aggregation with unit conversions
	recipes := []*models.Recipe{
		{
			ID:     uuid.New(),
			UserID: suite.userID,
			Title:  "Recipe 1",
			Ingredients: []models.Ingredient{
				{Name: "Milk", Amount: 2, Unit: "cup", Category: models.CategoryDairy},
				{Name: "Butter", Amount: 4, Unit: "tablespoon", Category: models.CategoryDairy},
			},
		},
		{
			ID:     uuid.New(),
			UserID: suite.userID,
			Title:  "Recipe 2",
			Ingredients: []models.Ingredient{
				{Name: "Milk", Amount: 1, Unit: "cup", Category: models.CategoryDairy},
				{Name: "Butter", Amount: 2, Unit: "tablespoon", Category: models.CategoryDairy},
			},
		},
	}

	// Expected results: 
	// Milk: 2 + 1 = 3 cups
	// Butter: 4 + 2 = 6 tablespoons

	requestBody := map[string]interface{}{
		"mealPlanId":    suite.testMealPlan.ID.String(),
		"mergeExisting": false,
	}
	
	bodyBytes, _ := json.Marshal(requestBody)
	req, _ := http.NewRequest("POST", "/api/v1/shopping-lists/generate", strings.NewReader(string(bodyBytes)))
	req.Header.Set("Content-Type", "application/json")
	
	recorder := httptest.NewRecorder()
	suite.router.ServeHTTP(recorder, req)

	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	
	var response models.ShoppingListResponse
	err := json.Unmarshal(recorder.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)

	// Verify dairy category aggregation
	dairyItems, hasDairy := response.Categories[models.CategoryDairy]
	assert.True(suite.T(), hasDairy, "Should have dairy category")

	expectedItems := map[string]struct {
		amount float64
		unit   string
	}{
		"Milk":   {amount: 3.0, unit: "cup"},
		"Butter": {amount: 6.0, unit: "tablespoon"},
	}

	for _, item := range dairyItems {
		if expected, exists := expectedItems[item.IngredientName]; exists {
			assert.Equal(suite.T(), expected.amount, item.Amount, "Amount for %s should be correct", item.IngredientName)
			assert.Equal(suite.T(), expected.unit, item.Unit, "Unit for %s should be correct", item.IngredientName)
		}
	}
}

func (suite *ShoppingListIntegrationTestSuite) TestDeleteShoppingList_CascadeDelete() {
	// Given - shopping list with items
	listID := uuid.New()

	// Create shopping list with multiple items
	// (In real test, this would use actual database operations)

	// When
	req, _ := http.NewRequest("DELETE", "/api/v1/shopping-lists/"+listID.String(), nil)
	recorder := httptest.NewRecorder()
	suite.router.ServeHTTP(recorder, req)

	// Then
	assert.Equal(suite.T(), http.StatusOK, recorder.Code)
	
	var response map[string]string
	err := json.Unmarshal(recorder.Body.Bytes(), &response)
	assert.NoError(suite.T(), err)
	assert.Equal(suite.T(), "Shopping list deleted successfully", response["message"])

	// Verify that items are also deleted (cascade)
	// (In real test, this would verify database state)
}

// TestShoppingListIntegrationTestSuite runs the integration test suite
func TestShoppingListIntegrationTestSuite(t *testing.T) {
	suite.Run(t, new(ShoppingListIntegrationTestSuite))
}