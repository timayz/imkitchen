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

// MockAuthService is a mock implementation of AuthService
type MockAuthService struct {
	mock.Mock
}

func (m *MockAuthService) Register(req services.RegisterRequest) (*services.AuthResponse, error) {
	args := m.Called(req)
	if args.Error(1) != nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.AuthResponse), nil
}

func (m *MockAuthService) Login(req services.LoginRequest) (*services.AuthResponse, error) {
	args := m.Called(req)
	if args.Error(1) != nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.AuthResponse), nil
}

func (m *MockAuthService) RefreshToken(req services.RefreshTokenRequest) (*services.AuthResponse, error) {
	args := m.Called(req)
	if args.Error(1) != nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.AuthResponse), nil
}

func (m *MockAuthService) Logout(accessToken string) error {
	args := m.Called(accessToken)
	return args.Error(0)
}

func (m *MockAuthService) ForgotPassword(req services.ForgotPasswordRequest) error {
	args := m.Called(req)
	return args.Error(0)
}

func (m *MockAuthService) ResetPassword(req services.ResetPasswordRequest) error {
	args := m.Called(req)
	return args.Error(0)
}

func (m *MockAuthService) ValidateToken(token string) (string, error) {
	args := m.Called(token)
	return args.String(0), args.Error(1)
}

func (m *MockAuthService) GetUserFromToken(token string) (*services.User, error) {
	args := m.Called(token)
	if args.Error(1) != nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*services.User), nil
}

func TestRegisterHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		requestBody    interface{}
		mockSetup      func(*MockAuthService)
		expectedStatus int
		expectedError  string
	}{
		{
			name: "successful registration",
			requestBody: services.RegisterRequest{
				Email:    "test@example.com",
				Password: "Password123!",
				Name:     "Test User",
			},
			mockSetup: func(m *MockAuthService) {
				m.On("Register", mock.AnythingOfType("services.RegisterRequest")).Return(
					&services.AuthResponse{
						AccessToken:  "mock-access-token",
						RefreshToken: "mock-refresh-token",
						ExpiresIn:    3600,
						TokenType:    "Bearer",
						User: services.User{
							ID:            "user-123",
							Email:         "test@example.com",
							EmailVerified: true,
							Name:          "Test User",
						},
					}, nil)
			},
			expectedStatus: http.StatusCreated,
		},
		{
			name: "invalid email format",
			requestBody: services.RegisterRequest{
				Email:    "invalid-email",
				Password: "Password123!",
				Name:     "Test User",
			},
			mockSetup:      func(m *MockAuthService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  "Invalid request body",
		},
		{
			name: "weak password",
			requestBody: services.RegisterRequest{
				Email:    "test@example.com",
				Password: "weak",
				Name:     "Test User",
			},
			mockSetup:      func(m *MockAuthService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  "Validation failed",
		},
		{
			name: "missing name",
			requestBody: services.RegisterRequest{
				Email:    "test@example.com",
				Password: "Password123!",
				Name:     "",
			},
			mockSetup:      func(m *MockAuthService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  "Validation failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockAuthService := new(MockAuthService)
			tt.mockSetup(mockAuthService)

			router := gin.New()
			router.POST("/auth/register", handlers.Register(mockAuthService))

			// Create request
			jsonBody, _ := json.Marshal(tt.requestBody)
			req := httptest.NewRequest("POST", "/auth/register", bytes.NewBuffer(jsonBody))
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

			mockAuthService.AssertExpectations(t)
		})
	}
}

func TestLoginHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		requestBody    interface{}
		mockSetup      func(*MockAuthService)
		expectedStatus int
		expectedError  string
	}{
		{
			name: "successful login",
			requestBody: services.LoginRequest{
				Email:    "test@example.com",
				Password: "Password123!",
			},
			mockSetup: func(m *MockAuthService) {
				m.On("Login", mock.AnythingOfType("services.LoginRequest")).Return(
					&services.AuthResponse{
						AccessToken:  "mock-access-token",
						RefreshToken: "mock-refresh-token",
						ExpiresIn:    3600,
						TokenType:    "Bearer",
						User: services.User{
							ID:            "user-123",
							Email:         "test@example.com",
							EmailVerified: true,
							Name:          "Test User",
						},
					}, nil)
			},
			expectedStatus: http.StatusOK,
		},
		{
			name: "invalid credentials",
			requestBody: services.LoginRequest{
				Email:    "test@example.com",
				Password: "wrongpassword",
			},
			mockSetup: func(m *MockAuthService) {
				m.On("Login", mock.AnythingOfType("services.LoginRequest")).Return(
					nil, assert.AnError)
			},
			expectedStatus: http.StatusUnauthorized,
			expectedError:  "Authentication failed",
		},
		{
			name: "missing email",
			requestBody: services.LoginRequest{
				Email:    "",
				Password: "Password123!",
			},
			mockSetup:      func(m *MockAuthService) {},
			expectedStatus: http.StatusBadRequest,
			expectedError:  "Invalid request body",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockAuthService := new(MockAuthService)
			tt.mockSetup(mockAuthService)

			router := gin.New()
			router.POST("/auth/login", handlers.Login(mockAuthService))

			// Create request
			jsonBody, _ := json.Marshal(tt.requestBody)
			req := httptest.NewRequest("POST", "/auth/login", bytes.NewBuffer(jsonBody))
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

			mockAuthService.AssertExpectations(t)
		})
	}
}

func TestRefreshTokenHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		requestBody    interface{}
		mockSetup      func(*MockAuthService)
		expectedStatus int
		expectedError  string
	}{
		{
			name: "successful token refresh",
			requestBody: services.RefreshTokenRequest{
				RefreshToken: "valid-refresh-token",
			},
			mockSetup: func(m *MockAuthService) {
				m.On("RefreshToken", mock.AnythingOfType("services.RefreshTokenRequest")).Return(
					&services.AuthResponse{
						AccessToken:  "new-access-token",
						RefreshToken: "new-refresh-token",
						ExpiresIn:    3600,
						TokenType:    "Bearer",
						User: services.User{
							ID:            "user-123",
							Email:         "test@example.com",
							EmailVerified: true,
							Name:          "Test User",
						},
					}, nil)
			},
			expectedStatus: http.StatusOK,
		},
		{
			name: "invalid refresh token",
			requestBody: services.RefreshTokenRequest{
				RefreshToken: "invalid-token",
			},
			mockSetup: func(m *MockAuthService) {
				m.On("RefreshToken", mock.AnythingOfType("services.RefreshTokenRequest")).Return(
					nil, assert.AnError)
			},
			expectedStatus: http.StatusUnauthorized,
			expectedError:  "Token refresh failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockAuthService := new(MockAuthService)
			tt.mockSetup(mockAuthService)

			router := gin.New()
			router.POST("/auth/refresh", handlers.RefreshToken(mockAuthService))

			// Create request
			jsonBody, _ := json.Marshal(tt.requestBody)
			req := httptest.NewRequest("POST", "/auth/refresh", bytes.NewBuffer(jsonBody))
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

			mockAuthService.AssertExpectations(t)
		})
	}
}

func TestLogoutHandler(t *testing.T) {
	gin.SetMode(gin.TestMode)

	tests := []struct {
		name           string
		setupContext   func(*gin.Context)
		mockSetup      func(*MockAuthService)
		expectedStatus int
		expectedError  string
	}{
		{
			name: "successful logout",
			setupContext: func(c *gin.Context) {
				c.Set("Token", "valid-access-token")
			},
			mockSetup: func(m *MockAuthService) {
				m.On("Logout", "valid-access-token").Return(nil)
			},
			expectedStatus: http.StatusOK,
		},
		{
			name: "no token in context",
			setupContext: func(c *gin.Context) {
				// Don't set token
			},
			mockSetup:      func(m *MockAuthService) {},
			expectedStatus: http.StatusUnauthorized,
			expectedError:  "No access token found",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			mockAuthService := new(MockAuthService)
			tt.mockSetup(mockAuthService)

			router := gin.New()
			router.Use(func(c *gin.Context) {
				tt.setupContext(c)
				c.Next()
			})
			router.POST("/auth/logout", handlers.Logout(mockAuthService))

			// Create request
			req := httptest.NewRequest("POST", "/auth/logout", nil)
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
			}

			mockAuthService.AssertExpectations(t)
		})
	}
}