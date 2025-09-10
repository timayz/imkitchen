package tests

import (
	"bytes"
	"encoding/json"
	"errors"
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

// MockCommunityImportService mocks the CommunityImportService interface
type MockCommunityImportService struct {
	mock.Mock
}

func (m *MockCommunityImportService) ImportCommunityRecipe(userID, communityRecipeID uuid.UUID, request *models.RecipeImportRequest) (*models.RecipeImportResponse, error) {
	args := m.Called(userID, communityRecipeID, request)
	return args.Get(0).(*models.RecipeImportResponse), args.Error(1)
}

func (m *MockCommunityImportService) GetImportHistory(userID uuid.UUID, page, limit int) ([]models.RecipeImport, int, error) {
	args := m.Called(userID, page, limit)
	return args.Get(0).([]models.RecipeImport), args.Int(1), args.Error(2)
}

func (m *MockCommunityImportService) CheckImportConflict(userID, communityRecipeID uuid.UUID) (*models.ImportConflict, error) {
	args := m.Called(userID, communityRecipeID)
	result := args.Get(0)
	if result == nil {
		return nil, args.Error(1)
	}
	return result.(*models.ImportConflict), args.Error(1)
}

func (m *MockCommunityImportService) GetImportStats(userID uuid.UUID) (*models.ImportStats, error) {
	args := m.Called(userID)
	return args.Get(0).(*models.ImportStats), args.Error(1)
}

func TestRecipeImportHandler_ImportCommunityRecipe(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		userID         string
		requestBody    models.RecipeImportRequest
		setupMock      func(*MockCommunityImportService)
		expectedStatus int
		expectedError  string
	}{
		{
			name:   "successful import",
			userID: uuid.New().String(),
			requestBody: models.RecipeImportRequest{
				CommunityRecipeID:   uuid.New().String(),
				PreserveAttribution: true,
				Customizations: &models.ImportCustomizations{
					Title: stringPtr("Custom Title"),
					Notes: stringPtr("My notes"),
				},
			},
			setupMock: func(mockService *MockCommunityImportService) {
				mockService.On("ImportCommunityRecipe", mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("*models.RecipeImportRequest")).Return(&models.RecipeImportResponse{
					Success:          true,
					PersonalRecipeID: stringPtr("new-recipe-id"),
					Message:          "Recipe successfully imported",
				}, nil)
			},
			expectedStatus: http.StatusCreated,
		},
		{
			name:   "missing community recipe ID",
			userID: uuid.New().String(),
			requestBody: models.RecipeImportRequest{
				CommunityRecipeID:   "",
				PreserveAttribution: true,
			},
			setupMock:      func(mockService *MockCommunityImportService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  "communityRecipeId is required",
		},
		{
			name:   "invalid community recipe ID format",
			userID: uuid.New().String(),
			requestBody: models.RecipeImportRequest{
				CommunityRecipeID:   "invalid-uuid",
				PreserveAttribution: true,
			},
			setupMock:      func(mockService *MockCommunityImportService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  "Invalid community recipe ID format",
		},
		{
			name:   "community recipe not found",
			userID: uuid.New().String(),
			requestBody: models.RecipeImportRequest{
				CommunityRecipeID:   uuid.New().String(),
				PreserveAttribution: true,
			},
			setupMock: func(mockService *MockCommunityImportService) {
				mockService.On("ImportCommunityRecipe", mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("*models.RecipeImportRequest")).Return((*models.RecipeImportResponse)(nil), errors.New("community recipe not found"))
			},
			expectedStatus: http.StatusNotFound,
			expectedError:  "Community recipe not found",
		},
		{
			name:   "recipe already imported",
			userID: uuid.New().String(),
			requestBody: models.RecipeImportRequest{
				CommunityRecipeID:   uuid.New().String(),
				PreserveAttribution: true,
			},
			setupMock: func(mockService *MockCommunityImportService) {
				mockService.On("ImportCommunityRecipe", mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("*models.RecipeImportRequest")).Return((*models.RecipeImportResponse)(nil), errors.New("recipe already imported"))
			},
			expectedStatus: http.StatusConflict,
			expectedError:  "Recipe already exists in your collection",
		},
		{
			name:   "rate limit exceeded",
			userID: uuid.New().String(),
			requestBody: models.RecipeImportRequest{
				CommunityRecipeID:   uuid.New().String(),
				PreserveAttribution: true,
			},
			setupMock: func(mockService *MockCommunityImportService) {
				mockService.On("ImportCommunityRecipe", mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("*models.RecipeImportRequest")).Return((*models.RecipeImportResponse)(nil), errors.New("import rate limit exceeded"))
			},
			expectedStatus: http.StatusTooManyRequests,
			expectedError:  "Import rate limit exceeded. Please try again later",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockService := new(MockCommunityImportService)
			tt.setupMock(mockService)

			handler := handlers.NewRecipeImportHandler(mockService)

			// Create request
			requestBody, _ := json.Marshal(tt.requestBody)
			req, _ := http.NewRequest("POST", "/api/v1/recipes/import", bytes.NewBuffer(requestBody))
			req.Header.Set("Content-Type", "application/json")

			// Setup Gin context
			w := httptest.NewRecorder()
			c, _ := gin.CreateTestContext(w)
			c.Request = req

			// Set user ID in context (simulating auth middleware)
			if tt.userID != "" {
				userUUID := uuid.MustParse(tt.userID)
				c.Set("userID", userUUID)
			}

			// Execute
			handler.ImportCommunityRecipe(c)

			// Assert
			assert.Equal(t, tt.expectedStatus, w.Code)

			var response map[string]interface{}
			err := json.Unmarshal(w.Body.Bytes(), &response)
			assert.NoError(t, err)

			if tt.expectedError != "" {
				assert.Equal(t, tt.expectedError, response["error"])
				assert.Equal(t, false, response["success"])
			} else {
				assert.Equal(t, true, response["success"])
				assert.NotEmpty(t, response["personalRecipeId"])
				assert.NotEmpty(t, response["message"])
			}

			mockService.AssertExpectations(t)
		})
	}
}

func TestRecipeImportHandler_GetImportHistory(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		userID         string
		queryParams    map[string]string
		setupMock      func(*MockCommunityImportService)
		expectedStatus int
		expectedError  string
	}{
		{
			name:   "successful fetch with default pagination",
			userID: uuid.New().String(),
			setupMock: func(mockService *MockCommunityImportService) {
				imports := []models.RecipeImport{
					{
						ID:                uuid.New(),
						UserID:            uuid.New(),
						PersonalRecipeID:  uuid.New(),
						CommunityRecipeID: uuid.New(),
					},
				}
				mockService.On("GetImportHistory", mock.AnythingOfType("uuid.UUID"), 1, 20).Return(imports, 1, nil)
			},
			expectedStatus: http.StatusOK,
		},
		{
			name:   "successful fetch with custom pagination",
			userID: uuid.New().String(),
			queryParams: map[string]string{
				"page":  "2",
				"limit": "10",
			},
			setupMock: func(mockService *MockCommunityImportService) {
				imports := []models.RecipeImport{}
				mockService.On("GetImportHistory", mock.AnythingOfType("uuid.UUID"), 2, 10).Return(imports, 0, nil)
			},
			expectedStatus: http.StatusOK,
		},
		{
			name:           "user not authenticated",
			userID:         "",
			setupMock:      func(mockService *MockCommunityImportService) {},
			expectedStatus: http.StatusUnauthorized,
			expectedError:  "User not authenticated",
		},
		{
			name:   "service error",
			userID: uuid.New().String(),
			setupMock: func(mockService *MockCommunityImportService) {
				mockService.On("GetImportHistory", mock.AnythingOfType("uuid.UUID"), 1, 20).Return([]models.RecipeImport{}, 0, errors.New("database error"))
			},
			expectedStatus: http.StatusInternalServerError,
			expectedError:  "Failed to fetch import history",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockService := new(MockCommunityImportService)
			tt.setupMock(mockService)

			handler := handlers.NewRecipeImportHandler(mockService)

			// Create request
			req, _ := http.NewRequest("GET", "/api/v1/recipes/import/history", nil)

			// Add query parameters
			if tt.queryParams != nil {
				q := req.URL.Query()
				for k, v := range tt.queryParams {
					q.Add(k, v)
				}
				req.URL.RawQuery = q.Encode()
			}

			// Setup Gin context
			w := httptest.NewRecorder()
			c, _ := gin.CreateTestContext(w)
			c.Request = req

			// Set user ID in context (simulating auth middleware)
			if tt.userID != "" {
				userUUID := uuid.MustParse(tt.userID)
				c.Set("userID", userUUID)
			}

			// Execute
			handler.GetImportHistory(c)

			// Assert
			assert.Equal(t, tt.expectedStatus, w.Code)

			var response map[string]interface{}
			err := json.Unmarshal(w.Body.Bytes(), &response)
			assert.NoError(t, err)

			if tt.expectedError != "" {
				assert.Equal(t, tt.expectedError, response["error"])
			} else {
				assert.NotNil(t, response["imports"])
				assert.NotNil(t, response["pagination"])
			}

			mockService.AssertExpectations(t)
		})
	}
}

func TestRecipeImportHandler_CheckImportConflict(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name             string
		userID           string
		communityRecipeID string
		setupMock        func(*MockCommunityImportService)
		expectedStatus   int
		expectedError    string
		expectedConflict bool
	}{
		{
			name:              "no conflict",
			userID:            uuid.New().String(),
			communityRecipeID: uuid.New().String(),
			setupMock: func(mockService *MockCommunityImportService) {
				mockService.On("CheckImportConflict", mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("uuid.UUID")).Return((*models.ImportConflict)(nil), nil)
			},
			expectedStatus:   http.StatusOK,
			expectedConflict: false,
		},
		{
			name:              "conflict exists",
			userID:            uuid.New().String(),
			communityRecipeID: uuid.New().String(),
			setupMock: func(mockService *MockCommunityImportService) {
				conflict := &models.ImportConflict{
					ExistingRecipeID:    uuid.New().String(),
					ExistingRecipeTitle: "Existing Recipe",
					ConflictType:        "duplicate_import",
				}
				mockService.On("CheckImportConflict", mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("uuid.UUID")).Return(conflict, nil)
			},
			expectedStatus:   http.StatusOK,
			expectedConflict: true,
		},
		{
			name:              "invalid community recipe ID format",
			userID:            uuid.New().String(),
			communityRecipeID: "invalid-uuid",
			setupMock:         func(mockService *MockCommunityImportService) {},
			expectedStatus:    http.StatusBadRequest,
			expectedError:     "Invalid community recipe ID format",
		},
		{
			name:              "user not authenticated",
			userID:            "",
			communityRecipeID: uuid.New().String(),
			setupMock:         func(mockService *MockCommunityImportService) {},
			expectedStatus:    http.StatusUnauthorized,
			expectedError:     "User not authenticated",
		},
		{
			name:              "service error",
			userID:            uuid.New().String(),
			communityRecipeID: uuid.New().String(),
			setupMock: func(mockService *MockCommunityImportService) {
				mockService.On("CheckImportConflict", mock.AnythingOfType("uuid.UUID"), mock.AnythingOfType("uuid.UUID")).Return((*models.ImportConflict)(nil), errors.New("database error"))
			},
			expectedStatus: http.StatusInternalServerError,
			expectedError:  "Failed to check import conflict",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockService := new(MockCommunityImportService)
			tt.setupMock(mockService)

			handler := handlers.NewRecipeImportHandler(mockService)

			// Create request
			req, _ := http.NewRequest("GET", "/api/v1/recipes/import/check/"+tt.communityRecipeID, nil)

			// Setup Gin context with path parameter
			w := httptest.NewRecorder()
			c, _ := gin.CreateTestContext(w)
			c.Request = req
			c.Params = gin.Params{
				{Key: "communityRecipeId", Value: tt.communityRecipeID},
			}

			// Set user ID in context (simulating auth middleware)
			if tt.userID != "" {
				userUUID := uuid.MustParse(tt.userID)
				c.Set("userID", userUUID)
			}

			// Execute
			handler.CheckImportConflict(c)

			// Assert
			assert.Equal(t, tt.expectedStatus, w.Code)

			var response map[string]interface{}
			err := json.Unmarshal(w.Body.Bytes(), &response)
			assert.NoError(t, err)

			if tt.expectedError != "" {
				assert.Equal(t, tt.expectedError, response["error"])
			} else {
				assert.Equal(t, tt.expectedConflict, response["hasConflict"])
				if tt.expectedConflict {
					assert.NotNil(t, response["conflict"])
				} else {
					assert.Nil(t, response["conflict"])
				}
			}

			mockService.AssertExpectations(t)
		})
	}
}

// Helper function to create string pointers
func stringPtr(s string) *string {
	return &s
}