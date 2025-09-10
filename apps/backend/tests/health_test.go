package tests

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/imkitchen/backend/internal/handlers"
	"github.com/stretchr/testify/assert"
)

func TestHealthCheck(t *testing.T) {
	// Set Gin to test mode
	gin.SetMode(gin.TestMode)
	
	// Create a new router
	router := gin.New()
	
	// Add the health check endpoint
	router.GET("/health", handlers.HealthCheck(nil))
	
	// Create a test request
	req, _ := http.NewRequest("GET", "/health", nil)
	resp := httptest.NewRecorder()
	
	// Perform the request
	router.ServeHTTP(resp, req)
	
	// Assert the response
	assert.Equal(t, http.StatusOK, resp.Code)
	
	// Parse response body
	var response handlers.HealthResponse
	err := json.Unmarshal(resp.Body.Bytes(), &response)
	assert.NoError(t, err)
	
	// Verify response structure
	assert.Equal(t, "healthy", response.Status)
	assert.Equal(t, "1.0.0", response.Version)
	assert.NotEmpty(t, response.Timestamp)
}

func TestReadinessCheck(t *testing.T) {
	// Set Gin to test mode
	gin.SetMode(gin.TestMode)
	
	// Create a new router
	router := gin.New()
	
	// Add the readiness check endpoint
	router.GET("/readiness", handlers.ReadinessCheck(nil))
	
	// Create a test request
	req, _ := http.NewRequest("GET", "/readiness", nil)
	resp := httptest.NewRecorder()
	
	// Perform the request
	router.ServeHTTP(resp, req)
	
	// Assert the response
	assert.Equal(t, http.StatusOK, resp.Code)
	
	// Parse response body
	var response handlers.HealthResponse
	err := json.Unmarshal(resp.Body.Bytes(), &response)
	assert.NoError(t, err)
	
	// Verify response structure
	assert.Equal(t, "ready", response.Status)
	assert.Equal(t, "1.0.0", response.Version)
	assert.NotEmpty(t, response.Timestamp)
	assert.Contains(t, response.Services, "redis")
	assert.Contains(t, response.Services, "database")
}