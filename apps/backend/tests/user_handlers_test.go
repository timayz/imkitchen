package tests

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/handlers"
	"github.com/imkitchen/backend/internal/services"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// MockUserService is a mock implementation of UserService
type MockUserService struct {
	mock.Mock
}

func (m *MockUserService) GetUserProfile(userID string) (*services.UserProfile, error) {
	args := m.Called(userID)
	if args.Error(1) != nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.UserProfile), nil
}

func (m *MockUserService) UpdateUserProfile(userID string, req services.UpdateProfileRequest) (*services.UserProfile, error) {
	args := m.Called(userID, req)
	if args.Error(1) != nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.UserProfile), nil
}

func (m *MockUserService) ChangeEmail(userID string, req services.ChangeEmailRequest, accessToken string) error {
	args := m.Called(userID, req, accessToken)
	return args.Error(0)
}

func (m *MockUserService) DeleteUserAccount(userID string, accessToken string) (*services.ExportDataResponse, error) {
	args := m.Called(userID, accessToken)
	if args.Error(1) != nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.ExportDataResponse), nil
}

func (m *MockUserService) ExportUserData(userID string) (*services.ExportDataResponse, error) {
	args := m.Called(userID)
	if args.Error(1) != nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.ExportDataResponse), nil
}

func TestGetCurrentUserHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		setupContext   func(*gin.Context)
		mockSetup      func(*MockUserService)
		expectedStatus int
		expectedError  string
	}{
		{
			name: "successful user profile retrieval",
			setupContext: func(c *gin.Context) {
				c.Set("UserID", "user-123")
			},
			mockSetup: func(m *MockUserService) {
				m.On("GetUserProfile", "user-123").Return(
					&services.UserProfile{
						ID:                      "user-123",
						Email:                   "test@example.com",
						EmailVerified:           true,
						FirstName:               "Test",
						LastName:                "User",
						DietaryRestrictions:     []string{"vegetarian"},
						CookingSkillLevel:       "intermediate",
						PreferredMealComplexity: "moderate",
					}, nil)
			},
			expectedStatus: http.StatusOK,
		},
		{
			name: "no user ID in context",
			setupContext: func(c *gin.Context) {
				// Don't set UserID
			},
			mockSetup:      func(m *MockUserService) {},
			expectedStatus: http.StatusUnauthorized,
			expectedError:  "User not authenticated",
		},
		{
			name: "user not found",
			setupContext: func(c *gin.Context) {
				c.Set("UserID", "nonexistent-user")
			},
			mockSetup: func(m *MockUserService) {
				m.On("GetUserProfile", "nonexistent-user").Return(
					nil, assert.AnError)
			},
			expectedStatus: http.StatusInternalServerError,
			expectedError:  "Failed to get user profile",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockUserService := new(MockUserService)
			tt.mockSetup(mockUserService)

			router := gin.New()
			router.Use(func(c *gin.Context) {
				tt.setupContext(c)
				c.Next()
			})
			router.GET("/users/me", handlers.GetCurrentUser(mockUserService))

			// Create request
			req := httptest.NewRequest("GET", "/users/me", nil)
			w := httptest.NewRecorder()

			// Execute
			router.ServeHTTP(w, req)

			// Assert
			assert.Equal(t, tt.expectedStatus, w.Code)

			var response map[string]interface{}
			json.Unmarshal(w.Body.Bytes(), &response)

			if tt.expectedError != "" {
				assert.Equal(t, "error", response["status"])
				assert.Contains(t, response["error"].(string), tt.expectedError)
			} else {
				assert.Equal(t, "success", response["status"])
				assert.NotNil(t, response["data"])
			}

			mockUserService.AssertExpectations(t)
		})
	}
}

func TestUpdateCurrentUserHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		requestBody    interface{}
		setupContext   func(*gin.Context)
		mockSetup      func(*MockUserService)
		expectedStatus int
		expectedError  string
	}{
		{
			name: "successful profile update",
			requestBody: services.UpdateProfileRequest{
				FirstName:               stringPtr("Updated"),
				LastName:                stringPtr("Name"),
				CookingSkillLevel:       stringPtr("advanced"),
				PreferredMealComplexity: stringPtr("complex"),
			},
			setupContext: func(c *gin.Context) {
				c.Set("UserID", "user-123")
			},
			mockSetup: func(m *MockUserService) {
				m.On("UpdateUserProfile", "user-123", mock.AnythingOfType("services.UpdateProfileRequest")).Return(
					&services.UserProfile{
						ID:                      "user-123",
						Email:                   "test@example.com",
						EmailVerified:           true,
						FirstName:               "Updated",
						LastName:                "Name",
						CookingSkillLevel:       "advanced",
						PreferredMealComplexity: "complex",
					}, nil)
			},
			expectedStatus: http.StatusOK,
		},
		{
			name: "invalid cooking skill level",
			requestBody: services.UpdateProfileRequest{
				CookingSkillLevel: stringPtr("expert"), // Invalid value
			},
			setupContext: func(c *gin.Context) {
				c.Set("UserID", "user-123")
			},
			mockSetup:      func(m *MockUserService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  "Validation failed",
		},
		{
			name: "no user ID in context",
			requestBody: services.UpdateProfileRequest{
				FirstName: stringPtr("Test"),
			},
			setupContext: func(c *gin.Context) {
				// Don't set UserID
			},
			mockSetup:      func(m *MockUserService) {},
			expectedStatus: http.StatusUnauthorized,
			expectedError:  "User not authenticated",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockUserService := new(MockUserService)
			tt.mockSetup(mockUserService)

			router := gin.New()
			router.Use(func(c *gin.Context) {
				tt.setupContext(c)
				c.Next()
			})
			router.PUT("/users/me", handlers.UpdateCurrentUser(mockUserService))

			// Create request
			jsonBody, _ := json.Marshal(tt.requestBody)
			req := httptest.NewRequest("PUT", "/users/me", bytes.NewBuffer(jsonBody))
			req.Header.Set("Content-Type", "application/json")
			w := httptest.NewRecorder()

			// Execute
			router.ServeHTTP(w, req)

			// Assert
			assert.Equal(t, tt.expectedStatus, w.Code)

			var response map[string]interface{}
			json.Unmarshal(w.Body.Bytes(), &response)

			if tt.expectedError != "" {
				assert.Equal(t, "error", response["status"])
				assert.Contains(t, response["error"].(string), tt.expectedError)
			} else {
				assert.Equal(t, "success", response["status"])
				assert.NotNil(t, response["data"])
			}

			mockUserService.AssertExpectations(t)
		})
	}
}

func TestDeleteCurrentUserHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		setupContext   func(*gin.Context)
		mockSetup      func(*MockUserService)
		expectedStatus int
		expectedError  string
	}{
		{
			name: "successful account deletion",
			setupContext: func(c *gin.Context) {
				c.Set("UserID", "user-123")
				c.Set("Token", "valid-access-token")
			},
			mockSetup: func(m *MockUserService) {
				m.On("DeleteUserAccount", "user-123", "valid-access-token").Return(
					&services.ExportDataResponse{
						Profile: services.UserProfile{
							ID:    "user-123",
							Email: "test@example.com",
						},
						Data: map[string]interface{}{
							"recipes": []interface{}{},
						},
					}, nil)
			},
			expectedStatus: http.StatusOK,
		},
		{
			name: "no user ID in context",
			setupContext: func(c *gin.Context) {
				c.Set("Token", "valid-access-token")
				// Don't set UserID
			},
			mockSetup:      func(m *MockUserService) {},
			expectedStatus: http.StatusUnauthorized,
			expectedError:  "User not authenticated",
		},
		{
			name: "no token in context",
			setupContext: func(c *gin.Context) {
				c.Set("UserID", "user-123")
				// Don't set Token
			},
			mockSetup:      func(m *MockUserService) {},
			expectedStatus: http.StatusUnauthorized,
			expectedError:  "No access token found",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockUserService := new(MockUserService)
			tt.mockSetup(mockUserService)

			router := gin.New()
			router.Use(func(c *gin.Context) {
				tt.setupContext(c)
				c.Next()
			})
			router.DELETE("/users/me", handlers.DeleteCurrentUser(mockUserService))

			// Create request
			req := httptest.NewRequest("DELETE", "/users/me", nil)
			w := httptest.NewRecorder()

			// Execute
			router.ServeHTTP(w, req)

			// Assert
			assert.Equal(t, tt.expectedStatus, w.Code)

			var response map[string]interface{}
			json.Unmarshal(w.Body.Bytes(), &response)

			if tt.expectedError != "" {
				assert.Equal(t, "error", response["status"])
				assert.Contains(t, response["error"].(string), tt.expectedError)
			} else {
				assert.Equal(t, "success", response["status"])
				assert.NotNil(t, response["data"])
			}

			mockUserService.AssertExpectations(t)
		})
	}
}

// Helper function to create string pointers
func stringPtr(s string) *string {
	return &s
}