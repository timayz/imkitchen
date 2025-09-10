package tests

import (
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/suite"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/services"
)

// Mock repositories
type MockShoppingListRepository struct {
	mock.Mock
}

func (m *MockShoppingListRepository) Create(list *models.ShoppingList) error {
	args := m.Called(list)
	return args.Error(0)
}

func (m *MockShoppingListRepository) CreateItems(items []models.ShoppingItem) error {
	args := m.Called(items)
	return args.Error(0)
}

func (m *MockShoppingListRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.ShoppingList, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.ShoppingList), args.Error(1)
}

func (m *MockShoppingListRepository) GetByUserID(userID uuid.UUID, status string, sortBy string) ([]models.ShoppingList, error) {
	args := m.Called(userID, status, sortBy)
	return args.Get(0).([]models.ShoppingList), args.Error(1)
}

func (m *MockShoppingListRepository) GetItemsByListID(listID uuid.UUID) ([]models.ShoppingItem, error) {
	args := m.Called(listID)
	return args.Get(0).([]models.ShoppingItem), args.Error(1)
}

func (m *MockShoppingListRepository) GetWithItems(id uuid.UUID, userID uuid.UUID) (*models.ShoppingList, []models.ShoppingItem, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.ShoppingList), args.Get(1).([]models.ShoppingItem), args.Error(2)
}

func (m *MockShoppingListRepository) UpdateItem(itemID uuid.UUID, updates *models.ShoppingItemUpdateRequest) error {
	args := m.Called(itemID, updates)
	return args.Error(0)
}

func (m *MockShoppingListRepository) UpdateStatus(listID uuid.UUID, userID uuid.UUID, status string) error {
	args := m.Called(listID, userID, status)
	return args.Error(0)
}

func (m *MockShoppingListRepository) Delete(listID uuid.UUID, userID uuid.UUID) error {
	args := m.Called(listID, userID)
	return args.Error(0)
}

func (m *MockShoppingListRepository) GetByMealPlanID(mealPlanID uuid.UUID) ([]models.ShoppingList, error) {
	args := m.Called(mealPlanID)
	return args.Get(0).([]models.ShoppingList), args.Error(1)
}

func (m *MockShoppingListRepository) GetActiveByUserID(userID uuid.UUID) ([]models.ShoppingList, error) {
	args := m.Called(userID)
	return args.Get(0).([]models.ShoppingList), args.Error(1)
}

type MockMealPlanRepository struct {
	mock.Mock
}

func (m *MockMealPlanRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.MealPlan, error) {
	args := m.Called(id, userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) Create(mealPlan *models.MealPlan) error {
	args := m.Called(mealPlan)
	return args.Error(0)
}

func (m *MockMealPlanRepository) GetByUserID(userID uuid.UUID, filters *models.MealPlanFilters) ([]models.MealPlan, error) {
	args := m.Called(userID, filters)
	return args.Get(0).([]models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) GetByWeekStart(userID uuid.UUID, weekStart time.Time) (*models.MealPlan, error) {
	args := m.Called(userID, weekStart)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) Update(id uuid.UUID, userID uuid.UUID, input *models.UpdateMealPlanInput) (*models.MealPlan, error) {
	args := m.Called(id, userID, input)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) Delete(id uuid.UUID, userID uuid.UUID) error {
	args := m.Called(id, userID)
	return args.Error(0)
}

func (m *MockMealPlanRepository) UpdateMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string, input *models.UpdateMealSlotInput) (*models.MealPlan, error) {
	args := m.Called(mealPlanID, userID, day, mealType, input)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

func (m *MockMealPlanRepository) DeleteMealSlot(mealPlanID uuid.UUID, userID uuid.UUID, day, mealType string) (*models.MealPlan, error) {
	args := m.Called(mealPlanID, userID, day, mealType)
	return args.Get(0).(*models.MealPlan), args.Error(1)
}

type MockRecipeRepository struct {
	mock.Mock
}

func (m *MockRecipeRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.Recipe, error) {
	args := m.Called(id, userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) Create(recipe *models.Recipe) error {
	args := m.Called(recipe)
	return args.Error(0)
}

func (m *MockRecipeRepository) GetByUserID(userID uuid.UUID, limit, offset int) ([]models.Recipe, error) {
	args := m.Called(userID, limit, offset)
	return args.Get(0).([]models.Recipe), args.Error(1)
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

type MockCacheService struct {
	mock.Mock
}

// ShoppingListServiceTestSuite test suite for shopping list service
type ShoppingListServiceTestSuite struct {
	suite.Suite
	service          *services.ShoppingListService
	shoppingRepo     *MockShoppingListRepository
	mealPlanRepo     *MockMealPlanRepository
	recipeRepo       *MockRecipeRepository
	cacheService     *MockCacheService
	userID           uuid.UUID
	mealPlanID       uuid.UUID
	listID           uuid.UUID
}

func (suite *ShoppingListServiceTestSuite) SetupTest() {
	suite.shoppingRepo = new(MockShoppingListRepository)
	suite.mealPlanRepo = new(MockMealPlanRepository)
	suite.recipeRepo = new(MockRecipeRepository)
	suite.cacheService = new(MockCacheService)

	suite.service = services.NewShoppingListService(
		suite.shoppingRepo,
		suite.mealPlanRepo,
		suite.recipeRepo,
		suite.cacheService,
	)

	suite.userID = uuid.New()
	suite.mealPlanID = uuid.New()
	suite.listID = uuid.New()
}

func (suite *ShoppingListServiceTestSuite) TestGenerateFromMealPlan_Success() {
	// Given
	mealPlan := &models.MealPlan{
		ID:        suite.mealPlanID,
		UserID:    suite.userID,
		WeekStart: time.Now(),
		Name:      "Test Meal Plan",
	}

	recipeID1 := uuid.New()
	recipeID2 := uuid.New()

	recipes := []*models.Recipe{
		{
			ID:     recipeID1,
			UserID: suite.userID,
			Title:  "Test Recipe 1",
			Ingredients: []models.Ingredient{
				{Name: "Chicken", Amount: 1, Unit: "pound", Category: models.CategoryProtein},
				{Name: "Rice", Amount: 2, Unit: "cup", Category: models.CategoryPantry},
			},
		},
		{
			ID:     recipeID2,
			UserID: suite.userID,
			Title:  "Test Recipe 2",
			Ingredients: []models.Ingredient{
				{Name: "Chicken", Amount: 0.5, Unit: "pound", Category: models.CategoryProtein},
				{Name: "Vegetables", Amount: 1, Unit: "cup", Category: models.CategoryProduce},
			},
		},
	}

	suite.mealPlanRepo.On("GetByID", suite.mealPlanID, suite.userID).Return(mealPlan, nil)
	suite.recipeRepo.On("GetByID", recipeID1, suite.userID).Return(recipes[0], nil)
	suite.recipeRepo.On("GetByID", recipeID2, suite.userID).Return(recipes[1], nil)
	suite.shoppingRepo.On("Create", mock.AnythingOfType("*models.ShoppingList")).Return(nil)
	suite.shoppingRepo.On("CreateItems", mock.AnythingOfType("[]models.ShoppingItem")).Return(nil)

	// When
	result, err := suite.service.GenerateFromMealPlan(suite.userID, suite.mealPlanID, false)

	// Then
	assert.NoError(suite.T(), err)
	assert.NotNil(suite.T(), result)
	assert.Equal(suite.T(), "Shopping List - Test Meal Plan", result.Name)
	assert.Equal(suite.T(), models.ShoppingListStatusActive, result.Status)

	// Verify mocks
	suite.mealPlanRepo.AssertExpectations(suite.T())
	suite.shoppingRepo.AssertExpectations(suite.T())
}

func (suite *ShoppingListServiceTestSuite) TestGenerateFromMealPlan_MealPlanNotFound() {
	// Given
	suite.mealPlanRepo.On("GetByID", suite.mealPlanID, suite.userID).Return((*models.MealPlan)(nil), nil)

	// When
	result, err := suite.service.GenerateFromMealPlan(suite.userID, suite.mealPlanID, false)

	// Then
	assert.Error(suite.T(), err)
	assert.Nil(suite.T(), result)
	assert.Contains(suite.T(), err.Error(), "meal plan not found")
}

func (suite *ShoppingListServiceTestSuite) TestGetShoppingList_Success() {
	// Given
	shoppingList := &models.ShoppingList{
		ID:     suite.listID,
		UserID: suite.userID,
		Name:   "Test Shopping List",
		Status: models.ShoppingListStatusActive,
	}

	items := []models.ShoppingItem{
		{
			ID:             uuid.New(),
			ShoppingListID: suite.listID,
			IngredientName: "Chicken",
			Amount:         1,
			Unit:           "pound",
			Category:       models.CategoryProtein,
		},
	}

	suite.shoppingRepo.On("GetWithItems", suite.listID, suite.userID).Return(shoppingList, items, nil)

	// When
	result, err := suite.service.GetShoppingList(suite.userID, suite.listID)

	// Then
	assert.NoError(suite.T(), err)
	assert.NotNil(suite.T(), result)
	assert.Equal(suite.T(), shoppingList.Name, result.Name)
	assert.Equal(suite.T(), 1, result.TotalItems)
}

func (suite *ShoppingListServiceTestSuite) TestUpdateItem_Success() {
	// Given
	shoppingList := &models.ShoppingList{
		ID:     suite.listID,
		UserID: suite.userID,
	}

	itemID := uuid.New()
	updates := &models.ShoppingItemUpdateRequest{
		IsCompleted: true,
		Notes:       stringPtr("Bought from grocery store"),
	}

	suite.shoppingRepo.On("GetByID", suite.listID, suite.userID).Return(shoppingList, nil)
	suite.shoppingRepo.On("UpdateItem", itemID, updates).Return(nil)

	// When
	err := suite.service.UpdateItem(suite.userID, suite.listID, itemID, updates)

	// Then
	assert.NoError(suite.T(), err)
	suite.shoppingRepo.AssertExpectations(suite.T())
}

func (suite *ShoppingListServiceTestSuite) TestExportShoppingList_JSON() {
	// Given
	shoppingList := &models.ShoppingList{
		ID:     suite.listID,
		UserID: suite.userID,
		Name:   "Test Shopping List",
		Status: models.ShoppingListStatusActive,
	}

	items := []models.ShoppingItem{
		{
			ID:             uuid.New(),
			ShoppingListID: suite.listID,
			IngredientName: "Chicken",
			Amount:         1,
			Unit:           "pound",
			Category:       models.CategoryProtein,
		},
	}

	suite.shoppingRepo.On("GetWithItems", suite.listID, suite.userID).Return(shoppingList, items, nil)

	// When
	data, filename, err := suite.service.ExportShoppingList(suite.userID, suite.listID, "json", false)

	// Then
	assert.NoError(suite.T(), err)
	assert.NotEmpty(suite.T(), data)
	assert.Contains(suite.T(), filename, ".json")
	assert.Contains(suite.T(), string(data), "Chicken")
}

func (suite *ShoppingListServiceTestSuite) TestExportShoppingList_CSV() {
	// Given
	shoppingList := &models.ShoppingList{
		ID:     suite.listID,
		UserID: suite.userID,
		Name:   "Test Shopping List",
		Status: models.ShoppingListStatusActive,
	}

	items := []models.ShoppingItem{
		{
			ID:             uuid.New(),
			ShoppingListID: suite.listID,
			IngredientName: "Chicken",
			Amount:         1,
			Unit:           "pound",
			Category:       models.CategoryProtein,
		},
	}

	suite.shoppingRepo.On("GetWithItems", suite.listID, suite.userID).Return(shoppingList, items, nil)

	// When
	data, filename, err := suite.service.ExportShoppingList(suite.userID, suite.listID, "csv", false)

	// Then
	assert.NoError(suite.T(), err)
	assert.NotEmpty(suite.T(), data)
	assert.Contains(suite.T(), filename, ".csv")
	assert.Contains(suite.T(), string(data), "Category,Item,Amount,Unit")
	assert.Contains(suite.T(), string(data), "Chicken")
}

func (suite *ShoppingListServiceTestSuite) TestDeleteShoppingList_Success() {
	// Given
	suite.shoppingRepo.On("Delete", suite.listID, suite.userID).Return(nil)

	// When
	err := suite.service.DeleteShoppingList(suite.userID, suite.listID)

	// Then
	assert.NoError(suite.T(), err)
	suite.shoppingRepo.AssertExpectations(suite.T())
}

// Helper function
func stringPtr(s string) *string {
	return &s
}

// TestShoppingListServiceTestSuite runs the test suite
func TestShoppingListServiceTestSuite(t *testing.T) {
	suite.Run(t, new(ShoppingListServiceTestSuite))
}