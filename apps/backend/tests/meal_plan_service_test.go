package tests

import (
	"encoding/json"
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
type MockMealPlanRepository struct {
	mock.Mock
}

func (m *MockMealPlanRepository) Create(mealPlan *models.MealPlan) error {
	args := m.Called(mealPlan)
	return args.Error(0)
}

func (m *MockMealPlanRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.MealPlan, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.MealPlan), args.Error(1)
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

func (m *MockRecipeRepository) Create(recipe *models.Recipe) error {
	args := m.Called(recipe)
	return args.Error(0)
}

func (m *MockRecipeRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.Recipe, error) {
	args := m.Called(id, userID)
	return args.Get(0).(*models.Recipe), args.Error(1)
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

// Test Suite
type MealPlanServiceTestSuite struct {
	suite.Suite
	mealPlanRepo *MockMealPlanRepository
	recipeRepo   *MockRecipeRepository
	service      services.MealPlanService
	userID       uuid.UUID
}

func (suite *MealPlanServiceTestSuite) SetupTest() {
	suite.mealPlanRepo = new(MockMealPlanRepository)
	suite.recipeRepo = new(MockRecipeRepository)
	suite.service = services.NewMealPlanService(suite.mealPlanRepo, suite.recipeRepo)
	suite.userID = uuid.New()
}

func (suite *MealPlanServiceTestSuite) TestCreateMealPlan_Success() {
	// Arrange
	weekStart := time.Now().Truncate(24 * time.Hour)
	input := &models.CreateMealPlanInput{
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

	// Mock: No existing meal plan for this week
	suite.mealPlanRepo.On("GetByWeekStart", suite.userID, weekStart).Return((*models.MealPlan)(nil), assert.AnError)

	// Mock: Successful creation
	suite.mealPlanRepo.On("Create", mock.AnythingOfType("*models.MealPlan")).Return(nil)

	// Act
	result, err := suite.service.CreateMealPlan(suite.userID, input)

	// Assert
	assert.NoError(suite.T(), err)
	assert.NotNil(suite.T(), result)
	assert.Equal(suite.T(), suite.userID, result.UserID)
	assert.Equal(suite.T(), "manual", result.GenerationType)
	assert.Equal(suite.T(), "active", result.Status)
	
	suite.mealPlanRepo.AssertExpectations(suite.T())
}

func (suite *MealPlanServiceTestSuite) TestCreateMealPlan_DuplicateWeek() {
	// Arrange
	weekStart := time.Now().Truncate(24 * time.Hour)
	input := &models.CreateMealPlanInput{
		WeekStartDate:  weekStart,
		GenerationType: "manual",
		Meals:          models.WeeklyMeals{},
	}

	existingMealPlan := &models.MealPlan{
		ID:     uuid.New(),
		UserID: suite.userID,
	}

	// Mock: Existing meal plan for this week
	suite.mealPlanRepo.On("GetByWeekStart", suite.userID, weekStart).Return(existingMealPlan, nil)

	// Act
	result, err := suite.service.CreateMealPlan(suite.userID, input)

	// Assert
	assert.Error(suite.T(), err)
	assert.Nil(suite.T(), result)
	assert.Contains(suite.T(), err.Error(), "meal plan already exists")
	
	suite.mealPlanRepo.AssertExpectations(suite.T())
}

func (suite *MealPlanServiceTestSuite) TestGetMealPlan_Success() {
	// Arrange
	mealPlanID := uuid.New()
	expectedMealPlan := &models.MealPlan{
		ID:     mealPlanID,
		UserID: suite.userID,
		Meals:  json.RawMessage(`{"monday":[],"tuesday":[],"wednesday":[],"thursday":[],"friday":[],"saturday":[],"sunday":[]}`),
	}

	// Mock repository call
	suite.mealPlanRepo.On("GetByID", mealPlanID, suite.userID).Return(expectedMealPlan, nil)

	// Act
	result, err := suite.service.GetMealPlan(mealPlanID, suite.userID)

	// Assert
	assert.NoError(suite.T(), err)
	assert.NotNil(suite.T(), result)
	assert.Equal(suite.T(), mealPlanID, result.ID)
	assert.Equal(suite.T(), suite.userID, result.UserID)
	
	suite.mealPlanRepo.AssertExpectations(suite.T())
}

func (suite *MealPlanServiceTestSuite) TestUpdateMealSlot_Success() {
	// Arrange
	mealPlanID := uuid.New()
	recipeID := uuid.New()
	day := "monday"
	mealType := "breakfast"
	
	input := &models.UpdateMealSlotInput{
		RecipeID: &recipeID.String(),
		Servings: &[]int{2}[0],
	}

	recipe := &models.Recipe{
		ID:     recipeID,
		UserID: suite.userID,
		Title:  "Test Recipe",
	}

	updatedMealPlan := &models.MealPlan{
		ID:     mealPlanID,
		UserID: suite.userID,
	}

	// Mock recipe validation
	suite.recipeRepo.On("GetByID", recipeID, suite.userID).Return(recipe, nil)

	// Mock meal slot update
	suite.mealPlanRepo.On("UpdateMealSlot", mealPlanID, suite.userID, day, mealType, input).Return(updatedMealPlan, nil)

	// Act
	result, err := suite.service.UpdateMealSlot(mealPlanID, suite.userID, day, mealType, input)

	// Assert
	assert.NoError(suite.T(), err)
	assert.NotNil(suite.T(), result)
	
	suite.mealPlanRepo.AssertExpectations(suite.T())
	suite.recipeRepo.AssertExpectations(suite.T())
}

func (suite *MealPlanServiceTestSuite) TestUpdateMealSlot_InvalidRecipe() {
	// Arrange
	mealPlanID := uuid.New()
	recipeID := uuid.New()
	day := "monday"
	mealType := "breakfast"
	
	input := &models.UpdateMealSlotInput{
		RecipeID: &recipeID.String(),
	}

	// Mock recipe not found
	suite.recipeRepo.On("GetByID", recipeID, suite.userID).Return((*models.Recipe)(nil), assert.AnError)

	// Act
	result, err := suite.service.UpdateMealSlot(mealPlanID, suite.userID, day, mealType, input)

	// Assert
	assert.Error(suite.T(), err)
	assert.Nil(suite.T(), result)
	assert.Contains(suite.T(), err.Error(), "recipe not found")
	
	suite.recipeRepo.AssertExpectations(suite.T())
}

func (suite *MealPlanServiceTestSuite) TestValidateMealType_Invalid() {
	// Arrange
	mealPlanID := uuid.New()
	day := "monday"
	invalidMealType := "brunch" // Invalid meal type
	
	input := &models.UpdateMealSlotInput{
		Servings: &[]int{2}[0],
	}

	// Act
	result, err := suite.service.UpdateMealSlot(mealPlanID, suite.userID, day, invalidMealType, input)

	// Assert
	assert.Error(suite.T(), err)
	assert.Nil(suite.T(), result)
	assert.Contains(suite.T(), err.Error(), "invalid meal type")
}

func (suite *MealPlanServiceTestSuite) TestValidateDay_Invalid() {
	// Arrange
	mealPlanID := uuid.New()
	invalidDay := "funday" // Invalid day
	mealType := "breakfast"
	
	input := &models.UpdateMealSlotInput{
		Servings: &[]int{2}[0],
	}

	// Act
	result, err := suite.service.UpdateMealSlot(mealPlanID, suite.userID, invalidDay, mealType, input)

	// Assert
	assert.Error(suite.T(), err)
	assert.Nil(suite.T(), result)
	assert.Contains(suite.T(), err.Error(), "invalid day")
}

func (suite *MealPlanServiceTestSuite) TestDeleteMealPlan_Success() {
	// Arrange
	mealPlanID := uuid.New()

	// Mock successful deletion
	suite.mealPlanRepo.On("Delete", mealPlanID, suite.userID).Return(nil)

	// Act
	err := suite.service.DeleteMealPlan(mealPlanID, suite.userID)

	// Assert
	assert.NoError(suite.T(), err)
	
	suite.mealPlanRepo.AssertExpectations(suite.T())
}

// Run the test suite
func TestMealPlanServiceSuite(t *testing.T) {
	suite.Run(t, new(MealPlanServiceTestSuite))
}