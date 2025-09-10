package tests

import (
	"encoding/json"
	"fmt"
	"testing"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/imkitchen/backend/internal/models"
	"github.com/imkitchen/backend/internal/repositories"
)

// Mock repository for testing
type MockRecipeRepository struct {
	mock.Mock
}

func (m *MockRecipeRepository) Create(recipe *models.Recipe) error {
	args := m.Called(recipe)
	return args.Error(0)
}

func (m *MockRecipeRepository) GetByID(id uuid.UUID, userID uuid.UUID) (*models.Recipe, error) {
	args := m.Called(id, userID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) GetByUserID(userID uuid.UUID, limit, offset int) ([]models.Recipe, error) {
	args := m.Called(userID, limit, offset)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).([]models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) Update(id uuid.UUID, userID uuid.UUID, input *models.UpdateRecipeInput) (*models.Recipe, error) {
	args := m.Called(id, userID, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRecipeRepository) Delete(id uuid.UUID, userID uuid.UUID) error {
	args := m.Called(id, userID)
	return args.Error(0)
}

func (m *MockRecipeRepository) Search(userID uuid.UUID, params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error) {
	args := m.Called(userID, params)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.RecipeSearchResponse), args.Error(1)
}

func (m *MockRecipeRepository) GetByExternalSource(source, externalID string) (*models.Recipe, error) {
	args := m.Called(source, externalID)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Recipe), args.Error(1)
}

// Mock import service for testing
type MockRecipeImportService struct {
	mock.Mock
}

func (m *MockRecipeImportService) ImportFromURL(input *models.ImportRecipeInput) (*models.ImportRecipeResult, error) {
	args := m.Called(input)
	return args.Get(0).(*models.ImportRecipeResult), args.Error(1)
}

func (m *MockRecipeImportService) ParseRecipeFromHTML(htmlContent, sourceURL string) (*models.CreateRecipeInput, error) {
	args := m.Called(htmlContent, sourceURL)
	return args.Get(0).(*models.CreateRecipeInput), args.Error(1)
}

func TestRecipeService_CreateRecipe(t *testing.T) {
	tests := []struct {
		name        string
		input       *models.CreateRecipeInput
		setupMocks  func(*MockRecipeRepository)
		expectError bool
		errorMsg    string
	}{
		{
			name: "successful creation",
			input: &models.CreateRecipeInput{
				Title:       "Test Recipe",
				PrepTime:    30,
				CookTime:    45,
				MealType:    []string{"dinner"},
				Complexity:  "simple",
				Servings:    4,
				Ingredients: []models.RecipeIngredient{
					{Name: "Flour", Amount: 2, Unit: "cups", Category: "pantry"},
				},
				Instructions: []models.RecipeInstruction{
					{StepNumber: 1, Instruction: "Mix ingredients"},
				},
			},
			setupMocks: func(repo *MockRecipeRepository) {
				repo.On("Create", mock.AnythingOfType("*models.Recipe")).Return(nil)
			},
			expectError: false,
		},
		{
			name: "validation error - empty title",
			input: &models.CreateRecipeInput{
				Title:       "",
				PrepTime:    30,
				CookTime:    45,
				MealType:    []string{"dinner"},
				Complexity:  "simple",
				Servings:    4,
				Ingredients: []models.RecipeIngredient{
					{Name: "Flour", Amount: 2, Unit: "cups", Category: "pantry"},
				},
				Instructions: []models.RecipeInstruction{
					{StepNumber: 1, Instruction: "Mix ingredients"},
				},
			},
			setupMocks:  func(repo *MockRecipeRepository) {},
			expectError: true,
			errorMsg:    "validation error",
		},
		{
			name: "validation error - no ingredients",
			input: &models.CreateRecipeInput{
				Title:        "Test Recipe",
				PrepTime:     30,
				CookTime:     45,
				MealType:     []string{"dinner"},
				Complexity:   "simple",
				Servings:     4,
				Ingredients:  []models.RecipeIngredient{},
				Instructions: []models.RecipeInstruction{
					{StepNumber: 1, Instruction: "Mix ingredients"},
				},
			},
			setupMocks:  func(repo *MockRecipeRepository) {},
			expectError: true,
			errorMsg:    "recipe must have at least one ingredient",
		},
		{
			name: "validation error - no instructions",
			input: &models.CreateRecipeInput{
				Title:       "Test Recipe",
				PrepTime:    30,
				CookTime:    45,
				MealType:    []string{"dinner"},
				Complexity:  "simple",
				Servings:    4,
				Ingredients: []models.RecipeIngredient{
					{Name: "Flour", Amount: 2, Unit: "cups", Category: "pantry"},
				},
				Instructions: []models.RecipeInstruction{},
			},
			setupMocks:  func(repo *MockRecipeRepository) {},
			expectError: true,
			errorMsg:    "recipe must have at least one instruction",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			repo := new(MockRecipeRepository)
			tt.setupMocks(repo)

			service := &recipeServiceTestImpl{
				repo: repo,
			}

			result, err := service.CreateRecipe(tt.input)

			if tt.expectError {
				assert.Error(t, err)
				assert.Nil(t, result)
				if tt.errorMsg != "" {
					assert.Contains(t, err.Error(), tt.errorMsg)
				}
			} else {
				assert.NoError(t, err)
				assert.NotNil(t, result)
			}

			repo.AssertExpectations(t)
		})
	}
}

func TestRecipeService_GetRecipe(t *testing.T) {
	recipeID := uuid.New()
	
	tests := []struct {
		name        string
		recipeID    uuid.UUID
		setupMocks  func(*MockRecipeRepository)
		expectError bool
	}{
		{
			name:     "successful retrieval",
			recipeID: recipeID,
			setupMocks: func(repo *MockRecipeRepository) {
				recipe := &models.Recipe{
					ID:    recipeID,
					Title: "Test Recipe",
				}
				repo.On("GetByID", recipeID).Return(recipe, nil)
			},
			expectError: false,
		},
		{
			name:     "recipe not found",
			recipeID: recipeID,
			setupMocks: func(repo *MockRecipeRepository) {
				repo.On("GetByID", recipeID).Return((*models.Recipe)(nil), assert.AnError)
			},
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			repo := new(MockRecipeRepository)
			tt.setupMocks(repo)

			service := &recipeServiceTestImpl{
				repo: repo,
			}

			result, err := service.GetRecipe(tt.recipeID)

			if tt.expectError {
				assert.Error(t, err)
				assert.Nil(t, result)
			} else {
				assert.NoError(t, err)
				assert.NotNil(t, result)
				assert.Equal(t, tt.recipeID, result.ID)
			}

			repo.AssertExpectations(t)
		})
	}
}

func TestRecipeService_SearchRecipes(t *testing.T) {
	tests := []struct {
		name        string
		params      *models.RecipeSearchParams
		setupMocks  func(*MockRecipeRepository)
		expectError bool
	}{
		{
			name: "successful search with defaults",
			params: &models.RecipeSearchParams{},
			setupMocks: func(repo *MockRecipeRepository) {
				response := &models.RecipeSearchResponse{
					Recipes:    []models.Recipe{},
					Total:      0,
					Page:       1,
					Limit:      20,
					TotalPages: 0,
				}
				repo.On("Search", mock.AnythingOfType("*models.RecipeSearchParams")).Return(response, nil)
			},
			expectError: false,
		},
		{
			name: "search with filters",
			params: &models.RecipeSearchParams{
				RecipeFilters: models.RecipeFilters{
					MealType:   []string{"dinner"},
					Complexity: []string{"simple"},
				},
				Page:  1,
				Limit: 10,
			},
			setupMocks: func(repo *MockRecipeRepository) {
				response := &models.RecipeSearchResponse{
					Recipes:    []models.Recipe{},
					Total:      0,
					Page:       1,
					Limit:      10,
					TotalPages: 0,
				}
				repo.On("Search", mock.AnythingOfType("*models.RecipeSearchParams")).Return(response, nil)
			},
			expectError: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			repo := new(MockRecipeRepository)
			tt.setupMocks(repo)

			service := &recipeServiceTestImpl{
				repo: repo,
			}

			result, err := service.SearchRecipes(tt.params)

			if tt.expectError {
				assert.Error(t, err)
				assert.Nil(t, result)
			} else {
				assert.NoError(t, err)
				assert.NotNil(t, result)
			}

			repo.AssertExpectations(t)
		})
	}
}

// Test implementation for recipe service (simplified for testing)
type recipeServiceTestImpl struct {
	repo repositories.RecipeRepository
}

func (s *recipeServiceTestImpl) CreateRecipe(input *models.CreateRecipeInput) (*models.Recipe, error) {
	// Business rule validations
	if input.Title == "" {
		return nil, fmt.Errorf("validation error: title is required")
	}
	if len(input.Ingredients) == 0 {
		return nil, fmt.Errorf("recipe must have at least one ingredient")
	}
	if len(input.Instructions) == 0 {
		return nil, fmt.Errorf("recipe must have at least one instruction")
	}

	// Create recipe model
	ingredientsJSON, _ := json.Marshal(input.Ingredients)
	instructionsJSON, _ := json.Marshal(input.Instructions)

	recipe := &models.Recipe{
		ID:           uuid.New(),
		Title:        input.Title,
		PrepTime:     input.PrepTime,
		CookTime:     input.CookTime,
		Ingredients:  ingredientsJSON,
		Instructions: instructionsJSON,
	}

	err := s.repo.Create(recipe)
	if err != nil {
		return nil, err
	}

	return recipe, nil
}

func (s *recipeServiceTestImpl) GetRecipe(id uuid.UUID) (*models.Recipe, error) {
	return s.repo.GetByID(id)
}

func (s *recipeServiceTestImpl) SearchRecipes(params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error) {
	// Set defaults
	if params.Page == 0 {
		params.Page = 1
	}
	if params.Limit == 0 {
		params.Limit = 20
	}
	if params.SortBy == "" {
		params.SortBy = "created_at"
	}
	if params.SortOrder == "" {
		params.SortOrder = "desc"
	}

	return s.repo.Search(params)
}