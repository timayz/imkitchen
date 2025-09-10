package tests

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/models"
)

// Mock recipe service for testing handlers
type MockRecipeService struct {
	mock.Mock
}

func (m *MockRecipeService) CreateRecipe(input *models.CreateRecipeInput) (*models.Recipe, error) {
	args := m.Called(input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRecipeService) GetRecipe(id uuid.UUID) (*models.Recipe, error) {
	args := m.Called(id)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRecipeService) UpdateRecipe(id uuid.UUID, input *models.UpdateRecipeInput) (*models.Recipe, error) {
	args := m.Called(id, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.Recipe), args.Error(1)
}

func (m *MockRecipeService) DeleteRecipe(id uuid.UUID) error {
	args := m.Called(id)
	return args.Error(0)
}

func (m *MockRecipeService) SearchRecipes(params *models.RecipeSearchParams) (*models.RecipeSearchResponse, error) {
	args := m.Called(params)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.RecipeSearchResponse), args.Error(1)
}

func (m *MockRecipeService) ImportRecipe(input *models.ImportRecipeInput) (*models.ImportRecipeResult, error) {
	args := m.Called(input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*models.ImportRecipeResult), args.Error(1)
}

func TestRecipeHandler_CreateRecipe(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		requestBody    interface{}
		setupMocks     func(*MockRecipeService)
		expectedStatus int
		expectedError  bool
	}{
		{
			name: "successful creation",
			requestBody: models.CreateRecipeInput{
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
			setupMocks: func(service *MockRecipeService) {
				recipe := &models.Recipe{
					ID:    uuid.New(),
					Title: "Test Recipe",
				}
				service.On("CreateRecipe", mock.AnythingOfType("*models.CreateRecipeInput")).Return(recipe, nil)
			},
			expectedStatus: http.StatusCreated,
			expectedError:  false,
		},
		{
			name:        "invalid JSON",
			requestBody: "invalid json",
			setupMocks:  func(service *MockRecipeService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  true,
		},
		{
			name: "service error",
			requestBody: models.CreateRecipeInput{
				Title: "Test Recipe",
			},
			setupMocks: func(service *MockRecipeService) {
				service.On("CreateRecipe", mock.AnythingOfType("*models.CreateRecipeInput")).Return(nil, assert.AnError)
			},
			expectedStatus: http.StatusInternalServerError,
			expectedError:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			service := new(MockRecipeService)
			tt.setupMocks(service)

			handler := handlers.NewRecipeHandler(service)
			router := gin.New()
			router.POST("/recipes", handler.CreateRecipe)

			body, _ := json.Marshal(tt.requestBody)
			req := httptest.NewRequest(http.MethodPost, "/recipes", bytes.NewBuffer(body))
			req.Header.Set("Content-Type", "application/json")

			w := httptest.NewRecorder()
			router.ServeHTTP(w, req)

			assert.Equal(t, tt.expectedStatus, w.Code)

			if tt.expectedError {
				var response map[string]interface{}
				json.Unmarshal(w.Body.Bytes(), &response)
				assert.Contains(t, response, "error")
			}

			service.AssertExpectations(t)
		})
	}
}

func TestRecipeHandler_GetRecipe(t *testing.T) {
	gin.SetMode(gin.TestMode)

	recipeID := uuid.New()

	tests := []struct {
		name           string
		recipeID       string
		setupMocks     func(*MockRecipeService)
		expectedStatus int
		expectedError  bool
	}{
		{
			name:     "successful retrieval",
			recipeID: recipeID.String(),
			setupMocks: func(service *MockRecipeService) {
				recipe := &models.Recipe{
					ID:    recipeID,
					Title: "Test Recipe",
				}
				service.On("GetRecipe", recipeID).Return(recipe, nil)
			},
			expectedStatus: http.StatusOK,
			expectedError:  false,
		},
		{
			name:     "invalid UUID",
			recipeID: "invalid-uuid",
			setupMocks: func(service *MockRecipeService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  true,
		},
		{
			name:     "recipe not found",
			recipeID: recipeID.String(),
			setupMocks: func(service *MockRecipeService) {
				service.On("GetRecipe", recipeID).Return(nil, assert.AnError)
			},
			expectedStatus: http.StatusInternalServerError,
			expectedError:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			service := new(MockRecipeService)
			tt.setupMocks(service)

			handler := handlers.NewRecipeHandler(service)
			router := gin.New()
			router.GET("/recipes/:id", handler.GetRecipe)

			req := httptest.NewRequest(http.MethodGet, "/recipes/"+tt.recipeID, nil)
			w := httptest.NewRecorder()
			router.ServeHTTP(w, req)

			assert.Equal(t, tt.expectedStatus, w.Code)

			if tt.expectedError {
				var response map[string]interface{}
				json.Unmarshal(w.Body.Bytes(), &response)
				assert.Contains(t, response, "error")
			}

			service.AssertExpectations(t)
		})
	}
}

func TestRecipeHandler_SearchRecipes(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		queryParams    string
		setupMocks     func(*MockRecipeService)
		expectedStatus int
		expectedError  bool
	}{
		{
			name:        "successful search",
			queryParams: "",
			setupMocks: func(service *MockRecipeService) {
				response := &models.RecipeSearchResponse{
					Recipes:    []models.Recipe{},
					Total:      0,
					Page:       1,
					Limit:      20,
					TotalPages: 0,
				}
				service.On("SearchRecipes", mock.AnythingOfType("*models.RecipeSearchParams")).Return(response, nil)
			},
			expectedStatus: http.StatusOK,
			expectedError:  false,
		},
		{
			name:        "search with filters",
			queryParams: "?search=chicken&mealType[]=dinner&page=1&limit=10",
			setupMocks: func(service *MockRecipeService) {
				response := &models.RecipeSearchResponse{
					Recipes:    []models.Recipe{},
					Total:      0,
					Page:       1,
					Limit:      10,
					TotalPages: 0,
				}
				service.On("SearchRecipes", mock.AnythingOfType("*models.RecipeSearchParams")).Return(response, nil)
			},
			expectedStatus: http.StatusOK,
			expectedError:  false,
		},
		{
			name:        "service error",
			queryParams: "",
			setupMocks: func(service *MockRecipeService) {
				service.On("SearchRecipes", mock.AnythingOfType("*models.RecipeSearchParams")).Return(nil, assert.AnError)
			},
			expectedStatus: http.StatusInternalServerError,
			expectedError:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			service := new(MockRecipeService)
			tt.setupMocks(service)

			handler := handlers.NewRecipeHandler(service)
			router := gin.New()
			router.GET("/recipes", handler.SearchRecipes)

			req := httptest.NewRequest(http.MethodGet, "/recipes"+tt.queryParams, nil)
			w := httptest.NewRecorder()
			router.ServeHTTP(w, req)

			assert.Equal(t, tt.expectedStatus, w.Code)

			if tt.expectedError {
				var response map[string]interface{}
				json.Unmarshal(w.Body.Bytes(), &response)
				assert.Contains(t, response, "error")
			} else {
				var response models.RecipeSearchResponse
				json.Unmarshal(w.Body.Bytes(), &response)
				assert.NotNil(t, response.Recipes)
			}

			service.AssertExpectations(t)
		})
	}
}

func TestRecipeHandler_DeleteRecipe(t *testing.T) {
	gin.SetMode(gin.TestMode)

	recipeID := uuid.New()

	tests := []struct {
		name           string
		recipeID       string
		setupMocks     func(*MockRecipeService)
		expectedStatus int
		expectedError  bool
	}{
		{
			name:     "successful deletion",
			recipeID: recipeID.String(),
			setupMocks: func(service *MockRecipeService) {
				service.On("DeleteRecipe", recipeID).Return(nil)
			},
			expectedStatus: http.StatusOK,
			expectedError:  false,
		},
		{
			name:     "invalid UUID",
			recipeID: "invalid-uuid",
			setupMocks: func(service *MockRecipeService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  true,
		},
		{
			name:     "service error",
			recipeID: recipeID.String(),
			setupMocks: func(service *MockRecipeService) {
				service.On("DeleteRecipe", recipeID).Return(assert.AnError)
			},
			expectedStatus: http.StatusInternalServerError,
			expectedError:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			service := new(MockRecipeService)
			tt.setupMocks(service)

			handler := handlers.NewRecipeHandler(service)
			router := gin.New()
			router.DELETE("/recipes/:id", handler.DeleteRecipe)

			req := httptest.NewRequest(http.MethodDelete, "/recipes/"+tt.recipeID, nil)
			w := httptest.NewRecorder()
			router.ServeHTTP(w, req)

			assert.Equal(t, tt.expectedStatus, w.Code)

			if tt.expectedError {
				var response map[string]interface{}
				json.Unmarshal(w.Body.Bytes(), &response)
				assert.Contains(t, response, "error")
			}

			service.AssertExpectations(t)
		})
	}
}

func TestRecipeHandler_ImportRecipe(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		requestBody    interface{}
		setupMocks     func(*MockRecipeService)
		expectedStatus int
		expectedError  bool
	}{
		{
			name: "successful import",
			requestBody: models.ImportRecipeInput{
				URL: "https://example.com/recipe",
			},
			setupMocks: func(service *MockRecipeService) {
				result := &models.ImportRecipeResult{
					Success: true,
					Recipe: &models.Recipe{
						ID:    uuid.New(),
						Title: "Imported Recipe",
					},
				}
				service.On("ImportRecipe", mock.AnythingOfType("*models.ImportRecipeInput")).Return(result, nil)
			},
			expectedStatus: http.StatusCreated,
			expectedError:  false,
		},
		{
			name: "import failure",
			requestBody: models.ImportRecipeInput{
				URL: "https://example.com/invalid-recipe",
			},
			setupMocks: func(service *MockRecipeService) {
				result := &models.ImportRecipeResult{
					Success: false,
					Error:   stringPtr("Failed to parse recipe"),
				}
				service.On("ImportRecipe", mock.AnythingOfType("*models.ImportRecipeInput")).Return(result, nil)
			},
			expectedStatus: http.StatusBadRequest,
			expectedError:  true,
		},
		{
			name:        "invalid JSON",
			requestBody: "invalid json",
			setupMocks:  func(service *MockRecipeService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			service := new(MockRecipeService)
			tt.setupMocks(service)

			handler := handlers.NewRecipeHandler(service)
			router := gin.New()
			router.POST("/recipes/import", handler.ImportRecipe)

			body, _ := json.Marshal(tt.requestBody)
			req := httptest.NewRequest(http.MethodPost, "/recipes/import", bytes.NewBuffer(body))
			req.Header.Set("Content-Type", "application/json")

			w := httptest.NewRecorder()
			router.ServeHTTP(w, req)

			assert.Equal(t, tt.expectedStatus, w.Code)

			if tt.expectedError {
				var response map[string]interface{}
				json.Unmarshal(w.Body.Bytes(), &response)
				// For import, error might be in the result or as an HTTP error
				assert.True(t, 
					response["error"] != nil || 
					(response["success"] == false && response["error"] != nil))
			}

			service.AssertExpectations(t)
		})
	}
}

// Helper function
func stringPtr(s string) *string {
	return &s
}